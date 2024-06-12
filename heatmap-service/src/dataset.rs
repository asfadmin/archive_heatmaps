use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum Dataset {
    #[serde(rename = "ALOS")]
    Alos,
    #[serde(rename = "UAVSAR")]
    Uavsar,
    #[serde(rename = "AIRSAR")]
    Airsar,
}

pub trait ToPartialString {
    fn to_partial_string(&self) -> String;
}

impl ToPartialString for Option<Dataset> {
    fn to_partial_string(&self) -> String {
        if let Some(dataset) = self {
            match dataset {
                Dataset::Alos => "ALOS PALSAR%".to_string(),
                Dataset::Uavsar => "UAVSAR%".to_string(),
                Dataset::Airsar => "AIRSAR%".to_string(),
            }
        } else {
            "%".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dataset_to_partial_string() {
        assert_eq!(Some(Dataset::Alos).to_partial_string(), "ALOS PALSAR%");
        assert_eq!(Some(Dataset::Uavsar).to_partial_string(), "UAVSAR%");
        assert_eq!(Some(Dataset::Airsar).to_partial_string(), "AIRSAR%");
        assert_eq!(None.to_partial_string(), "%");
    }
}


