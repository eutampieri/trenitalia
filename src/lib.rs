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
    pub arrival: chrono::Local,
    pub departure: chrono::Local,
}

#[derive(Debug)]
pub struct TrainTrip {
    pub from: TrainStation,
    pub to: TrainStation, 
    pub train_number: String,
    pub train_type: TrainType,
    pub stops: Vec<TrainTripStop>,
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
                let from = self.find_train_station(train_trip.origine.as_str()).expect("Inconsistency in Trenitalia naming");
                let to = self.find_train_station(train_trip.destinazione.as_str()).expect("Inconsistency in Trenitalia naming");
                train_trips.push(TrainTrip{
                    from: TrainStation{id: String::from(from.id.as_str()), name: String::from(from.name.as_str()), position: from.position, region_id: from.region_id},
                    to: TrainStation{id: String::from(to.id.as_str()), name: String::from(to.name.as_str()), position: to.position, region_id: to.region_id},
                    // TODO Aggiungere fermate
                    stops: vec![],
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
        let mut min_diff = std::f64::MAX;
        let mut found_station = &self.stations[0];
        for station in &self.stations {
            let diff = diff::chars(station.name.to_lowercase().as_str(), &name.to_lowercase());
            let mut eq = 0;
            for d in diff {
                if let diff::Result::Both(_, _) = d {
                    eq = eq+1;
                }
            }
            let current_diff = name.len().max(station.name.len()) as f64 / eq as f64;
            if current_diff < min_diff {
                min_diff = current_diff;
                found_station = station;
            }
        }
        //println!("{:?}, {}", found_station, min_diff);
        if min_diff < 1.5 {Some(found_station)} else {None}
    }

    pub fn train_info(&self, number: String, from: String) {

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
        //println!("{:?}, {:?}", imola, calalzo);
        println!("{:?}", t.find_train_station("iNola"));
        println!("{:?}", t.find_trips(imola, calalzo, &chrono::Local::now()));
    }
}
