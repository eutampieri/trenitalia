use chrono::prelude::*;
use serde::{Serialize, Deserialize};

mod mapping;

static WORDS_EQUALITY_THRESHOLD: f64 = 0.70;

//pub struct TrainTrips(Vec<TrainTrip>);

/// Module which contains utilities
mod utils {
    /// This function returns the normalized typing distance between two strings
    pub fn match_strings(first: &str, second: &str) -> f64 {
        if first.to_lowercase() == second.to_lowercase() {
            1.0
        } else {
            strsim::normalized_damerau_levenshtein(&first.to_lowercase(), &second.to_lowercase())
        }
    }
}

/*impl TrainTrips{
    /// This method calculates the total duration of a trip
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.0[0].departure.1).clone();
        let arrivo = (&self.0[&self.0.len() - 1].arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
}*/

// TODO Aggiungere tipi treno
/// Train type and number representation
#[derive(Debug, Clone)]
pub enum TrainNumber{
    Regionale{number: u32},
    RegionaleVeloce{number: u32},
    InterCity{number: u32},
    FrecciaRossa{number: u32},
    FrecciaArgento{number: u32},
    FrecciaBianca{number: u32},
    InterCityNotte{number: u32},
    EuroNight{number: u32},
    EuroCity{number: u32},
    Bus{number: u32},
    //EuroCityÖBBDB(u32),
    /// Unknown train, the second field is the train type as returned from the API
    Unknown{number: u32, name: String},
}

/// A specific stop in a train trip
#[derive(Debug)]
pub struct TrainTripStop {
    pub station: TrainStation,
    pub platform: String,
    pub arrival: Option<chrono::DateTime<chrono::Local>>,
    pub departure: Option<chrono::DateTime<chrono::Local>>,
    pub expected_arrival: Option<chrono::DateTime<chrono::Local>>,
    pub expected_departure: Option<chrono::DateTime<chrono::Local>>,
}

/// A specific stop in a train trip
#[derive(Debug)]
pub struct DetailedTrainTripStop {
    pub station: TrainStation,
    pub platform: String,
    pub arrival: Option<chrono::DateTime<chrono::Local>>,
    pub departure: Option<chrono::DateTime<chrono::Local>>,
    pub expected_arrival: Option<chrono::DateTime<chrono::Local>>,
    pub expected_departure: Option<chrono::DateTime<chrono::Local>>,
}

/// A train trip with stops specified
pub struct DetailedTrainTrip {
    pub from: TrainStation,
    pub to: TrainStation, 
    pub train_number: TrainNumber,
    pub stops: Vec<TrainTripStop>,
}

/// A train trip between two stations. Stops aren't specified
#[derive(Debug, Clone)]
pub struct TrainTrip {
    pub train_number: TrainNumber,
    /// Specify the station and time of arrival
    pub arrival: (TrainStation, chrono::DateTime<chrono::Local>),
    /// Specify the station and time of departure
    pub departure: (TrainStation, chrono::DateTime<chrono::Local>),
}

impl TrainTrip{
    /// This method returns the trip's duration
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.departure.1).clone();
        let arrivo = (&self.arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
}

#[derive(Debug)]
pub struct TrainInfo {
    pub current_station: TrainStation,
    pub current_delay: i16,
    pub is_at_station: bool,
    pub stops: Vec<DetailedTrainTripStop>,
}

