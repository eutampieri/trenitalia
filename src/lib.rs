use chrono::prelude::*;

mod mapping;

static WORDS_EQUALITY_THRESHOLD: f64 = 0.70;

pub struct TrainTrips(Vec<TrainTrip>);

mod utils {
    pub fn match_strings(first: &str, second: &str) -> f64 {
        if first.to_lowercase() == second.to_lowercase() {
            1.0
        } else {
            strsim::normalized_damerau_levenshtein(&first.to_lowercase(), &second.to_lowercase())
        }
    }
}

impl TrainTrips{
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.0[0].departure.1).clone();
        let arrivo = (&self.0[&self.0.len() - 1].arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
}

// TODO Aggiungere tipi treno
#[derive(Debug, Clone, Copy)]
pub enum TrainType{
    Regionale,
    RegionaleVeloce,
    InterCity,
    FrecciaRossa,
    FrecciaArgento,
    FrecciaBianca,
    InterCityNotte,
    EuroNight,
    EuroCity,
    Bus,
    //EuroCityÖBBDB,
    Unknown,
}

#[derive(Debug)]
pub struct TrainTripStop {
    pub station: TrainStation,
    pub platform: String,
    pub arrival: Option<chrono::DateTime<chrono::Local>>,
    pub departure: Option<chrono::DateTime<chrono::Local>>,
    pub expected_arrival: Option<chrono::DateTime<chrono::Local>>,
    pub expected_departure: Option<chrono::DateTime<chrono::Local>>,
}

pub struct DetailedTrainTrip {
    pub from: TrainStation,
    pub to: TrainStation, 
    pub train_number: String,
    pub train_type: TrainType,
    pub stops: Vec<TrainTripStop>,
}

#[derive(Debug, Clone)]
pub struct TrainTrip {
    pub train_number: String,
    pub train_type: TrainType,
    pub arrival: (TrainStation, chrono::DateTime<chrono::Local>),
    pub departure: (TrainStation, chrono::DateTime<chrono::Local>),
}

