use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use types::*;

mod mapping;
#[macro_use]
mod utils;
mod lefrecce;
mod types;
mod viaggiatreno;

#[cfg(test)]
mod tests;

const WORDS_EQUALITY_THRESHOLD: f64 = 0.70;

//pub struct TrainTrips(Vec<TrainTrip>);

/*impl TrainTrips{
    /// This method calculates the total duration of a trip
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.0[0].departure.1).clone();
        let arrivo = (&self.0[&self.0.len() - 1].arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
}*/
// TODO Aggiungere tipi treno

pub struct Trenitalia {
    stations: Vec<TrainStation>,
    /// Hash map that matches aliases to indexes of the `stations` vector
    fast_station_lookup: std::collections::HashMap<String, usize>,
}

impl Trenitalia {
    /// Creates a new Trenitalia instance
    pub fn new() -> Trenitalia {
        let id_to_lf_tsv = include_str!("../id_lf_map.tsv");
        let id_to_lf: std::collections::HashMap<String, String> = id_to_lf_tsv
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
            .iter()
            .map(|x| (String::from(*&x[0]), String::from(*&x[1])))
            .collect::<Vec<(String, String)>>()
            .into_iter()
            .collect();
        let id_to_vt_tsv = include_str!("../id_vt.tsv");
        let id_to_vt: std::collections::HashMap<String, String> = id_to_vt_tsv
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
            .iter()
            .map(|x| (String::from(*&x[0]), String::from(*&x[1])))
            .collect::<Vec<(String, String)>>()
            .into_iter()
            .collect();

        let aliases_tsv = include_str!("../aliases.tsv");
        let aliases: Vec<Vec<&str>> = aliases_tsv
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>();

        let station_list_tsv = include_str!("../stations.tsv");
        let station_list = station_list_tsv.split("\n").collect::<Vec<&str>>();
        let mapped_stations: Vec<TrainStation> = station_list
            .iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>()
            .iter()
            .map(|x| {
                let mut a = vec![String::from(x[0])];
                let mut v: Vec<String> = Vec::new();
                for alias in &aliases {
                    if &alias[1] == &x[1] {
                        v.push(String::from(alias[0]))
                    }
                }
                a.append(&mut v);
                TrainStation {
                    id: String::from(x[1]),
                    aliases: a,
                    position: (x[3].parse::<f64>().unwrap(), x[4].parse::<f64>().unwrap()),
                    region_id: x[2].parse::<u8>().unwrap(),
                    lefrecce_name: id_to_lf.get(x[1]).map(|x| String::from(x)),
                    vt_id: id_to_vt.get(x[1]).map(|x| String::from(x)),
                }
            })
            .collect();
        let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for i in 0..mapped_stations.len() {
            for alias in &mapped_stations[i].aliases {
                lookup.insert(String::from(alias), i);
            }
            if mapped_stations[i].lefrecce_name.is_some() {
                lookup.insert(
                    String::from(
                        mapped_stations[i]
                            .lefrecce_name
                            .as_ref()
                            .unwrap_or(&"".to_string()),
                    )
                    .to_uppercase(),
                    i,
                );
            }
        }
        Trenitalia {
            stations: mapped_stations,
            fast_station_lookup: lookup,
        }
    }
    /// Find a trip between two stations using LeFrecce API
    /// Find a trip between two stations using ViaggiaTreno API and falling back to LeFrecce
    pub fn find_trips(
        &self,
        from: &TrainStation,
        to: &TrainStation,
        when: &chrono::DateTime<chrono::Local>,
    ) -> Vec<Vec<TrainTrip>> {
        let mut result: Vec<Vec<TrainTrip>> = Vec::new();
        let url = format!("http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/soluzioniViaggioNew/{}/{}/{}",
            from.short_id().unwrap(),
            to.short_id().unwrap(),
            when.format("%FT%T")
        );
        if cfg!(debug_assertions) {
            println!("{}", url);
        }
        let body: mapping::VTJourneySearchResult = serde_json::from_value(
            ureq::get(url.as_str())
                .call()
                .expect("Failed API call")
                .into_json()
                .unwrap(),
        )
        .unwrap();
        if body.soluzioni.len() == 0 {
            return lefrecce::find_trips(from, to, when);
        }
        for soluzione in body.soluzioni {
            let mut train_trips: Vec<TrainTrip> = Vec::new();
            if cfg!(debug_assertions) {
                println!(
                    "expected: {}, found: {}, delta: {}",
                    &from.get_name(),
                    &soluzione.vehicles[0]
                        .origine
                        .as_ref()
                        .unwrap_or(&String::from("")),
                    utils::match_strings(
                        &soluzione.vehicles[0]
                            .origine
                            .as_ref()
                            .unwrap_or(&String::from(""))
                            .to_lowercase(),
                        &from.get_name()
                    )
                );
            }
            if utils::match_strings(
                &soluzione.vehicles[0]
                    .origine
                    .as_ref()
                    .unwrap_or(&String::from("")),
                &from.get_name(),
            ) < WORDS_EQUALITY_THRESHOLD
            {
                let filling_to = &self.stations[*self
                    .fast_station_lookup
                    .get(
                        soluzione.vehicles[0]
                            .origine
                            .as_ref()
                            .unwrap_or(&String::from("")),
                    )
                    .expect("Inconsistency in Trenitalia")];
                if cfg!(debug_assertions) {
                    println!("filling_to = {:?}", filling_to);
                }
                let filling_solutions = lefrecce::find_trips(from, filling_to, when);
                for filling_solution in filling_solutions.iter() {
                    if filling_solution[0].departure.1
                        >= chrono::Local.timestamp(when.timestamp(), 0)
                        && filling_solution[&filling_solution.len() - 1].arrival.1
                            <= chrono::Local
                                .datetime_from_str(
                                    soluzione.vehicles[0].orarioPartenza.as_str(),
                                    "%FT%T",
                                )
                                .expect("Data non valida")
                    {
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
                let from = &self.stations[*self
                    .fast_station_lookup
                    .get(train_trip.origine.as_ref().unwrap_or(&String::from("")))
                    .expect("Inconsistency in Trenitalia")];
                let to = &self.stations[*self
                    .fast_station_lookup
                    .get(
                        train_trip
                            .destinazione
                            .as_ref()
                            .unwrap_or(&String::from("")),
                    )
                    .expect("Inconsistency in Trenitalia")];
                if old_to.is_some() && old_to != Some(&from.get_name()) {
                    let filling_solutions = lefrecce::find_trips(&old_to_stn, from, &old_ts);
                    for filling_solution in filling_solutions.iter() {
                        if filling_solution[0].departure.1 >= old_ts
                            && filling_solution[&filling_solution.len() - 1].arrival.1
                                <= chrono::Local
                                    .datetime_from_str(train_trip.orarioPartenza.as_str(), "%FT%T")
                                    .expect("Data non valida")
                        {
                            for filling_train in filling_solution {
                                train_trips.push(filling_train.clone());
                            }
                            break;
                        }
                    }
                }
                old_to = Some(&to.get_name());
                old_to_stn = to.clone();
                old_ts = chrono::Local
                    .datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T")
                    .expect("Data non valida");
                train_trips.push(TrainTrip {
                    departure: (
                        from.clone(),
                        chrono::Local
                            .datetime_from_str(train_trip.orarioPartenza.as_str(), "%FT%T")
                            .expect("Data non valida"),
                    ),
                    arrival: (
                        to.clone(),
                        chrono::Local
                            .datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T")
                            .expect("Data non valida"),
                    ),
                    train_number: utils::match_train_type(
                        &train_trip.categoriaDescrizione,
                        train_trip.numeroTreno.parse::<u32>().unwrap_or(
                            train_trip
                                .numeroTreno
                                .chars()
                                .into_iter()
                                .map(|x| if x.is_digit(10) { x } else { '0' })
                                .collect::<String>()
                                .parse::<u32>()
                                .unwrap(),
                        ),
                    ),
                });
            }
            if cfg!(debug_assertions) {
                println!(
                    "expected: {}, found: {}, delta: {}",
                    &to.get_name(),
                    &soluzione.vehicles[&soluzione.vehicles.len() - 1]
                        .destinazione
                        .as_ref()
                        .unwrap_or(&String::from("")),
                    utils::match_strings(
                        &soluzione.vehicles[&soluzione.vehicles.len() - 1]
                            .destinazione
                            .as_ref()
                            .unwrap_or(&String::from("")),
                        &to.get_name(),
                    )
                );
            }
            if utils::match_strings(
                &soluzione.vehicles[&soluzione.vehicles.len() - 1]
                    .destinazione
                    .as_ref()
                    .unwrap_or(&String::from("")),
                &to.get_name(),
            ) < WORDS_EQUALITY_THRESHOLD
            {
                let filling_from = &self.stations[*self
                    .fast_station_lookup
                    .get(
                        soluzione.vehicles[&soluzione.vehicles.len() - 1]
                            .destinazione
                            .as_ref()
                            .unwrap_or(&String::from("")),
                    )
                    .expect("Inconsistency in Trenitalia")];
                if cfg!(debug_assertions) {
                    println!("filling_from = {:?}", filling_from);
                }
                let filling_solutions = lefrecce::find_trips(filling_from, to, when);
                for filling_solution in filling_solutions.iter() {
                    if filling_solution[0].departure.1
                        >= chrono::Local
                            .datetime_from_str(
                                soluzione.vehicles[&soluzione.vehicles.len() - 1]
                                    .orarioArrivo
                                    .as_str(),
                                "%FT%T",
                            )
                            .expect("Data non valida")
                    {
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
        let url = format!(
            "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/autocompletaStazione/{}",
            name
        );
        if cfg!(debug_assertions) {
            println!("{}", url);
        }
        let response = ureq::get(&url)
            .call()
            .expect("Failed API call")
            .into_string()
            .unwrap();
        if response.len() == 0 {
            return None;
        }
        let body: Vec<Vec<&str>> = response
            .trim_end_matches('\n')
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.split("|").collect::<Vec<&str>>())
            .collect();
        if body.len() == 0 {
            None
        } else {
            for station in &self.stations {
                let vt_id = match &station.vt_id {
                    Some(x) => Some(String::from(x)),
                    None => None,
                };

                if station.vt_id.is_none() {
                    continue;
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
                return if min_diff >= WORDS_EQUALITY_THRESHOLD {
                    Some(found_station)
                } else {
                    None
                };
            }
        };
    }

    /// Get train details from ViaggiaTreno
    fn train_info_raw(&self, number: u32, from: &str) -> TrainInfo {
        let url =
            format!(
            "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/tratteCanvas/{}/{}/{}",
            from, number, current_timestamp_ms!()
        );
        let response: Vec<mapping::VTDetailedTrainTripLeg> = serde_json::from_value(
            ureq::get(&url)
                .call()
                .expect("Failed API call")
                .into_json()
                .unwrap(),
        )
        .unwrap();
        TrainInfo::from(&response, self)
    }

    /// Get train details, provided that you know the originating station
    pub fn train_info(&self, number: u32, from: String) -> Result<TrainInfo, &str> {
        let url = format!("http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}", number);
        let response = ureq::get(&url)
            .call()
            .expect("Failed API call")
            .into_string()
            .unwrap();
        let body: Vec<Vec<&str>> = response
            .trim_end_matches('\n')
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.split("|").collect::<Vec<&str>>())
            .collect();
        let train_station_of_origination: &str = match body.len() {
            1 => body[0][1].split('-').collect::<Vec<&str>>()[1],
            0 => {
                return Err("No train found");
            }
            _ => {
                let mut station_code = "";
                let mut min_diff = 0.0;
                for option in body {
                    let diff = utils::match_strings(
                        &option[0].split('-').collect::<Vec<&str>>()[1]
                            .trim_start()
                            .to_lowercase(),
                        &from.to_lowercase(),
                    );
                    if diff < min_diff {
                        min_diff = diff;
                        station_code = option[1].split('-').collect::<Vec<&str>>()[1];
                    }
                    if diff == 1.0 {
                        break;
                    }
                }
                if min_diff == 0.0 {
                    return Err("Train not found");
                } else {
                    station_code
                }
            }
        };
        Ok(self.train_info_raw(number, train_station_of_origination))
    }

    /// Get train details, knowing that it calls at a certain station
    pub fn train_info_calling_at(
        &self,
        number: u32,
        calling_at: &TrainStation,
    ) -> Result<TrainInfo, &str> {
        let url = format!("http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}", number);
        let response = ureq::get(&url)
            .call()
            .expect("Failed API call")
            .into_string()
            .unwrap();
        let body: Vec<Vec<&str>> = response
            .trim_end_matches('\n')
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.split("|").collect::<Vec<&str>>())
            .collect();
        match body.len() {
            1 => Ok(self.train_info_raw(number, body[0][1].split('-').collect::<Vec<&str>>()[1])),
            0 => Err("Train not found"),
            _ => {
                for option in body {
                    let train_info =
                        self.train_info_raw(number, option[1].split('-').collect::<Vec<&str>>()[1]);
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
    pub fn nearest_station(&self, point: (f64, f64)) -> &TrainStation {
        let mut min_dist = std::f64::MAX;
        let mut sta = &self.stations[0];
        for station in &self.stations {
            let dist_sq =
                (station.position.0 - point.0).powf(2.0) + (station.position.1 - point.1).powf(2.0);
            if dist_sq < min_dist {
                sta = station;
                min_dist = dist_sq;
            }
        }
        sta
    }
}
