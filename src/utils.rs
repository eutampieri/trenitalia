use crate::types::TrainNumber;

/// Module which contains utilities

/// This function returns the normalized typing distance between two strings
pub fn match_strings(first: &str, second: &str) -> f64 {
    if first.to_lowercase() == second.to_lowercase() {
        1.0
    } else {
        strsim::normalized_damerau_levenshtein(&first.to_lowercase(), &second.to_lowercase())
    }
}

macro_rules! current_timestamp_ms {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
}

/// Builds a TrainNumber enum from the train number and train type
pub fn match_train_type(description: &str, number: u32) -> TrainNumber {
    let train_type = match description {
        "RV" => TrainNumber::RegionaleVeloce { number: number },
        "Regionale" => TrainNumber::Regionale { number: number },
        "Frecciarossa" => TrainNumber::FrecciaRossa { number: number },
        "Frecciaargento" => TrainNumber::FrecciaArgento { number: number },
        "IC" => TrainNumber::InterCity { number: number },
        "Frecciabianca" => TrainNumber::FrecciaBianca { number: number },
        "ICN" => TrainNumber::InterCityNotte { number: number },
        "EN" => TrainNumber::EuroNight { number: number },
        "EC" => TrainNumber::EuroCity { number: number },
        "REG" => TrainNumber::Regionale { number: number },
        "Autobus" => TrainNumber::Bus { number: number },
        "BUS" => TrainNumber::Bus { number: number },
        "FR" => TrainNumber::FrecciaRossa { number: number },
        "FA" => TrainNumber::FrecciaArgento { number: number },
        "FB" => TrainNumber::FrecciaBianca { number: number },
        "ECB" => TrainNumber::EuroCity { number: number },
        _ => TrainNumber::Unknown {
            number: number,
            name: String::from(description),
        },
    };
    train_type
}
