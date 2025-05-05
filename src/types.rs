use chrono::TimeZone;
use serde::{Deserialize, Serialize};

use crate::{mapping, Trenitalia};

/// Train type and number representation
#[derive(Debug, Clone)]
pub enum TrainNumber {
    Regionale {
        number: u32,
    },
    RegionaleVeloce {
        number: u32,
    },
    InterCity {
        number: u32,
    },
    FrecciaRossa {
        number: u32,
    },
    FrecciaArgento {
        number: u32,
    },
    FrecciaBianca {
        number: u32,
    },
    InterCityNotte {
        number: u32,
    },
    EuroNight {
        number: u32,
    },
    EuroCity {
        number: u32,
    },
    Bus {
        number: u32,
    },
    //EuroCityÖBBDB(u32),
    /// Unknown train, the second field is the train type as returned from the API
    Unknown {
        number: u32,
        name: String,
    },
}
impl std::string::ToString for TrainNumber {
    fn to_string(&self) -> String {
        format!(
            "{}{}",
            match self {
                Self::Regionale { number: _ } => "R",
                Self::RegionaleVeloce { number: _ } => "RV",
                Self::InterCity { number: _ } => "IC",
                Self::FrecciaRossa { number: _ } => "ES*FR",
                Self::FrecciaArgento { number: _ } => "ES*FA",
                Self::FrecciaBianca { number: _ } => "FB",
                Self::InterCityNotte { number: _ } => "ICN",
                Self::EuroNight { number: _ } => "EN",
                Self::EuroCity { number: _ } => "EC",
                Self::Bus { number: _ } => "BUS",
                Self::Unknown { number: _, name: _ } => "?",
            },
            u32::from(self)
        )
    }
}
impl std::convert::From<&TrainNumber> for u32 {
    fn from(from: &TrainNumber) -> Self {
        *match from {
            TrainNumber::Regionale { number } => number,
            TrainNumber::RegionaleVeloce { number } => number,
            TrainNumber::InterCity { number } => number,
            TrainNumber::FrecciaRossa { number } => number,
            TrainNumber::FrecciaArgento { number } => number,
            TrainNumber::FrecciaBianca { number } => number,
            TrainNumber::InterCityNotte { number } => number,
            TrainNumber::EuroNight { number } => number,
            TrainNumber::EuroCity { number } => number,
            TrainNumber::Bus { number } => number,
            TrainNumber::Unknown { number, name: _ } => number,
        }
    }
}

/// A specific stop in a train trip
#[derive(Debug)]
pub struct TrainTripStop {
    pub station: TrainStation,
    pub platform: String,
    pub arrival: Option<chrono::DateTime<chrono::Local>>,
    pub departure: Option<chrono::DateTime<chrono::Local>>,
    pub expected_arrival: Option<chrono::DateTime<chrono::Local>>,
    pub expected_departure: Option<chrono::DateTime<chrono::Local>>,
}

/// A specific stop in a train trip
#[derive(Debug)]
pub struct DetailedTrainTripStop {
    pub station: TrainStation,
    pub platform: String,
    pub arrival: Option<chrono::DateTime<chrono::Local>>,
    pub departure: Option<chrono::DateTime<chrono::Local>>,
    pub expected_arrival: Option<chrono::DateTime<chrono::Local>>,
    pub expected_departure: Option<chrono::DateTime<chrono::Local>>,
}

/// A train trip with stops specified
pub struct DetailedTrainTrip {
    pub from: TrainStation,
    pub to: TrainStation,
    pub train_number: TrainNumber,
    pub stops: Vec<TrainTripStop>,
}

/// A train trip between two stations. Stops aren't specified
#[derive(Debug, Clone)]
pub struct TrainTrip {
    pub train_number: TrainNumber,
    /// Specify the station and time of arrival
    pub arrival: (TrainStation, chrono::DateTime<chrono::Local>),
    /// Specify the station and time of departure
    pub departure: (TrainStation, chrono::DateTime<chrono::Local>),
}

