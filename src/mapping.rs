#![allow(non_snake_case)]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VTJourneySearchResult {
    pub soluzioni: Vec<VTTrainSolution>,
    pub origine: String,
    pub destinazione: String,
    pub errore: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VTTrainSolution {
    pub durata: Option<String>,
    pub vehicles: Vec<VTTrainTripLeg>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VTTrainTripLeg {
    pub origine: Option<String>,
    pub destinazione: Option<String>,
    pub orarioPartenza: String,
    pub orarioArrivo: String,
    pub categoria: Option<String>,
    pub categoriaDescrizione: String,
    pub numeroTreno: String,
}

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
pub struct LFOffer {
    pub name: String,
    pub extraInfo: Vec<String>,
    pub points: f64,
    pub price: f64,
    pub message: String,
    pub offeridlist: Vec<LFOfferID>,
    pub credentials: Option<String>,
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
