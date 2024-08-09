extern crate heatmap_api;

use chrono::naive::NaiveDate;
use heatmap_api::Filter;
use image::ImageEncoder;
use leptos::wasm_bindgen::JsCast;
use leptos::*;
use wasm_bindgen::JsValue;
use winit::platform::web;
use std::io::BufWriter;
use std::fs::File;
use std::rc::Rc;

#[component]
pub fn UserInterface(set_filter: leptos::WriteSignal<Filter>) -> impl IntoView {
    let min_date = NaiveDate::from_ymd_opt(2019, 1, 1)
        .expect("Failed to parse left hand side when finding min_date")
        .format("%Y-%m-%d")
        .to_string();

    let max_date = NaiveDate::from_ymd_opt(2024, 4, 21)
        .expect("Failed to parse left hand side when finding max_date")
        .format("%Y-%m-%d")
        .to_string();

    let (start_date, _set_start_date) = create_signal(min_date);
    let start_date_element: NodeRef<html::Input> = create_node_ref();
    let (end_date, _set_end_date) = create_signal(max_date);
    let end_date_element: NodeRef<html::Input> = create_node_ref();

    let doc = document();

    let img = use_context::<ReadSignal<Option<image::Rgba32FImage>>>()
        .expect("Failed to get img read signal in user interface");

    let (image_url, set_image_url) = create_signal("".to_owned());

    // PNG ENCODER MAY BE THE WAY
    create_effect(move |_| {

        






        web_sys::console::log_1(&"Updating <img>".into());
        if let Some(export_image) = img.get() {

            let image_png: &mut Rc<Vec<u8>> = &mut Rc::new(Vec::new());

            let image_data: Vec<u8> = image::DynamicImage::from(export_image.clone())
                .to_rgba8()
                .into_raw();

            web_sys::console::log_1(&format!("Image Data: {:?}", export_image).into());

            {
                let mut encoder = png::Encoder::new(
                    Rc::get_mut(image_png).expect("Failed to get mut of Rc"), 
                    export_image.width(), 
                    export_image.height());
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                
                let mut writer = encoder
                    .write_header()
                    .expect("Failed to get mut of Rc");
        
                writer.write_image_data(&image_data.as_slice())
                    .expect("Failed to write image data");
            }

            web_sys::console::log_1(&format!("Written image_png: {:?}", Rc::get_mut(image_png).expect("Failed to get contents of Rc")).into());

            let js_array = js_sys::Uint8Array::new_with_length(Rc::get_mut(image_png).expect("Failed to get mut or Rc").len() as u32);
            let _: Vec<_> = Rc::get_mut(image_png).expect("Failed to get mut Rc")
                .iter()
                .enumerate()
                .map(|(index, value)| {js_array.set_index(index as u32, *value)})
                .collect();

            let file = web_sys::File::new_with_u8_array_sequence(&js_array, "heatmap.png")
                .expect("Failed to create file");

            web_sys::console::log_1(&file);

            let url = web_sys::Url::create_object_url_with_blob(&file)
                    .expect("Failed to create URL for image");
            set_image_url(url);
        } else {
            web_sys::console::log_1(&"img.get() returned None".into())
        }
        web_sys::console::log_1(&"Updated <img>".into());
    });

    // Run when an element of the UI changes, updates the filter signal
    let on_update = move |_: web_sys::Event| {
        let mut product_type = Vec::new();

        // If there is a checked button in granule_type append its value to the filter_string
        if let Ok(nodes) = doc.query_selector_all("input[name=granule_type]:checked") {
            for i in 0..nodes.length() {
                let val = nodes
                    .get(i)
                    .expect("Failed to get node in on_submit")
                    .dyn_into::<web_sys::Element>()
                    .expect("Failed to cast Node to element")
                    .get_attribute("value")
                    .expect("Failed to get value attribute")
                    .parse::<u32>()
                    .expect("Failed to parse u32 from val");

                match val {
                    0 => product_type.push(heatmap_api::ProductTypes::GroundRangeDetected),
                    1 => product_type.push(heatmap_api::ProductTypes::SingleLookComplex),
                    2 => product_type.push(heatmap_api::ProductTypes::Ocean),
                    _ => (),
                }
            }
        }

        let mut platform_type = Vec::new();

        // If there is a checked button in sat_selection append its value to the filter_string
        if let Ok(nodes) = doc.query_selector_all("input[name=sat_selection]:checked") {
            for i in 0..nodes.length() {
                let val = nodes
                    .get(i)
                    .expect("Failed to get node in on_submit")
                    .dyn_into::<web_sys::Element>()
                    .expect("Failed to cast Node to element")
                    .get_attribute("value")
                    .expect("Failed to get value attribute")
                    .parse::<u32>()
                    .expect("Failed to parse u32 from val");

                match val {
                    0 => platform_type.push(heatmap_api::PlatformType::Sentinel1A),
                    1 => platform_type.push(heatmap_api::PlatformType::Sentinel1B),
                    _ => (),
                }
            }
        }

        // Gets the selected start and end dates
        let start_date = start_date_element()
            .expect("failed to get start date value")
            .value();
        let end_date = end_date_element()
            .expect("failed to get end_date value")
            .value();

        set_filter(heatmap_api::Filter {
            product_type,
            platform_type,
            start_date,
            end_date,
        })
    };

    view! {
        <div class="user_interface">
            <form id="form">
                <div id="checkboxes">
                    <div id="product_types">
                        <p>Products</p>
                        <input
                            on:input=on_update.clone()
                            class="checkbox"
                            type="checkbox"
                            id="grd"
                            name="granule_type"
                            value=0
                            checked
                        />
                        <label class="text" for="grd">
                            "GRD"
                        </label>
                        <br/>
                        <input
                            on:input=on_update.clone()
                            class="checkbox"
                            type="checkbox"
                            id="slc"
                            name="granule_type"
                            value=1
                            checked
                        />
                        <label class="text" for="slc">
                            "SLC"
                        </label>
                        <br/>
                        <input
                            on:input=on_update.clone()
                            class="checkbox"
                            type="checkbox"
                            id="ocn"
                            name="granule_type"
                            value=2
                            checked
                        />
                        <label class="text" for="ocn">
                            "OCN"
                        </label>
                    </div>

                    <div id="platform_types">
                        <p>Platforms</p>
                        <input
                            on:input=on_update.clone()
                            class="checkbox"
                            type="checkbox"
                            id="s1-a"
                            name="sat_selection"
                            value=0
                            checked
                        />
                        <label class="text" for="s1-a">
                            "S1A"
                        </label>
                        <br/>
                        <input
                            on:input=on_update.clone()
                            class="checkbox"
                            type="checkbox"
                            id="s1-b"
                            name="sat_selection"
                            value=1
                            checked
                        />
                        <label class="text" for="s1-b">
                            "S1B"
                        </label>
                        <br/>
                    </div>
                </div>

                <div id="date_range">
                    <table>
                        <tr>
                            <td>
                                <label class="text" for="start_date">
                                    Start Date
                                </label>
                            </td>
                            <td>
                                <input
                                    type="date"
                                    class="datepicker"
                                    node_ref=start_date_element
                                    prop:value=start_date
                                    on:change=on_update.clone()
                                />
                            </td>
                        </tr>
                        <tr>
                            <td>
                                <label class="text" for="end_date">
                                    End Date
                                </label>
                            </td>
                            <td>
                                <input
                                    type="date"
                                    class="datepicker"
                                    node_ref=end_date_element
                                    prop:value=end_date
                                    on:change=on_update.clone()
                                />
                            </td>
                        </tr>
                    </table>
                </div>
            </form>
            <div>
                <a
                    class="button"
                    href=move || {image_url()}
                    download="heatmap.png"
                >
                    Export to PNG
                </a>
            </div>
        </div>
    }
}
