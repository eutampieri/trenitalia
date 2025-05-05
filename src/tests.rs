use super::*;

#[test]
fn lookup_test() {
    let t = Trenitalia::new();
    println!("{:?}", t.find_train_station("bolzano"));
    assert!(t.fast_station_lookup.get("Bolzano").is_some());
}

#[test]
fn test() {
    let t = Trenitalia::new();
    let _calalzo = t.nearest_station((46.45, 12.383333));
    let _carnia = t.nearest_station((46.374318, 13.134141));
    let imola = t.nearest_station((44.3533, 11.7141));
    let cesena = t.nearest_station((44.133333, 12.233333));
    //println!("{:?}, {:?}", imola, calalzo);
    println!("{:?}", t.find_train_station("bologna centrale"));
    let _bologna = t.find_train_station("marradi").unwrap();
    println!("{:?}", t.find_trips(imola, _bologna, &chrono::Local::now())); /*
                                                                                .iter()
                                                                                .map(|x| TrainTrips(x.to_vec()).get_duration())
                                                                                .collect::<Vec<chrono::Duration>>()
                                                                            );*/
    println!(
        "{:?}",
        t.find_trips(cesena, imola, &chrono::Local::now())[0][0].get_fare()
    );
    let a = TrainNumber::EuroCity { number: 2019 };
    a.to_string();
    println!("{:?}", t.train_info(6568, "Piacenza".to_string()).unwrap());
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
