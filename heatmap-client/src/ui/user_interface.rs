use chrono::naive::NaiveDate;
use leptos::wasm_bindgen::JsCast;
use leptos::*;

stylance::import_crate_style!(style, "src/ui/user_interface.module.scss");

#[component]
pub fn UserInterface(set_filter: WriteSignal<String>) -> impl IntoView {
    let filter = use_context::<ReadSignal<String>>()
        .expect("ERROR: Failed to get colormap read signal context in Canvas()");

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

        let mut granule_filter_string = "".to_string();

        // If there is a checked button in granule_type append its value to the filter_string
        if let Ok(nodes) = doc.query_selector_all("input[name=granule_type]:checked") {
            for i in 0..nodes.length() {
                let val = nodes
                    .get(i)
                    .expect("Failed to get node in on_submit")
                    .dyn_into::<web_sys::Element>()
                    .expect("Failed to cast Node to element")
                    .get_attribute("value")
                    .expect("Failed to get value attribute");
                granule_filter_string += &val;
                if i != nodes.length() - 1 {
                    granule_filter_string += "/";
                }
            }
        }

        if granule_filter_string.is_empty() {
            granule_filter_string += "GRD/SLC/OCN"
        }

        let mut sat_filter_string = "".to_string();
        // If there is a checked button in sat_selection append its value to the filter_string
        if let Ok(nodes) = doc.query_selector_all("input[name=sat_selection]:checked") {
            sat_filter_string += " ";
            for i in 0..nodes.length() {
                let val = nodes
                    .get(i)
                    .expect("Failed to get node in on_submit")
                    .dyn_into::<web_sys::Element>()
                    .expect("Failed to cast Node to element")
                    .get_attribute("value")
                    .expect("Failed to get value attribute");
                sat_filter_string += &val;
                if i != nodes.length() - 1 {
                    sat_filter_string += "/";
                }
            }
        }

        web_sys::console::log_1(&format!("{:?}", sat_filter_string).into());

        if sat_filter_string == *" " {
            sat_filter_string += "SA/SB";
        }

        let mut filter_string = granule_filter_string + &sat_filter_string;

        // Convert slider values into Dates
        let start_date = NaiveDate::from_num_days_from_ce_opt(start_date() as i32)
            .expect("Failed to parse start date");
        let end_date = NaiveDate::from_num_days_from_ce_opt(end_date() as i32)
            .expect("Failed to parse end date");

        filter_string += &(" ".to_string() + &start_date.format("%Y-%m-%d").to_string());

        filter_string += &(" ".to_string() + &end_date.format("%Y-%m-%d").to_string());

        set_filter(filter_string)
    };

    view! {
        <div class=style::user_interface>
            <form on:submit=on_submit>
                <div class=style::data_types>
                    <input
                        type="checkbox"
                        id="grd"
                        name="granule_type"
                        value="GRD"
                    />
                    <label class=style::radio_text for="grd">
                        "GRD"
                    </label>
                    <br/>
                    <input
                        type="checkbox"
                        id="slc"
                        name="granule_type"
                        value="SLC"
                    />
                    <label class=style::radio_text for="slc">
                        "SLC"
                    </label>
                    <br/>
                    <input
                        type="checkbox"
                        id="ocn"
                        name="granule_type"
                        value="OCN"
                    />
                    <label class=style::radio_text for="ocn">
                        "OCN"
                    </label>
                </div>

                <div class=style::sat_selection_div>
                    <input
                        type="checkbox"
                        id="s1-a"
                        name="sat_selection"
                        value="SA"
                    />
                    <label class=style::radio_text for="s1-a">
                        "S1A"
                    </label>
                    <br/>
                    <input
                        type="checkbox"
                        id="s1-b"
                        name="sat_selection"
                        value="SB"
                    />
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

            <p class=style::range_text>"Filter: " {filter}</p>

        </div>
    }
}
