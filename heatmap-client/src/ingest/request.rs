pub async fn request(filter: heatmap_api::Filter) -> heatmap_api::HeatmapData {
    let client = reqwest::Client::new();
    let data = client
        .post("http://localhost:8000/query") // TODO, some configuration mechanism for this
        .json(&heatmap_api::HeatmapQuery { filter })
        .send()
        .await
        .expect("ERROR: Failed to recive data from post request");

    let str = data
        .text()
        .await
        .expect("ERROR: Failed to deserialize Response into json str");

    web_sys::console::log_2(&"Data text: ".into(), &format!("{:?}", str).into());

    let json_data: heatmap_api::HeatmapData =
        serde_json::from_str(&str).expect("ERROR: Failed to deserialized json data");

    // Deserialize the json into a HeatmapData struct
    web_sys::console::log_1(&"Data succesfully deserialized".into());
    json_data
}
