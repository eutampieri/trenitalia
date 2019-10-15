use chrono::prelude::*;
use std::io::*;
use drs_primitives::*;

mod mapping;

// TODO Aggiungere tipi treno
#[derive(Debug)]
pub enum TrainType{
    Regionale,
    RegionaleVeloce,
}

#[derive(Debug)]
pub struct TrainTripStop {
    pub station: TrainStation,
    pub platform: String,
    pub arrival: Option<chrono::DateTime<chrono::Local>>,
    pub departure: Option<chrono::DateTime<chrono::Local>>,
    pub expected_arrival: chrono::DateTime<chrono::Local>,
    pub expected_departure: chrono::DateTime<chrono::Local>,
}

#[derive(Debug)]
pub struct TrainTrip {
    pub train_number: String,
    pub train_type: TrainType,
    pub arrival: (TrainStation, chrono::DateTime<chrono::Local>),
    pub departure: (TrainStation, chrono::DateTime<chrono::Local>),
}

#[derive(Debug)]
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
        let mut file = std::fs::File::open("stazioni_coord.tsv").unwrap();
        let mut station_list_tsv = String::new();
        file.read_to_string(&mut station_list_tsv).unwrap();
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
                    // TODO parsing tipo treno
                    train_type: TrainType::Regionale
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

    pub fn train_info(&self, number: String, from: String) {

    }
    pub fn train_info_through_station(&self, number: String, through: &TrainStation) {

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
        let bologna = t.find_train_station("bologna");
        //println!("{:?}, {:?}", imola, calalzo);
        println!("{:?}", t.find_train_station_offline("iNola"));
        println!("{:?}", t.find_trips(imola, bologna.unwrap(), &chrono::Local::now()));
    }
}