impl TrainInfo {
    pub fn from(vtvec: &Vec<mapping::VTDetailedTrainTripLeg>, trenitalia: &Trenitalia) -> Self {
        let mut delay: i16 = 0;
        let mut current_station: TrainStation = trenitalia.find_train_station(&vtvec[&vtvec.len()-1].stazione).unwrap().clone();
        let mut in_station: bool = false;
        let mut stations_list: Vec<DetailedTrainTripStop> = Vec::new();
        for stop in vtvec {
            let this_station = trenitalia.find_train_station(&stop.stazione).unwrap();
            if stop.stazioneCorrente {
                current_station = this_station.clone();
                if let Some(r) = stop.fermata.partenzaReale { 
                    if let Some(t) = stop.fermata.partenza_teorica{
                        delay = ((r as i64 - t as i64)/60000i64) as i16;
                    }
                } else if let Some(t) = stop.fermata.partenza_teorica {
                    delay = (((chrono::Local::now().timestamp_millis() - t as i64)/60000i64) as i16).max(0);
                }
                if stop.fermata.arrivoReale.is_some() && stop.fermata.partenzaReale.is_none() {
                    // If the train has arrived but not departed yet
                    in_station = true;
                }
            }
            let this_stop = DetailedTrainTripStop{
                arrival: stop.fermata.arrivoReale.map(|ts| chrono::Local.timestamp((ts/1000) as i64, 0)),
                departure: stop.fermata.partenzaReale.map(|ts| chrono::Local.timestamp((ts/1000) as i64, 0)),
                expected_arrival: stop.fermata.arrivo_teorico.map(|ts| chrono::Local.timestamp((ts/1000) as i64, 0)),
                expected_departure: stop.fermata.partenza_teorica.map(|ts| chrono::Local.timestamp((ts/1000) as i64, 0)),
                platform: stop.fermata.binarioEffettivoPartenzaDescrizione.as_ref().unwrap_or(
                    stop.fermata.binarioProgrammatoPartenzaDescrizione.as_ref().unwrap_or(
                        stop.fermata.binarioEffettivoArrivoDescrizione.as_ref().unwrap_or(
                            stop.fermata.binarioProgrammatoArrivoDescrizione.as_ref().unwrap_or(&"?".to_string())
                        )
                    )
                ).to_string(),
                station: this_station.clone()
            };
            stations_list.push(this_stop);
        }
        TrainInfo{current_delay: delay, is_at_station: in_station, current_station: current_station, stops: stations_list}
    }
}

/// Struct that holds the train station data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainStation {
    /// Three-charachters ID
    pub id: String,
    /// Trenitalia region ID
    pub region_id: u8,
    /// Tuple that contains latitude and longitude
    pub position: (f64, f64),
    /// List of possible aliases of a station
    pub aliases: Vec<String>,
    /// Station ID used for the ViaggaTreno API
    pub vt_id: Option<String>,
    /// Station name used for the LeFrecce API
    pub lefrecce_name: Option<String>,
}

impl TrainStation {
    /// Get the short version of ViaggiaTreno ID
    fn short_id(&self) -> Option<String> {
        match &self.vt_id {
            None => None,
            Some(x) => Some(str::replace(x, "S", "").parse::<u16>().unwrap().to_string())
        }
    }
    /// Get the station's name (the first alias)
    pub fn get_name(&self) -> &str {
        &self.aliases[0]
    }
}

pub struct Trenitalia {
    stations: Vec<TrainStation>,
    /// Hash map that matches aliases to indexes of the `stations` vector
    fast_station_lookup: std::collections::HashMap<String, usize>
}

