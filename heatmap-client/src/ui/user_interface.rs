use leptos::*;

stylance::import_crate_style!(style, "src/ui/user_interface.module.scss");

#[component]
pub fn UserInterface(set_filter: WriteSignal<String>) -> impl IntoView {

    let filter = use_context::<ReadSignal<String>>()
        .expect("ERROR: Failed to get colormap read signal context in Canvas()");

    view! {
        <div class=style::user_interface>

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
                <input class=style::radio_buttons type="radio" id="grd" name="granule_type" value="GRD"/>
                    <label class=style::radio_text for="grd">"GRD"</label><br/>
                <input class=style::radio_buttons type="radio" id="slc" name="granule_type" value="SLC"/>
                    <label class=style::radio_text for="slc">"SLC"</label><br/>
                <input class=style::radio_buttons type="radio" id="ocn" name="granule_type" value="OCN"/>
                    <label class=style::radio_text for="ocn">"OCN"</label>
            </div>

            <div class = style::sat_selection>
                <input class=style::radio_buttons type="radio" id="s1-a" name="sat_selection" value="S1-A"/>
                    <label class=style::radio_text for="s1-a">"S1-A"</label><br/>
                <input class=style::radio_buttons type="radio" id="s1-b" name="sat_selection" value="S1-B"/>
                    <label class=style::radio_text for="s1-b">"S1-B"</label><br/>
            </div>

        </div>
    }
}
