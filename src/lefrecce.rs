use chrono::TimeZone;
use serde::{Deserialize, Serialize};

use crate::types::{TrainStation, TrainTrip};

#[derive(Serialize, Deserialize, Debug)]
pub struct LFTrain {
    pub trainidentifier: String,
    pub trainacronym: Option<String>,
    pub traintype: char,
    pub pricetype: char,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFSolution {
    pub idsolution: String,
    pub origin: String,
    pub destination: String,
    pub direction: String,
    pub departuretime: u64,
    pub arrivaltime: u64,
    pub minprice: Option<f64>,
    pub optionaltext: Option<String>,
    pub duration: String,
    pub changesno: u8,
    pub bookable: bool,
    pub saleable: bool,
    pub trainlist: Vec<LFTrain>,
    pub onlycustom: bool,
    pub extraInfo: Vec<String>,
    pub showSeat: bool,
    pub specialOffer: Option<f64>,
    pub transportMeasureList: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFLeg {
    pub idleg: String,
    pub bookingtype: char,
    pub segments: Vec<LFSegment>,
    pub servicelist: Vec<LFService>,
    pub gift: bool,
    pub trainidentifier: String,
    pub trainacronym: String,
    pub departurestation: String,
    pub departuretime: String,
    pub arrivalstation: String,
    pub arrivaltime: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct LFCredential {
    pub credentialid: u16,
    pub format: u8,
    pub name: String,
    pub description: String,
    pub possiblevalues: String,
    pub typeCredential: char,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFOffer {
    pub name: String,
    pub extraInfo: Vec<String>,
    pub points: f64,
    pub price: f64,
    pub message: String,
    pub offeridlist: Vec<LFOfferID>,
    //pub credentials: Option<Vec<LFCredential>>,
    pub available: i64,
    pub visible: bool,
    pub selected: bool,
    pub specialOffers: Vec<LFOffer>,
    //pub standingPlace: bool,
    pub seatToPay: bool,
    pub disableSeatmapSelection: bool,
    pub transportMeasure: Option<String>,
    pub saleable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFOfferID {
    pub xmlid: String,
    pub price: f64,
    pub eligible: Option<String>,
    pub messages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFSubService {
    pub name: String,
    pub offerlist: Vec<LFOffer>,
    pub hasGift: bool,
    pub minprice: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFService {
    pub name: String,
    pub offerlist: Option<Vec<LFOffer>>,
    pub subservicelist: Option<Vec<LFSubService>>,
    pub hasGift: bool,
    pub minprice: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFSegment {
    pub trainidentifier: String,
    pub trainacronym: Option<String>,
    pub departurestation: String,
    pub departuretime: String,
    pub arrivalstation: String,
    pub arrivaltime: String,
    pub nodexmlid: String,
    pub showseatmap: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFDetailedSolution {
    pub idsolution: String,
    pub leglist: Vec<LFLeg>,
    pub extraInfo: Vec<String>,
}

pub fn find_trips(
    from: &TrainStation,
    to: &TrainStation,
    when: &chrono::DateTime<chrono::Local>,
) -> Vec<Vec<TrainTrip>> {
    if from.id == to.id || from.lefrecce_name.is_none() || from.lefrecce_name.is_none() {
        return vec![];
    }
    let mut result: Vec<Vec<TrainTrip>> = Vec::new();
    let client = ureq::agent();
    let url = format!("https://www.lefrecce.it/msite/api/solutions?origin={}&destination={}&arflag=A&adate={}&atime={}&adultno=1&childno=0&direction=A&frecce=false&onlyRegional=false",
        from.lefrecce_name.clone().unwrap().replace(" ", "%20"),
        to.lefrecce_name.clone().unwrap().replace(" ", "%20"),
        when.format("%d/%m/%Y"),
        when.format("%H")
    );
    if cfg!(debug_assertions) {
        println!("{}", url);
    }
    let body: Vec<LFSolution> = serde_json::from_value(
        client
            .get(url.as_str())
            .call()
            .unwrap()
            .into_json()
            .unwrap(),
    )
    .unwrap();
    for solution in &body {
        let mut train_trips: Vec<TrainTrip> = Vec::new();
        let url_details = format!(
            "https://www.lefrecce.it/msite/api/solutions/{}/standardoffers",
            solution.idsolution
        );
        if cfg!(debug_assertions) {
            println!("{}", url_details);
        }
        let body_details: LFDetailedSolution = serde_json::from_value(
            client
                .get(url_details.as_str())
                .call()
                .expect("Failed API call")
                .into_json()
                .unwrap(),
        )
        .unwrap();
        for leg in &body_details.leglist {
            for train in &leg.segments {
                if train.trainidentifier == String::from("Same") {
                    continue;
                }
                let acronym = train
                    .trainacronym
                    .as_ref()
                    .map_or(String::from(""), |x| String::from(x.as_str()));
                let train_name_exploded: Vec<&str> = train.trainidentifier.split(' ').collect();
                let train_number = train_name_exploded[&train_name_exploded.len() - 1];
                unimplemented!();
                let from = from;
                let to = to;
                train_trips.push(TrainTrip {
                    departure: (
                        from.clone(),
                        chrono::Local
                            .datetime_from_str(train.departuretime.as_str(), "%+")
                            .expect("Data non valida"),
                    ),
                    arrival: (
                        to.clone(),
                        chrono::Local
                            .datetime_from_str(train.arrivaltime.as_str(), "%+")
                            .expect("Data non valida"),
                    ),
                    train_number: crate::utils::match_train_type(
                        &acronym,
                        train_number.parse::<u32>().unwrap_or(
                            train_number
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
        }
        result.push(train_trips);
    }
    /*if cfg!(debug_assertions) {
        println!("{:?}", result);
    }*/
    result
}
