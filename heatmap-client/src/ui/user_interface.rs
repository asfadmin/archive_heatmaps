use leptos::*;
use web_sys::Element;
use leptos::wasm_bindgen::JsCast;

stylance::import_crate_style!(style, "src/ui/user_interface.module.scss");

#[component]
pub fn UserInterface(set_filter: WriteSignal<String>) -> impl IntoView {

    let filter = use_context::<ReadSignal<String>>()
        .expect("ERROR: Failed to get colormap read signal context in Canvas()");

    let doc = document();

    // Might be worth reworking this, we are mixing web_sys and leptos here but weve done the same elsewhere so we could also just roll with it
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // Stop page from reloading
        ev.prevent_default();
        
        let mut filter_string = "".to_string();

        // If there is a checked button in granule_type append its value to the filter_string
        if let Ok(nodes) = doc.query_selector_all("input[name=granule_type]:checked") {

            for i in 0..nodes.length() {
                let val = nodes.get(i)
                .expect("Failed to get node in on_submit")
                .dyn_into::<web_sys::Element>()
                .expect("Failed to cast Node to element")
                .get_attribute("value")
                .expect("Failed to get value attribute");
                filter_string += &val;
                if i != nodes.length() - 1 {
                    filter_string += "/";
                }
            }

        }

        // If there is a checked button in sat_selection append its value to the filter_string
        if let Ok(nodes) = doc.query_selector_all("input[name=sat_selection]:checked") {
            filter_string += " ";
            for i in 0..nodes.length() {
                let val = nodes.get(i)
                .expect("Failed to get node in on_submit")
                .dyn_into::<web_sys::Element>()
                .expect("Failed to cast Node to element")
                .get_attribute("value")
                .expect("Failed to get value attribute");
                filter_string += &val;
                if i != nodes.length() - 1 {
                    filter_string += "/";
                }
            }

        }

        set_filter(filter_string)
    };

    view! {
        <div class=style::user_interface>
            <form on:submit=on_submit>
                <input
                    class=style::range_slider
                    id="date_range"
                    type="range"
                    min="1"
                    max="100"
                    value="50"
                />
                    <br/><label class=style::range_text for="date_range">"Date Range"</label><br/>

                <div class = style::data_types>
                    <input class=style::check_box type="checkbox" id="grd" name="granule_type" value="GRD" checked/>
                        <label class=style::radio_text for="grd">"GRD"</label><br/>
                    <input class=style::check_box type="checkbox" id="slc" name="granule_type" value="SLC" checked/>
                        <label class=style::radio_text for="slc">"SLC"</label><br/>
                    <input class=style::check_box type="checkbox" id="ocn" name="granule_type" value="OCN" checked/>
                        <label class=style::radio_text for="ocn">"OCN"</label>
                </div>

                <div class = style::sat_selection>
                    <input class=style::check_box type="checkbox" id="s1-a" name="sat_selection" value="S1-A" checked/>
                        <label class=style::radio_text for="s1-a">"S1-A"</label><br/>
                    <input class=style::check_box type="checkbox" id="s1-b" name="sat_selection" value="S1-B" checked/>
                        <label class=style::radio_text for="s1-b">"S1-B"</label><br/>
                </div>

                <input class=style::submit_button type="submit" value="Submit"/>
            </form>

            <p class=style::range_text>"Filter: "{filter}</p>

        </div>
    }
}
