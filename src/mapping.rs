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
    pub trainacronym: String,
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
    pub minprice: f64,
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
    pub servicelist: Vec<String>,
    pub gift: bool,
    pub trainidentifier: String,
    pub trainacronym: String,
    pub departurestation: String,
    pub departuretime: String,
    pub arrivalstation: String,
    pub arrivaltime: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LFSegment {
    pub trainidentifier: String,
    pub trainacronym: String,
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
