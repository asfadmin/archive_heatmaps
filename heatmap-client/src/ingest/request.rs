use heatmap_api::{HeatmapData, OutlineResponse};

// Send a request to the service for data based on the filter
pub async fn request(filter: heatmap_api::Filter) -> (HeatmapData, OutlineResponse) {
    let client = reqwest::Client::new();

    // Send a POST request to the service with the filter as a json payload
    let data = client
        .post("http://localhost:8000/heatmap") // TODO, some configuration mechanism for this
        .json(&heatmap_api::HeatmapQuery { filter })
        .send()
        .await
        .expect("ERROR: Failed to recive data from post request");

    // Deserialize response into json string
    let str = data
        .text()
        .await
        .expect("ERROR: Failed to deserialize Response into json str");

    web_sys::console::log_2(&"Data text: ".into(), &format!("{:?}", str).into());

    // Convert json string into a HeatmapData struct
    let json_data: heatmap_api::HeatmapData =
        serde_json::from_str(&str).expect("ERROR: Failed to deserialized json data");

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
    (json_data, outline_data)
}
