use heatmap::Canvas;
use leptos::*;

mod heatmap;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <Canvas/> })
}