impl TrainTrip{
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.departure.1).clone();
        let arrivo = (&self.arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
    pub fn from(reference: &TrainTrip) -> Self {
        TrainTrip{
            train_number: String::from(reference.train_number.as_str()),
            train_type: reference.train_type,
            arrival: (TrainStation::from(&reference.arrival.0), reference.arrival.1),
            departure: (TrainStation::from(&reference.departure.0), reference.departure.1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainStation {
    pub id: String,
    pub region_id: u8,
    pub position: (f64, f64),
    pub aliases: Vec<String>,
    pub vt_id: Option<String>,
    pub lefrecce_name: Option<String>,
}

impl TrainStation {
    pub fn from(reference: &Self) -> Self {
        let lf_n = match &reference.lefrecce_name {
            Some(x) => Some(String::from(x)),
            None => None
        };
        let vt_id = match &reference.vt_id {
            Some(x) => Some(String::from(x)),
            None => None
        };
        TrainStation{
            aliases: reference.aliases.iter().map(|x| String::from(x)).collect(),
            id: String::from(reference.id.as_str()),
            lefrecce_name: lf_n,
            vt_id: vt_id,
            region_id: reference.region_id,
            position: reference.position,
        }
    }
    fn short_id(&self) -> Option<String> {
        match &self.vt_id {
            None => None,
            Some(x) => Some(str::replace(x, "S", "").parse::<u16>().unwrap().to_string())
        }
    }
    pub fn get_name(&self) -> &str {
        &self.aliases[0]
    }
}

pub struct Trenitalia {
    stations: Vec<TrainStation>,
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
    fn match_train_type(&self, description: &str) -> TrainType{
        let train_type = match description {
            "RV" => TrainType::RegionaleVeloce,
            "Regionale" => TrainType::Regionale,
            "Frecciarossa" => TrainType::FrecciaRossa,
            "Frecciaargento" => TrainType::FrecciaArgento,
            "IC" => TrainType::InterCity,
            "Frecciabianca" => TrainType::FrecciaBianca,
            "ICN" => TrainType::InterCityNotte,
            "EN" => TrainType::EuroNight,
            "EC" => TrainType::EuroCity,
            "REG" => TrainType::Regionale,
            "Autobus" => TrainType::Bus,
            "BUS" => TrainType::Bus,
            "FR" => TrainType::FrecciaRossa,
            "FA" => TrainType::FrecciaArgento,
            "FB" => TrainType::FrecciaBianca,
            "ECB" => TrainType::EuroCity,
            _ => TrainType::Unknown,
        };
        match train_type{
            TrainType::Unknown =>{
                let url = format!("https://eutampieri.eu/tipi_treno.php?tipo={}", description.replace(" ", "%20"));
                let _ = reqwest::get(url.as_str());
            },
            _ => {}
        }
        train_type
    }
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
                        departure: (TrainStation::from(from),
                            chrono::Local.datetime_from_str(train.departuretime.as_str(), "%+").expect("Data non valida"),
                        ),
                        arrival: (TrainStation::from(to),
                            chrono::Local.datetime_from_str(train.arrivaltime.as_str(), "%+").expect("Data non valida"),
                        ),
                        train_number: String::from(train_number),
                        train_type: self.match_train_type(&acronym)
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
                            train_trips.push(TrainTrip::from(filling_train));
                        }
                        break;
                    }
                }
            }
            let mut old_to: Option<&str> = None;
            let mut old_to_stn = TrainStation::from(to);
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
                                train_trips.push(TrainTrip::from(filling_train));
                            }
                            break;
                        }
                    }
                }
                old_to = Some(&to.get_name());
                old_to_stn = TrainStation::from(to);
                old_ts = chrono::Local.datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T").expect("Data non valida");
                train_trips.push(TrainTrip{
                    departure: (TrainStation::from(from),
                        chrono::Local.datetime_from_str(train_trip.orarioPartenza.as_str(), "%FT%T").expect("Data non valida"),
                    ),
                    arrival: (TrainStation::from(to),
                        chrono::Local.datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T").expect("Data non valida"),
                    ),
                    train_number: String::from(train_trip.numeroTreno.as_str()),
                    train_type: self.match_train_type(&train_trip.categoriaDescrizione)
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
                            train_trips.push(TrainTrip::from(filling_train));
                        }
                        break;
                    }
                }
            }
            result.push(train_trips);
        }
        result
    }

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

    fn train_info_raw(&self, number: &str, from: &str){}

    pub fn train_info(&self, number: &str, from: String) {
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}", number);
        let response = reqwest::get(&url).unwrap().text().unwrap();
        let body: Vec<Vec<&str>> = response.trim_end_matches('\n')
        .split("\n").collect::<Vec<&str>>().iter()
        .map(|&x| x.split("|").collect::<Vec<&str>>()).collect();
        let train_station_of_origination: &str = match body.len() {
            1 => body[0][1].split('-').collect::<Vec<&str>>()[1],
            0 => {
                unimplemented!();
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
                if min_diff == 0.0 {unimplemented!()} else {station_code}
            }
        };
        self.train_info_raw(number, train_station_of_origination)
    }
    pub fn train_info_through_station(&self, number: &str, through: &TrainStation) {

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
        let _cesena = t.nearest_station((44.133333, 12.233333));
        //println!("{:?}, {:?}", imola, calalzo);
        println!("{:?}", t.find_train_station("bologna centrale"));
        let bologna = t.find_train_station("vipiteno").unwrap();
        println!("{:?}", t.find_trips(imola, bologna, &chrono::Local::now()));/*
            .iter()
            .map(|x| TrainTrips(x.to_vec()).get_duration())
            .collect::<Vec<chrono::Duration>>()
        );*/
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
