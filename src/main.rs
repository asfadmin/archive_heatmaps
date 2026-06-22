#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]

use canvas::Canvas;
use chrono::NaiveDate;
    use leptos::{prelude::*, mount::mount_to_body };
use ui::user_interface::UserInterface;

mod canvas;
mod ingest;
mod ui;
mod types;

#[component]
fn App() -> impl IntoView {
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
        ],
        start_date: NaiveDate::from_ymd_opt(2019, 1, 1)
            .expect("Failed to create start date when creating filter signal")
            .format("%Y-%m-%d")
            .to_string(),
        end_date: NaiveDate::from_ymd_opt(2024, 4, 21)
            .expect("Failed to create end date when creating filter signal")
            .format("%Y-%m-%d")
            .to_string(),
    });
    provide_context(filter);

    let (generate_img, set_generate_img) = signal(false);
    provide_context(generate_img);

    view! {
        <div>
            <UserInterface set_filter set_generate_img/>
            <Canvas set_generate_img/>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}
