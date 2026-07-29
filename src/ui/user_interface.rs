use chrono::naive::NaiveDate;
use leptos::wasm_bindgen::JsCast as _;
use leptos::{html, prelude::*};
use types::Filter;

use crate::ui::expansion_button::ExpansionButton;
use crate::types::ExpansionSignal;
use crate::types::{self, DateRange, ReadySignal};

#[component]
pub fn UserInterface(
    set_filter: WriteSignal<Filter>,
    set_title: WriteSignal<String>,
    set_body: WriteSignal<String>,
) -> impl IntoView {
    let filter =
        use_context::<ReadSignal<Filter>>().expect("Failed to get filter from context in UI");

    let ReadySignal(ready) =
        use_context::<ReadySignal>().expect("Failed to get ready read signal from context in UI");

    let (expanded, set_expanded) = signal(true);
    provide_context(ExpansionSignal(expanded));

    let (start_date, _) = signal(
        filter
            .get_untracked()
            .date_range
            .start
            .format("%Y-%m-%d")
            .to_string(),
    );
    let start_date_element: NodeRef<html::Input> = NodeRef::new();
    let (end_date, _) = signal(
        filter
            .get_untracked()
            .date_range
            .end
            .format("%Y-%m-%d")
            .to_string(),
    );
    let end_date_element: NodeRef<html::Input> = NodeRef::new();

    let max_date = chrono::Utc::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let min_date = NaiveDate::from_ymd_opt(2014, 6, 1)
        .expect("Failed to create min date in UI")
        .format("%Y-%m-%d")
        .to_string();

    let doc = document();

    // Run when an element of the UI changes, updates the filter signal
    let on_update = move |_| {
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
                    0 => product_type.push(types::ProductTypes::GroundRangeDetected),
                    1 => product_type.push(types::ProductTypes::SingleLookComplex),
                    2 => product_type.push(types::ProductTypes::Ocean),
                    _ => (),
                }
            }
        }

        if product_type.is_empty() {
            // TO-DO: Add feedback to user about why the query is not valid
            set_title("Invalid Filter".to_string());
            set_body("Include at least 1 product type".to_string());
            return;
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
                    0 => platform_type.push(types::PlatformType::Sentinel1A),
                    1 => platform_type.push(types::PlatformType::Sentinel1B),
                    2 => platform_type.push(types::PlatformType::Sentinel1C),
                    3 => platform_type.push(types::PlatformType::Sentinel1D),
                    _ => (),
                }
            }
        }

        if platform_type.is_empty() {
            // TO-DO: Add feedback to user about why the query is not valid
            set_title("Invalid Filter".to_string());
            set_body("Include at least 1 platform type".to_string());
            return;
        }

        // Gets the selected start and end dates
        let start_date_naive = NaiveDate::parse_from_str(
            &start_date_element
                .read_untracked()
                .as_ref()
                .expect("Failed to read start date element")
                .value(),
            "%Y-%m-%d",
        )
        .expect("Failed to parse start date from HTML Input");
        let end_date_naive = NaiveDate::parse_from_str(
            &end_date_element
                .read_untracked()
                .as_ref()
                .expect("Failed to read end date element")
                .value(),
            "%Y-%m-%d",
        )
        .expect("Failed to parse end date from HTML Input");

        if start_date_naive > end_date_naive {
            // TO-DO: Add feedback to user about why the query is not valid
            set_title("Invalid Filter".to_string());
            set_body("Start Date must be before End Date".to_string());
            return;
        }

        set_filter(types::Filter {
            product_type,
            platform_type,
            date_range: DateRange::new(start_date_naive, end_date_naive)
                .expect("Failed to create DateRange"),
        });
    };

    view! {
        <div 
            class="user-interface"
            class:floater-closed=move || !expanded()
        >
            <ExpansionButton set_expanded/>
            <Show when=move || { expanded() }>
                <form id="form">
                    <div id="checkboxes">
                        <div id="product_types">
                            <p>Products</p>
                            <input
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
                            <input
                                class="checkbox"
                                type="checkbox"
                                id="s1-c"
                                name="sat_selection"
                                value=2
                                checked
                            />
                            <label class="text" for="s1-c">
                                "S1C"
                            </label>
                            <br/>
                            <input
                                class="checkbox"
                                type="checkbox"
                                id="s1-d"
                                name="sat_selection"
                                value=3
                                checked
                            />
                            <label class="text" for="s1-d">
                                "S1D"
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
                                        max=max_date.clone()
                                        min=min_date.clone()
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
                                        max=max_date.clone()
                                        min=min_date.clone()
                                    />
                                </td>
                            </tr>
                        </table>
                    </div>
                    <div id="submit">
                        <input
                            type="button"
                            value="Generate Heatmap!"
                            class="button"
                            disabled=move || !ready()
                            on:click=on_update.clone()
                        />
                    </div>
                </form>
            </Show>
            // <div>
            //     <a
            //         class="button"
            //         on:click= move |_| {set_generate_img(true)}
            //     >
            //         Export to PNG
            //     </a>
            // </div>
        </div>
    }
}
