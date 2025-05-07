use heatmap_api::{HeatmapData, OutlineResponse};

// Send a request to the service for data based on the filter
pub async fn request(filter: heatmap_api::Filter) -> (HeatmapData, OutlineResponse) {
    let client = reqwest::Client::new();

    // Get the granule data from the service
    let heatmap_data: HeatmapData = bincode::decode_from_slice(
        &client
            .post("http://localhost:8000/heatmap") // TODO, some configuration mechanism for this
            .json(&heatmap_api::HeatmapQuery { filter })
            .send()
            .await
            .expect("ERROR: Failed to recive data from post request")
            .bytes()
            .await
            .expect("ERROR: Failed to convert response into Bytes"),
        bincode::config::standard(),
    )
    .expect("ERROR: Failed to deserialized json data")
    .0;

    // Get the outline data from the service
    // *** This should be broken out into its own function so we only get and mesh the world outline once ***
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
    (heatmap_data, outline_data)
}
