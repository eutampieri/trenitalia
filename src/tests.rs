use super::*;

#[test]
fn existing_station_can_be_found() {
    let t = Trenitalia::new();
    assert!(t.find_train_station("bolzano").is_some());
}

#[test]
fn can_find_trips_between_two_stations_on_same_line() {
    let t = Trenitalia::new();
    let bologna = t.find_train_station("bologna centrale").unwrap();
    let cesena = t.nearest_station((44.133333, 12.233333));
    let trips = t.find_trips(bologna, cesena, &chrono::Local::now());
    assert!(!trips.is_empty());
}
