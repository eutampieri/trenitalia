use chrono::prelude::*;
use std::io::*;
use drs_primitives::*;

mod mapping;

struct TrainTrips(Vec<TrainTrip>);

impl TrainTrips{
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.0[0].departure.1).clone();
        let arrivo = (&self.0[&self.0.len() - 1].arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
}

// TODO Aggiungere tipi treno
#[derive(Debug, Clone)]
pub enum TrainType{
    Regionale,
    RegionaleVeloce,
    InterCity,
    FrecciaRossa,
    FrecciaArgento,
    FrecciaBianca,
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
}

#[derive(Debug, Clone)]
pub struct TrainStation {
    pub name: String,
    pub id: String,
    pub region_id: u8,
    pub position: Coord
}

impl TrainStation {
    fn short_id(&self) -> String {
        str::replace(&self.id, "S", "").parse::<u16>().unwrap().to_string()
    }
}

pub struct Trenitalia {
    stations: Vec<TrainStation>
}

impl Trenitalia {
    /// Creates a new Trenitalia instance
    pub fn new() -> Trenitalia {
        let station_list_tsv = include_str!("../stazioni_coord.tsv");
        let mut station_list = station_list_tsv.split("\n").collect::<Vec<&str>>();
        station_list.remove(0);
        station_list.remove(&station_list.len()-1);
        let mapped_stations: Vec<TrainStation> = station_list.iter()
            .map(|&x| x.split("\t").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>().iter()
            .map(|x|  TrainStation{id: String::from(x[1]), name: String::from(x[0]), position: Coord{
                lat: x[3].parse::<f64>().unwrap(),
                lon: x[4].parse::<f64>().unwrap()
            }, region_id: x[2].parse::<u8>().unwrap()}).collect();
        Trenitalia{stations: mapped_stations}
    }
    fn match_train_type(&self, description: &str) -> TrainType{
        let train_type = match description {
            "RV" => TrainType::RegionaleVeloce,
            "Regionale" => TrainType::Regionale,
            "Frecciarossa" => TrainType::FrecciaRossa,
            "Frecciaargento" => TrainType::FrecciaArgento,
            "IC" => TrainType::InterCity,
            "Frecciabianca" => TrainType::FrecciaBianca,
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
    pub fn find_trips(&self, from: &TrainStation, to: &TrainStation, when: &chrono::DateTime<chrono::Local>) -> Vec<Vec<TrainTrip>>{
        let mut result: Vec<Vec<TrainTrip>> = Vec::new();
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/soluzioniViaggioNew/{}/{}/{}",
            from.short_id(),
            to.short_id(),
            when.format("%FT%T")
        );
        let body: mapping::JourneySearchResult = reqwest::get(url.as_str()).unwrap().json().unwrap();
        for soluzione in body.soluzioni {
            let mut train_trips: Vec<TrainTrip> = Vec::new();
            if strsim::normalized_damerau_levenshtein(&soluzione.vehicles[0].origine.to_lowercase(), &from.name.to_lowercase()) < 0.1 {
                continue;
            }
            for train_trip in soluzione.vehicles {
                let from = self.find_train_station_offline(train_trip.origine.as_str())
                    .unwrap_or_else(|| self.find_train_station_offline(train_trip.origine.as_str()).expect("Inconsistency in Trenitalia"));
                let to = self.find_train_station_offline(train_trip.destinazione.as_str())
                    .unwrap_or_else(|| self.find_train_station_offline(train_trip.destinazione.as_str()).expect("Inconsistency in Trenitalia"));
                train_trips.push(TrainTrip{
                    departure: (TrainStation{
                            id: String::from(&from.id),
                            name: String::from(&from.name),
                            position: from.position,
                            region_id: from.region_id
                        },
                        chrono::Local.datetime_from_str(train_trip.orarioPartenza.as_str(), "%FT%T").expect("Data non valida"),
                    ),
                    arrival: (TrainStation{
                            id: String::from(&to.id),
                            name: String::from(&to.name),
                            position: to.position,
                            region_id: to.region_id
                        },
                        chrono::Local.datetime_from_str(train_trip.orarioArrivo.as_str(), "%FT%T").expect("Data non valida"),
                    ),
                    train_number: train_trip.numeroTreno,
                    train_type: self.match_train_type(&train_trip.categoriaDescrizione)
                });
            }
            result.push(train_trips);
        }
        result
    }

    pub fn find_train_station(&self, name: &str) -> Option<&TrainStation> {
        //return Some(&self.stations[0]);
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/autocompletaStazione/{}", name);
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
                if station.id == body[0][1] {
                    return Some(station);
                }
            }
            None
        }
    }

    pub fn find_train_station_offline(&self, name: &str) -> Option<&TrainStation> {
        let mut min_diff = 0.0;
        let mut found_station = &self.stations[0];
        for station in &self.stations {
            if station.name == name {
                return Some(station);
            }
        }
        for station in &self.stations {
            let diff = strsim::normalized_damerau_levenshtein(&station.name.to_lowercase(), &name.to_lowercase());
            if diff > min_diff {
                min_diff = diff;
                found_station = station;
            }
        }
        if min_diff > 0.65 {Some(found_station)} else {None}
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
                    let diff = strsim::normalized_damerau_levenshtein(
                        &option[0].split('-').collect::<Vec<&str>>()[1].trim_start().to_lowercase(),
                        &from.to_lowercase()
                    );
                    if diff < min_diff {
                        min_diff = diff;
                        station_code = option[1].split('-').collect::<Vec<&str>>()[1];
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
        let coord = Coord{lat: point.0, lon: point.1};
        for station in &self.stations {
            if station.position.distance(&coord) < min_dist {
                sta = station;
                min_dist = station.position.distance(&coord);
            }
        }
        sta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distance_test() {
        let a = Coord{lat: 1.0, lon: 1.0};
        let b = Coord{lat: 2.0, lon: 2.0};
        assert_eq!(a.distance(&b), 157.22543203805722);
    }
    #[test]
    fn test(){
        let t = Trenitalia::new();
        let calalzo = t.nearest_station((46.45, 12.383333));
        let _carnia = t.nearest_station((46.374318, 13.134141));
        let imola = t.nearest_station((44.3533, 11.7141));
        let cesena = t.nearest_station((44.133333, 12.233333));
        let bologna = t.find_train_station("roma");
        //println!("{:?}, {:?}", imola, calalzo);
        println!("{:?}", t.find_train_station_offline("immola"));
        println!("{:?}", t.find_trips(imola, bologna.unwrap(), &chrono::Local::now()));/*
            .iter()
            .map(|x| TrainTrips(x.to_vec()).get_duration())
            .collect::<Vec<chrono::Duration>>()
        );*/
    }
}
