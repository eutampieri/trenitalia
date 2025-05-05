#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

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
pub struct VTDetailedTrainTripStop {
    pub orientamento: Option<Vec<String>>,
    pub kcNumTreno: Option<String>,
    pub stazione: String,
    pub id: String,
    pub listaCorrispondenza: Option<Vec<String>>,
    //#[serde(alias = "programmata", alias = "partenza_teorica")]
    pub programmata: Option<u64>,
    pub partenza_teorica: Option<u64>,
    pub programmataZero: Option<u64>,
    //#[serde(alias = "effettiva", alias = "arrivo_teorico")]
    pub effettiva: Option<u64>,
    pub arrivo_teorico: Option<u64>,
    pub ritardo: i16,
    pub partenzaTeoricaZero: Option<u64>,
    pub arrivoTeoricoZero: Option<u64>,
    //#[serde(alias = "nextChanged")]
    pub isNextChanged: bool,
    pub nextChanged: bool,
    pub partenzaReale: Option<u64>,
    pub arrivoReale: Option<u64>,
    //#[serde(alias = "ritardo")]
    pub ritardoPartenza: i16,
    //#[serde(alias = "ritardo")]
    pub ritardoArrivo: i16,
    pub progressivo: i32,
    pub binarioEffettivoArrivoCodice: Option<String>,
    pub binarioEffettivoArrivoTipo: Option<String>,
    pub binarioEffettivoArrivoDescrizione: Option<String>,
    pub binarioProgrammatoArrivoCodice: Option<String>,
    pub binarioProgrammatoArrivoDescrizione: Option<String>,
    pub binarioEffettivoPartenzaCodice: Option<String>,
    pub binarioEffettivoPartenzaTipo: Option<String>,
    pub binarioEffettivoPartenzaDescrizione: Option<String>,
    pub binarioProgrammatoPartenzaCodice: Option<String>,
    pub binarioProgrammatoPartenzaDescrizione: Option<String>,
    pub tipoFermata: char,
    pub visualizzaPrevista: bool,
    pub nextTrattaType: i8,
    pub actualFermataType: i8,
    pub materiale_label: Option<String>,
    //#[serde(flatten, rename(serialize = "altro"))]
    //pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VTDetailedTrainTripLeg {
    pub last: bool,
    pub stazioneCorrente: bool,
    pub id: String,
    pub stazione: String,
    pub fermata: VTDetailedTrainTripStop,
    pub partenzaReale: bool,
    pub arrivoReale: bool,
    pub first: bool,
    pub orientamento: Vec<String>,
    pub nextTrattaType: Option<i8>,
    pub actualFermataType: Option<i8>,
    pub previousTrattaType: Option<i8>,
    pub trattaType: i8,
}
