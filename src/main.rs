#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::single_call_fn,
    clippy::same_name_method,
    clippy::expect_used,
    clippy::absolute_paths,
    clippy::implicit_return,
    clippy::too_many_lines,
)]

use canvas::Canvas;
use chrono::NaiveDate;
use leptos::{mount::mount_to_body, prelude::*};
use ui::user_interface::UserInterface;

mod canvas;
mod ingest;
mod types;
mod ui;

use crate::types::DateRange;

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
    mount_to_body(Application);
}
