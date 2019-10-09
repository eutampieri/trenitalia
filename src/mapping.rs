use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JourneySearchResult {
    pub soluzioni: Vec<TrainSolution>,
    pub origine: String,
    pub destinazione: String,
    pub errore: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrainSolution {
    pub durata: Option<String>,
    pub vehicles: Vec<TrainTripLeg>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrainTripLeg {
    pub origine: String,
    pub destinazione: String,
    pub orarioPartenza: String,
    pub orarioArrivo: String,
    pub categoria: String,
    pub categoriaDescrizione: String,
    pub numeroTreno: String,
}
