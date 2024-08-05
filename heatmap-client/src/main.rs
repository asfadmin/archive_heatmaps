#![feature(let_chains)]
#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]

use canvas::Canvas;
use chrono::NaiveDate;
use leptos::*;
use ui::user_interface::UserInterface;
extern crate heatmap_api;

mod canvas;
mod ingest;
mod ui;

fn main() {
    console_error_panic_hook::set_once();

    // Default filter, used on startup
    let (filter, set_filter) = create_signal(heatmap_api::Filter {
        product_type: vec![
            heatmap_api::ProductTypes::GroundRangeDetected,
            heatmap_api::ProductTypes::SingleLookComplex,
            heatmap_api::ProductTypes::Ocean,
        ],
        platform_type: vec![
            heatmap_api::PlatformType::Sentinel1A,
            heatmap_api::PlatformType::Sentinel1B,
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

    let app = view! {
        <div>
            <UserInterface set_filter/>
            <Canvas/>
        </div>
    };

    leptos::mount_to_body(move || app)
}
