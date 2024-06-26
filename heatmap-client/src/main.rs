use display::Canvas;
use leptos::*;

mod display;
mod ingest;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <Canvas/> })
}
