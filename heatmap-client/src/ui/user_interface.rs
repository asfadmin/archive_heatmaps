extern crate heatmap_api;

use chrono::naive::NaiveDate;
use leptos::wasm_bindgen::JsCast;
use leptos::*;
use stylers::style_str;

#[component]
pub fn UserInterface(set_filter: WriteSignal<heatmap_api::Filter>) -> impl IntoView {
    let min_date = NaiveDate::from_ymd_opt(2021, 1, 2)
        .expect("Failed to parse left hand side when finding min_date")
        .format("%Y-%m-%d")
        .to_string();

    let max_date = NaiveDate::from_ymd_opt(2021, 2, 2)
        .expect("Failed to parse left hand side when finding max_date")
        .format("%Y-%m-%d")
        .to_string();

    let (start_date, _set_start_date) = create_signal(min_date);
    let start_date_element: NodeRef<html::Input> = create_node_ref();
    let (end_date, _set_end_date) = create_signal(max_date);
    let end_date_element: NodeRef<html::Input> = create_node_ref();

    let doc = document();

    // Might be worth reworking this, we are mixing web_sys and leptos here but weve done the same elsewhere so we could also just roll with it
    // This closure is run when the submit button is pressed, it formats a filter string and sets a signal
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

        // Convert slider values into Dates
        let start_date = start_date_element()
            .expect("failed to get start date value")
            .value();
        let end_date = end_date_element()
            .expect("failed to get end_date value")
            .value();

        web_sys::console::log_1(&start_date.clone().into());

        set_filter(heatmap_api::Filter {
            product_type,
            platform_type,
            start_date,
            end_date,
        })
    };

    let (class_name, style_val) = style_str!("UserInterface",
        div.user_interface {
            position: absolute;
            z-index: 1;
            padding: "5mm";
            border-radius: "5mm";
            box-shadow: 0 3px 10px rgba(0, 0, 0, 0.2);
        }

        #form {
            display: flex;
            flex-direction: column;
            gap: "5mm";
        }

        * {
            color: white;
            text-align: left;
            font-family: "inter, sans";
            font-size: "12pt";
            background: #303030;
        }

        .datepicker {
            color: white;
            border: none;
        }

        .checkbox {
            color: white;
            border: none;
        }

        p {
            margin: "1mm";
        }

        td {
            padding-right: "1mm";
        }

        #checkboxes {
            display: flex;
            justify-content: space-around;
        }
    );

    view! { class=class_name,
        <style>{style_val}</style>
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
        </div>
    }
}
