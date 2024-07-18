use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineResponse {
    pub data: OutlineData,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapData {
    pub data: InteriorData,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InteriorData {
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
    pub dataset: Dataset,
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

pub async fn request() -> (HeatmapData, OutlineResponse) {
    let client = reqwest::Client::new();
    let data = client
        .post("http://localhost:8000/heatmap") // TODO, some configuration mechanism for this
        .json(&HeatmapQuery {
            dataset: Dataset::Alos,
        })
        .send()
        .await
        .expect("ERROR: Failed to recive data from post request");

    let str = data
        .text()
        .await
        .expect("ERROR: Failed to deserialize Response into json str");

    web_sys::console::log_2(&"Data text: ".into(), &format!("{:?}", str).into());

    let json_data: HeatmapData =
        serde_json::from_str(&str).expect("ERROR: Failed to deserialized json data");

    let outline_data: OutlineResponse = serde_json::from_str(
        &client
            .get("http://localhost:8000/outline")
            .send()
            .await
            .expect("Failed to recieve outline data from post request")
            .text()
            .await
            .expect("failed to convert outline data to text"),
    )
    .expect("failed to deserialize json data");

    // Deserialize the json into a HeatmapData struct
    web_sys::console::log_1(&"Data succesfully deserialized".into());
    (json_data, outline_data)
}