impl Trenitalia {
    /// Creates a new Trenitalia instance
    pub fn new() -> Trenitalia {
        let id_to_lf_tsv = include_str!("../id_lf_map.tsv");
        let id_to_lf: std::collections::HashMap<String, String> = std::collections::HashMap::from(id_to_lf_tsv.split("\n").collect::<Vec<&str>>()
            .iter().map(|&x| x.split("\t").collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>()
            .iter().map(|x| (String::from(*&x[0]), String::from(*&x[1]))).collect::<Vec<(String, String)>>().into_iter().collect());
        let id_to_vt_tsv = include_str!("../id_vt.tsv");
        let id_to_vt: std::collections::HashMap<String, String> = std::collections::HashMap::from(id_to_vt_tsv.split("\n").collect::<Vec<&str>>()
            .iter().map(|&x| x.split("\t").collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>()
            .iter().map(|x| (String::from(*&x[0]), String::from(*&x[1]))).collect::<Vec<(String, String)>>().into_iter().collect());

        let aliases_tsv = include_str!("../aliases.tsv");
        let aliases: Vec<Vec<&str>> = aliases_tsv.split("\n").collect::<Vec<&str>>()
            .iter().map(|&x| x.split("\t").collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();

        let station_list_tsv = include_str!("../stations.tsv");
        let station_list = station_list_tsv.split("\n").collect::<Vec<&str>>();
        let mapped_stations: Vec<TrainStation> = station_list.iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>().iter()
            .map(|x|  {let mut a = vec![String::from(x[0])];
                let mut v: Vec<String> = Vec::new();
                for alias in &aliases {
                    if &alias[1] == &x[1] {
                        v.push(String::from(alias[0]))
                    }
                }
                a.append(&mut v);TrainStation{
                id: String::from(x[1]),
                aliases: a,
                position: (
                    x[3].parse::<f64>().unwrap(),
                    x[4].parse::<f64>().unwrap()
                ),
                region_id: x[2].parse::<u8>().unwrap(),
                lefrecce_name: id_to_lf.get(x[1]).map(|x| String::from(x)),
                vt_id: id_to_vt.get(x[1]).map(|x| String::from(x)),
            }}).collect();
        let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for i in 0..mapped_stations.len() {
            for alias in &mapped_stations[i].aliases{
                lookup.insert(String::from(alias), i);
            }
            if mapped_stations[i].lefrecce_name.is_some() {
                lookup.insert(String::from(mapped_stations[i].lefrecce_name.as_ref().unwrap_or(&"".to_string())).to_uppercase(), i);
            }
        }
        Trenitalia{stations: mapped_stations, fast_station_lookup: lookup}
    }
    /// Builds a TrainNumber enum from the train number and train type
    fn match_train_type(&self, description: &str, number: u32) -> TrainNumber{
        let train_type = match description {
            "RV" => TrainNumber::RegionaleVeloce{number: number},
            "Regionale" => TrainNumber::Regionale{number: number},
            "Frecciarossa" => TrainNumber::FrecciaRossa{number: number},
            "Frecciaargento" => TrainNumber::FrecciaArgento{number: number},
            "IC" => TrainNumber::InterCity{number: number},
            "Frecciabianca" => TrainNumber::FrecciaBianca{number: number},
            "ICN" => TrainNumber::InterCityNotte{number: number},
            "EN" => TrainNumber::EuroNight{number: number},
            "EC" => TrainNumber::EuroCity{number: number},
            "REG" => TrainNumber::Regionale{number: number},
            "Autobus" => TrainNumber::Bus{number: number},
            "BUS" => TrainNumber::Bus{number: number},
            "FR" => TrainNumber::FrecciaRossa{number: number},
            "FA" => TrainNumber::FrecciaArgento{number: number},
            "FB" => TrainNumber::FrecciaBianca{number: number},
            "ECB" => TrainNumber::EuroCity{number: number},
            _ => TrainNumber::Unknown{number: number, name: String::from(description)},
        };
        match train_type{
            TrainNumber::Unknown{number: _, name: _} =>{
                let url = format!("https://eutampieri.eu/tipi_treno.php?tipo={}", description.replace(" ", "%20"));
                let _ = reqwest::get(url.as_str());
            },
            _ => {}
        }
        train_type
    }
    /// Find a trip between two stations using LeFrecce API
    fn find_trips_lefrecce(&self, from: &TrainStation, to: &TrainStation, when: &chrono::DateTime<chrono::Local>) -> Vec<Vec<TrainTrip>>{
        if from.id == to.id || from.lefrecce_name.is_none() || from.lefrecce_name.is_none(){
            return vec![];
        }
        let mut result: Vec<Vec<TrainTrip>> = Vec::new();
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let url = format!("https://www.lefrecce.it/msite/api/solutions?origin={}&destination={}&arflag=A&adate={}&atime={}&adultno=1&childno=0&direction=A&frecce=false&onlyRegional=false",
            from.lefrecce_name.clone().unwrap().replace(" ", "%20"),
            to.lefrecce_name.clone().unwrap().replace(" ", "%20"),
            when.format("%d/%m/%Y"),
            when.format("%H")
        );
        if cfg!(debug_assertions) {
            println!("{}", url);
        }
        let body: Vec<mapping::LFSolution> = client.get(url.as_str()).send().unwrap().json().unwrap();
        for solution in &body {
            let mut train_trips: Vec<TrainTrip> = Vec::new();
            let url_details = format!("https://www.lefrecce.it/msite/api/solutions/{}/standardoffers", solution.idsolution);
            if cfg!(debug_assertions) {
                println!("{}", url_details);
            }
            let body_details: mapping::LFDetailedSolution = client.get(url_details.as_str()).send().unwrap().json().unwrap();
            for leg in &body_details.leglist {
                for train in &leg.segments {
                    if train.trainidentifier == String::from("Same") {
                        continue;
                    }
                    let acronym = train.trainacronym.as_ref().map_or(String::from(""), |x| String::from(x.as_str()));
                    let train_name_exploded: Vec<&str> = train.trainidentifier.split(' ').collect();
                    let train_number = train_name_exploded[&train_name_exploded.len()-1];
                    let from = &self.stations[
                    *self.fast_station_lookup.get(&train.departurestation.to_uppercase())
                    .or_else(|| {
                        let url = format!("https://eutampieri.eu/fix_localita.php?nome={}", &train.departurestation);
                        let _ = reqwest::get(url.as_str());
                        None
                    }).expect("Inconsistency in Trenitalia")];
                    let to = &self.stations[
                    *self.fast_station_lookup.get(&train.arrivalstation.to_uppercase())
                    .or_else(|| {
                        let url = format!("https://eutampieri.eu/fix_localita.php?nome={}", &train.arrivalstation);
                        let _ = reqwest::get(url.as_str());
                        None
                    }).expect("Inconsistency in Trenitalia")];
                    train_trips.push(TrainTrip{
                        departure: (from.clone(),
                            chrono::Local.datetime_from_str(train.departuretime.as_str(), "%+").expect("Data non valida"),
                        ),
                        arrival: (to.clone(),
                            chrono::Local.datetime_from_str(train.arrivaltime.as_str(), "%+").expect("Data non valida"),
                        ),
                        train_number: self.match_train_type(&acronym, train_number.parse::<u32>().unwrap())
                    });
                }
            }
            result.push(train_trips);
        }
        /*if cfg!(debug_assertions) {
            println!("{:?}", result);
        }*/
        result
    }
    /// Find a trip between two stations using ViaggiaTreno API and falling back to LeFrecce
    pub fn find_trips(&self, from: &TrainStation, to: &TrainStation, when: &chrono::DateTime<chrono::Local>) -> Vec<Vec<TrainTrip>>{
        let mut result: Vec<Vec<TrainTrip>> = Vec::new();
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/soluzioniViaggioNew/{}/{}/{}",
            from.short_id().unwrap(),
            to.short_id().unwrap(),
            when.format("%FT%T")
        );
        if cfg!(debug_assertions) {
            println!("{}", url);
        }
        let body: mapping::VTJourneySearchResult = reqwest::get(url.as_str()).unwrap().json().unwrap();
        if body.soluzioni.len() == 0{
            return self.find_trips_lefrecce(from, to, when);
        }
        for soluzione in body.soluzioni {
            let mut train_trips: Vec<TrainTrip> = Vec::new();
            if cfg!(debug_assertions) {
                println!("expected: {}, found: {}, delta: {}",
                &from.get_name(),
                &soluzione.vehicles[0].origine.as_ref().unwrap_or(&String::from("")),
                utils::match_strings(
                &soluzione.vehicles[0].origine.as_ref().unwrap_or(&String::from("")).to_lowercase(),
                &from.get_name()
            ));
            }
            if utils::match_strings(
                &soluzione.vehicles[0].origine.as_ref().unwrap_or(&String::from("")),
                &from.get_name()
            ) < WORDS_EQUALITY_THRESHOLD {
                let filling_to = &self.stations[
                    *self.fast_station_lookup.get(
                        soluzione.vehicles[0].origine.as_ref().unwrap_or(&String::from("")))
                    .or_else(|| {
                        let url = format!("https://eutampieri.eu/fix_localita.php?nome={}", soluzione.vehicles[0].origine.as_ref().unwrap_or(&String::from("")));
                        let _ = reqwest::get(url.as_str());
                        None
                    }).expect("Inconsistency in Trenitalia")];
                if cfg!(debug_assertions) {
                    println!("filling_to = {:?}", filling_to);
                }
                let filling_solutions = self.find_trips_lefrecce(from, filling_to, when);
                for filling_solution in filling_solutions.iter() {
                    if filling_solution[0].departure.1 >= chrono::Local.timestamp(when.timestamp(), 0) && filling_solution[&filling_solution.len()-1].arrival.1 <= chrono::Local.datetime_from_str(soluzione.vehicles[0].orarioPartenza.as_str(), "%FT%T").expect("Data non valida") {
                        for filling_train in filling_solution {
                            train_trips.push(filling_train.clone());
                        }
                        break;
                    }
                }
            }
            let mut old_to: Option<&str> = None;
            let mut old_to_stn = to.clone();
            let mut old_ts = chrono::Local.timestamp(when.timestamp(), 0);
            for train_trip in soluzione.vehicles.iter() {
                let from = &self.stations[
                    *self.fast_station_lookup.get(
                        train_trip.origine.as_ref().unwrap_or(&String::from("")))
                    .or_else(|| {
                        let url = format!("https://eutampieri.eu/fix_localita.php?nome={}", train_trip.origine.as_ref().unwrap_or(&String::from("")));
                        let _ = reqwest::get(url.as_str());
                        None
                    }).expect("Inconsistency in Trenitalia")];
                let to = &self.stations[
                    *self.fast_station_lookup.get(
                        train_trip.destinazione.as_ref().unwrap_or(&String::from("")))
                    .or_else(|| {
                        let url = format!("https://eutampieri.eu/fix_localita.php?nome={}", train_trip.destinazione.as_ref().unwrap_or(&String::from("")));
                        let _ = reqwest::get(url.as_str());
                        None
                    }).expect("Inconsistency in Trenitalia")];
                if old_to.is_some() && old_to!=Some(&from.get_name()){
                    let filling_solutions = self.find_trips_lefrecce(&old_to_stn, from, &old_ts);
                    for filling_solution in filling_solutions.iter() {
                        if filling_solution[0].departure.1 >= old_ts && filling_solution[&filling_solution.len()-1].arrival.1 <= chrono::Local.datetime_from_str(train_trip.orarioPartenza.as_str(), "%FT%T").expect("Data non valida") {
                            for filling_train in filling_solution {
                                train_trips.push(filling_train.clone());
                            }
                            break;
                        }
                    }
                }
                old_to = Some(&to.get_name());
                old_to_stn = to.clone();
                old_ts = chrono::Local.datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T").expect("Data non valida");
                train_trips.push(TrainTrip{
                    departure: (from.clone(),
                        chrono::Local.datetime_from_str(train_trip.orarioPartenza.as_str(), "%FT%T").expect("Data non valida"),
                    ),
                    arrival: (to.clone(),
                        chrono::Local.datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T").expect("Data non valida"),
                    ),
                    train_number: self.match_train_type(&train_trip.categoriaDescrizione, train_trip.numeroTreno.parse::<u32>().unwrap())
                });
            }
            if cfg!(debug_assertions) {
                println!("expected: {}, found: {}, delta: {}",
                &to.get_name(),
                &soluzione.vehicles[&soluzione.vehicles.len()-1].destinazione.as_ref().unwrap_or(&String::from("")),
                utils::match_strings(
                &soluzione.vehicles[&soluzione.vehicles.len()-1].destinazione.as_ref().unwrap_or(&String::from("")),
                &to.get_name(),
            ));
            }
            if utils::match_strings(
                &soluzione.vehicles[&soluzione.vehicles.len()-1].destinazione.as_ref().unwrap_or(&String::from("")),
                &to.get_name()
            ) < WORDS_EQUALITY_THRESHOLD {
                let filling_from = &self.stations[
                    *self.fast_station_lookup.get(
                        soluzione.vehicles[&soluzione.vehicles.len()-1].destinazione.as_ref().unwrap_or(&String::from("")))
                    .or_else(|| {
                        let url = format!("https://eutampieri.eu/fix_localita.php?nome={}", soluzione.vehicles[&soluzione.vehicles.len()-1].destinazione.as_ref().unwrap_or(&String::from("")));
                        let _ = reqwest::get(url.as_str());
                        None
                    }).expect("Inconsistency in Trenitalia")];
                if cfg!(debug_assertions) {
                    println!("filling_from = {:?}", filling_from);
                }
                let filling_solutions = self.find_trips_lefrecce(filling_from, to, when);
                for filling_solution in filling_solutions.iter() {
                    if filling_solution[0].departure.1 >= chrono::Local.datetime_from_str(soluzione.vehicles[&soluzione.vehicles.len()-1].orarioArrivo.as_str(), "%FT%T").expect("Data non valida") {
                        for filling_train in filling_solution {
                            train_trips.push(filling_train.clone());
                        }
                        break;
                    }
                }
            }
            result.push(train_trips);
        }
        result
    }

    /// Call to the ViaggiaTreno station lookup API
    pub fn find_train_station_online(&self, name: &str) -> Option<&TrainStation> {
        //return Some(&self.stations[0]);
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/autocompletaStazione/{}", name);
        if cfg!(debug_assertions) {
            println!("{}", url);
        }
        let response = reqwest::get(&url).unwrap().text().unwrap();
        if response.len() == 0 {
            return None;
        }
        let body: Vec<Vec<&str>> = response.trim_end_matches('\n')
        .split("\n").collect::<Vec<&str>>().iter()
        .map(|&x| x.split("|").collect::<Vec<&str>>()).collect();
        if body.len() == 0 {
            None
        } else {
            for station in &self.stations {
                let vt_id = match &station.vt_id {
                    Some(x) => Some(String::from(x)),
                    None => None
                };

                if station.vt_id.is_none(){
                    continue
                } else if vt_id.unwrap() == body[0][1] {
                    return Some(station);
                }
            }
            None
        }
    }

    /// Return a station object reference that has the requested ID
    pub fn get_train_station(&self, id: &str) -> Option<&TrainStation> {
        if cfg!(debug_assertions) {
            println!("{:?}", id);
        }
        for station in &self.stations {
            if &station.id == id {
                return Some(station);
            }
        }
        None
    }

    /// Look for a train station
    pub fn find_train_station(&self, name: &str) -> Option<&TrainStation> {
        let mut min_diff = 0.0;
        let mut found_station = &self.stations[0];
        match self.fast_station_lookup.get(&name.to_uppercase()) {
            Some(x) => return Some(&self.stations[*x]),
            None => {
                for station in &self.stations {
                    for alias in &station.aliases {
                        let diff = utils::match_strings(alias, &name);
                        if cfg!(debug_assertions) {
                            //println!("Difference between {} and {} = {}", &station.name, &name, diff);
                        }
                        if diff == 1.0 {
                            return Some(station);
                        }
                        if diff > min_diff {
                            min_diff = diff;
                            found_station = station;
                        }
                    }
                }
                return if min_diff >= WORDS_EQUALITY_THRESHOLD {Some(found_station)} else {None}
            }
        };
    }

    /// Get train details from ViaggiaTreno
    fn train_info_raw(&self, number: &str, from: &str) -> TrainInfo {
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/tratteCanvas/{}/{}", from, number);
        let response: Vec<mapping::VTDetailedTrainTripLeg> = reqwest::get(&url).unwrap().json().unwrap();
        TrainInfo::from(&response, self)
    }

    /// Get train details, provided that you know the originating station
    pub fn train_info(&self, number: &str, from: String) -> Result<TrainInfo, &str> {
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}", number);
        let response = reqwest::get(&url).unwrap().text().unwrap();
        let body: Vec<Vec<&str>> = response.trim_end_matches('\n')
        .split("\n").collect::<Vec<&str>>().iter()
        .map(|&x| x.split("|").collect::<Vec<&str>>()).collect();
        let train_station_of_origination: &str = match body.len() {
            1 => body[0][1].split('-').collect::<Vec<&str>>()[1],
            0 => {
                return Err("No train found");
            },
            _ => {
                let mut station_code = "";
                let mut min_diff = 0.0;
                for option in body {
                    let diff = utils::match_strings(
                        &option[0].split('-').collect::<Vec<&str>>()[1].trim_start().to_lowercase(),
                        &from.to_lowercase()
                    );
                    if diff < min_diff {
                        min_diff = diff;
                        station_code = option[1].split('-').collect::<Vec<&str>>()[1];
                    }
                    if diff == 1.0 {
                        break;
                    }
                }
                if min_diff == 0.0 {return Err("Train not found");} else {station_code}
            }
        };
        Ok(self.train_info_raw(number, train_station_of_origination))
    }

    /// Get train details, knowing that it calls at a certain station
    pub fn train_info_calling_at(&self, number: &str, calling_at: &TrainStation) -> Result<TrainInfo, &str> {
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}", number);
        let response = reqwest::get(&url).unwrap().text().unwrap();
        let body: Vec<Vec<&str>> = response.trim_end_matches('\n')
        .split("\n").collect::<Vec<&str>>().iter()
        .map(|&x| x.split("|").collect::<Vec<&str>>()).collect();
        match body.len() {
            1 => Ok(self.train_info_raw(number, body[0][1].split('-').collect::<Vec<&str>>()[1])),
            0 => {
                Err("Train not found")
            },
            _ => {
                for option in body {
                    let train_info = self.train_info_raw(number, option[1].split('-').collect::<Vec<&str>>()[1]);
                    for stop in &train_info.stops {
                        if stop.station.id == calling_at.id {
                            return Ok(train_info);
                        }
                    }
                }
                return Err("Train not found");
            }
        }
    }
    /// Finds the nearest station from a point
    pub fn nearest_station(&self, point: (f64,f64)) -> &TrainStation {
        let mut min_dist = std::f64::MAX;
        let mut sta = &self.stations[0];
        for station in &self.stations {
            let dist_sq = (station.position.0 - point.0).powf(2.0) + (station.position.1 - point.1).powf(2.0);
            if dist_sq < min_dist {
                sta = station;
                min_dist = dist_sq;
            }
        }
        sta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_test() {
        let t = Trenitalia::new();
        println!("{:?}", t.find_train_station("bolzano"));
        assert!(t.fast_station_lookup.get("Bolzano").is_some());
    }

    #[test]
    fn test(){
        let t = Trenitalia::new();
        let _calalzo = t.nearest_station((46.45, 12.383333));
        let _carnia = t.nearest_station((46.374318, 13.134141));
        let imola = t.nearest_station((44.3533, 11.7141));
        let cesena = t.nearest_station((44.133333, 12.233333));
        //println!("{:?}, {:?}", imola, calalzo);
        println!("{:?}", t.find_train_station("bologna centrale"));
        let _bologna = t.find_train_station("vipiteno").unwrap();
        println!("{:?}", t.find_trips(cesena, imola, &chrono::Local::now()));/*
            .iter()
            .map(|x| TrainTrips(x.to_vec()).get_duration())
            .collect::<Vec<chrono::Duration>>()
        );*/
        println!("{:?}", t.train_info("6568", "Piacenza".to_string()).unwrap());
    }

    /*#[test]
    fn test_bastardissimo(){
        let class = std::env::var("CLASS_BASTARDA").unwrap().parse::<usize>().unwrap();
        let t = Trenitalia::new();
        let station_list_tsv = include_str!("../stazioni_coord.tsv");
        let station_list = station_list_tsv.split("\n").collect::<Vec<&str>>();
        let mapped_stations: Vec<super::TrainStation> = station_list.iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>().iter()
            .map(|x|  super::TrainStation{id: String::from(x[1]), aliases: vec![String::from(x[0])], position: Coord{
                lat: x[3].parse::<f64>().unwrap(),
                lon: x[4].parse::<f64>().unwrap()
            }, region_id: x[2].parse::<u8>().unwrap()}).collect();
        let start = mapped_stations.len()as f64*(class-1)as f64/100.0;
        let end = mapped_stations.len()as f64*class as f64/100.0;
        for i in start as usize..end as usize {
            let from = &mapped_stations[i];
            for to in &mapped_stations {
                if from.id == to.id {
                    continue;
                }
                println!("Trip from {} to {}", from.get_name(), to.get_name());
                let res = t.find_trips(from, to, &chrono::Local::now());
                drop(res);
            }
        }
    }*/
}
