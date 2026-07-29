#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]

use canvas::Canvas;
use chrono::NaiveDate;
use leptos::{mount::mount_to_body, prelude::*};
use ui::{popup::Popup, user_interface::UserInterface, legend::Legend};

mod canvas;
mod ingest;
mod types;
mod ui;

use crate::types::{DateRange, GeneratePngSignal, PopupBody, PopupTitle, ReadySignal};

#[component]
fn Application() -> impl IntoView {
    // Default filter, used on startup
    let (filter, set_filter) = signal(types::Filter {
        product_type: vec![
            types::ProductTypes::GroundRangeDetected,
            types::ProductTypes::SingleLookComplex,
            types::ProductTypes::Ocean,
        ],
        platform_type: vec![
            types::PlatformType::Sentinel1A,
            types::PlatformType::Sentinel1B,
            types::PlatformType::Sentinel1C,
            types::PlatformType::Sentinel1D,
        ],
        date_range: DateRange::new(
            NaiveDate::from_ymd_opt(2026, 6, 1)
                .expect("Failed to create start date when creating filter signal"),
            NaiveDate::from_ymd_opt(2026, 7, 1)
                .expect("Failed to create end date when creating filter signal"),
        )
        .expect("Failed to create DateRange"),
    });
    provide_context(filter);

    // Determines if the loading bar is displayed or not, false is displayed, true is hidden
    let (ready, set_ready) = signal(false);
    provide_context(ReadySignal(ready));

    let (generate_img, set_generate_img) = signal(false);
    provide_context(GeneratePngSignal(generate_img));

    let (title, set_title) = signal("Disclaimers".to_string());
    provide_context(PopupTitle(title));

    let (body, set_body) = signal(
        "This product is in early development, expect to see bugs! <br/><br/>
        Generated data is not guaranteed to be accurate. <br/><br/>
        On encountering a crash the app will hang. <br/>
        Check the dev console if the page becomes unresponsive."
            .to_string(),
    );
    provide_context(PopupBody(body));

    view! {
        <div>
            <Popup/>
            <UserInterface set_filter set_title set_body/>
            <Canvas set_generate_img set_ready/>
            <Legend/>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(Application);
}
