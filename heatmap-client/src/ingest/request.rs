use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
    pub weights: Vec<u64>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

#[derive(Deserialize, Serialize)]
pub struct HeatmapQuery {
    pub dataset: Option<Dataset>,
}

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
    fn _to_partial_string(&self) -> String;
}

impl ToPartialString for Option<Dataset> {
    fn _to_partial_string(&self) -> String {
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

// Dont know if this function works or not, couldnt get the service set up to test it
pub async fn request() -> HeatmapData {
    let client = reqwest::Client::new();
    let res = client
        .post("redis://127.0.0.1:6379") // Not sure if this is the correct url to send the request to
        .json(&map)
        .send()
        .await?;

    // Deserialize the json into a HeatmapData struct
    let data: HeatmapData = serde_json::from_str(res)?;

    return data;
}
