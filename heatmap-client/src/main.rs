#![feature(let_chains)]
use canvas::Canvas;
use leptos::*;
use ui::user_interface::UserInterface;

mod canvas;
mod ingest;
mod ui;

fn main() {
    console_error_panic_hook::set_once();

    let (filter, set_filter) = create_signal(String::from("magma"));

    provide_context(filter);

    let app = view! {
        <div>
            <UserInterface set_filter/>
            <Canvas></Canvas>
        </div>
    };

    leptos::mount_to_body(move || app)
}