impl TrainTrip {
    /// This method returns the trip's duration
    pub fn get_duration(&self) -> chrono::Duration {
        let partenza = (&self.departure.1).clone();
        let arrivo = (&self.arrival.1).clone();
        arrivo.signed_duration_since(partenza)
    }
    /// This method returns the trip's fare
    pub fn get_fare(&self) -> Option<f64> {
        let url = format!("https://www.lefrecce.it/msite/api/solutions?origin={}&destination={}&arflag=A&adate={}&atime={}&adultno=1&childno=0&direction=A&frecce=false&onlyRegional=false",
            self.departure.0.lefrecce_name.clone().unwrap().replace(" ", "%20"),
            self.arrival.0.lefrecce_name.clone().unwrap().replace(" ", "%20"),
            self.departure.1.format("%d/%m/%Y"),
            self.departure.1.format("%H")
        );
        let answer = ureq::get(url.as_str())
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        let body: Vec<crate::lefrecce::LFSolution> = serde_json::from_str(&answer).unwrap();
        for result in body {
            if chrono::Local.timestamp_millis(result.departuretime as i64) == self.departure.1
                && chrono::Local.timestamp_millis(result.arrivaltime as i64) == self.arrival.1
            {
                return result.minprice;
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct TrainInfo {
    pub current_station: TrainStation,
    pub current_delay: i16,
    pub is_at_station: bool,
    pub stops: Vec<DetailedTrainTripStop>,
}

impl TrainInfo {
    pub fn from(vtvec: &Vec<mapping::VTDetailedTrainTripLeg>, trenitalia: &Trenitalia) -> Self {
        let mut delay: i16 = 0;
        let mut current_station: TrainStation = trenitalia
            .find_train_station(&vtvec[&vtvec.len() - 1].stazione)
            .unwrap()
            .clone();
        let mut in_station: bool = false;
        let mut stations_list: Vec<DetailedTrainTripStop> = Vec::new();
        for stop in vtvec {
            let this_station = trenitalia.find_train_station(&stop.stazione).unwrap();
            if stop.stazioneCorrente {
                current_station = this_station.clone();
                if let Some(r) = stop.fermata.partenzaReale {
                    if let Some(t) = stop.fermata.partenza_teorica {
                        delay = ((r as i64 - t as i64) / 60000i64) as i16;
                    }
                } else if let Some(t) = stop.fermata.partenza_teorica {
                    delay = (((chrono::Local::now().timestamp_millis() - t as i64) / 60000i64)
                        as i16)
                        .max(0);
                }
                if stop.fermata.arrivoReale.is_some() && stop.fermata.partenzaReale.is_none() {
                    // If the train has arrived but not departed yet
                    in_station = true;
                }
            }
            let this_stop = DetailedTrainTripStop {
                arrival: stop
                    .fermata
                    .arrivoReale
                    .map(|ts| chrono::Local.timestamp((ts / 1000) as i64, 0)),
                departure: stop
                    .fermata
                    .partenzaReale
                    .map(|ts| chrono::Local.timestamp((ts / 1000) as i64, 0)),
                expected_arrival: stop
                    .fermata
                    .arrivo_teorico
                    .map(|ts| chrono::Local.timestamp((ts / 1000) as i64, 0)),
                expected_departure: stop
                    .fermata
                    .partenza_teorica
                    .map(|ts| chrono::Local.timestamp((ts / 1000) as i64, 0)),
                platform: stop
                    .fermata
                    .binarioEffettivoPartenzaDescrizione
                    .as_ref()
                    .unwrap_or(
                        stop.fermata
                            .binarioProgrammatoPartenzaDescrizione
                            .as_ref()
                            .unwrap_or(
                                stop.fermata
                                    .binarioEffettivoArrivoDescrizione
                                    .as_ref()
                                    .unwrap_or(
                                        stop.fermata
                                            .binarioProgrammatoArrivoDescrizione
                                            .as_ref()
                                            .unwrap_or(&"?".to_string()),
                                    ),
                            ),
                    )
                    .to_string(),
                station: this_station.clone(),
            };
            stations_list.push(this_stop);
        }
        TrainInfo {
            current_delay: delay,
            is_at_station: in_station,
            current_station: current_station,
            stops: stations_list,
        }
    }
}

/// Struct that holds the train station data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainStation {
    /// Three-charachters ID
    pub id: String,
    /// Trenitalia region ID
    pub region_id: u8,
    /// Tuple that contains latitude and longitude
    pub position: (f64, f64),
    /// List of possible aliases of a station
    pub aliases: Vec<String>,
    /// Station ID used for the ViaggaTreno API
    pub vt_id: Option<String>,
    /// Station name used for the LeFrecce API
    pub lefrecce_name: Option<String>,
}

impl TrainStation {
    /// Get the short version of ViaggiaTreno ID
    pub fn short_id(&self) -> Option<String> {
        match &self.vt_id {
            None => None,
            Some(x) => Some(str::replace(x, "S", "").parse::<u16>().unwrap().to_string()),
        }
    }
    /// Get the station's name (the first alias)
    pub fn get_name(&self) -> &str {
        &self.aliases[0]
    }
}
