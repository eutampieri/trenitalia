use std::io::*;
use drs_primitives::*;

mod mapping;

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
    pub fn find_journey(&self, from: &TrainStation, to: &TrainStation, when: &chrono::DateTime<chrono::Local>){
        let url = format!("http://www.viaggiatreno.it/viaggiatrenonew/resteasy/viaggiatreno/soluzioniViaggioNew/{}/{}/{}",
            from.short_id(),
            to.short_id(),
            when.format("%FT%T")
        );
        println!("{}", url);
        let body: mapping::JourneySearchResult = reqwest::get(url.as_str()).unwrap().json().unwrap();
        println!("{:?}", body);
    }

    pub fn find_train_station(&self, name: String) -> Option<&TrainStation> {
        let mut min_diff = std::f64::MAX;
        let mut found_station = &self.stations[0];
        for station in &self.stations {
            let diff = diff::chars(station.name.to_lowercase().as_str(), name.to_lowercase().as_str());
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
        println!("{:?}", t.find_train_station(String::from("iNola")));
        println!("{:?}", t.find_train_station(String::from("imolla")));
        //t.find_journey(imola, calalzo, &chrono::Local::now());
    }
}
