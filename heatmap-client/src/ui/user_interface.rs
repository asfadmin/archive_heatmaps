extern crate heatmap_api;

use chrono::naive::NaiveDate;
use leptos::wasm_bindgen::JsCast;
use leptos::*;

stylance::import_crate_style!(style, "src/ui/user_interface.module.scss");

#[component]
pub fn UserInterface(set_filter: WriteSignal<heatmap_api::Filter>) -> impl IntoView {
    let base_date = NaiveDate::from_ymd_opt(1, 1, 1).expect("Failed to create Naive");

    let min_date: i64 = NaiveDate::from_ymd_opt(2021, 1, 2)
        .expect("Failed to parse left hand side when finding min_date")
        .signed_duration_since(base_date)
        .num_days();

    let max_date: i64 = NaiveDate::from_ymd_opt(2021, 2, 2)
        .expect("Failed to parse left hand side when finding max_date")
        .signed_duration_since(base_date)
        .num_days();

    let (start_date, set_start_date) = create_signal(min_date);
    let (end_date, set_end_date) = create_signal(max_date);

    let doc = document();

    // Might be worth reworking this, we are mixing web_sys and leptos here but weve done the same elsewhere so we could also just roll with it
    // This closure is run when the submit button is pressed, it formats a filter string and sets a signal
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // Stop page from reloading
        ev.prevent_default();

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
        let start_date = NaiveDate::from_num_days_from_ce_opt(start_date() as i32)
            .expect("Failed to parse start date")
            .format("%Y-%m-%d")
            .to_string();
        let end_date = NaiveDate::from_num_days_from_ce_opt(end_date() as i32)
            .expect("Failed to parse end date")
            .format("%Y-%m-%d")
            .to_string();

        set_filter(heatmap_api::Filter {
            product_type,
            platform_type,
            start_date,
            end_date,
        })
    };

    view! {
        <div class=style::user_interface>
            <form on:submit=on_submit>
                <div class=style::data_types>
                    <input type="checkbox" id="grd" name="granule_type" value=0/>
                    <label class=style::radio_text for="grd">
                        "GRD"
                    </label>
                    <br/>
                    <input type="checkbox" id="slc" name="granule_type" value=1/>
                    <label class=style::radio_text for="slc">
                        "SLC"
                    </label>
                    <br/>
                    <input type="checkbox" id="ocn" name="granule_type" value=2/>
                    <label class=style::radio_text for="ocn">
                        "OCN"
                    </label>
                </div>

                <div class=style::sat_selection_div>
                    <input type="checkbox" id="s1-a" name="sat_selection" value=0/>
                    <label class=style::radio_text for="s1-a">
                        "S1A"
                    </label>
                    <br/>
                    <input type="checkbox" id="s1-b" name="sat_selection" value=1/>
                    <label class=style::radio_text for="s1-b">
                        "S1B"
                    </label>
                    <br/>
                </div>

                <div>
                    <input
                        type="range"
                        prop:value=start_date
                        on:change=move |ev| {
                            let val = event_target_value(&ev)
                                .parse::<i64>()
                                .expect("Failed to parse an i64 from event value start slider");
                            if val > end_date() {
                                set_start_date(end_date())
                            } else {
                                set_start_date(val)
                            }
                        }

                        min=min_date
                        max=max_date
                    />
                    <input
                        type="range"
                        prop:value=end_date
                        on:change=move |ev| {
                            let val = event_target_value(&ev)
                                .parse::<i64>()
                                .expect("Failed to parse an i64 from event value in end_slider");
                            if val < start_date() {
                                set_end_date(start_date())
                            } else {
                                set_end_date(val)
                            }
                        }

                        min=min_date
                        max=max_date
                    />
                    <p class=style::radio_text>"Start Date: " {start_date}</p>
                    <p class=style::radio_text>"End Date: " {end_date}</p>
                </div>

                <input class=style::submit_button type="submit" value="Submit"/>
            </form>
        </div>
    }
}
