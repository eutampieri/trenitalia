use super::*;

#[test]
fn existing_station_can_be_found() {
    let t = Trenitalia::new();
    assert!(t.find_train_station("bolzano").is_some());
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
