#![feature(let_chains)]
use canvas::Canvas;
use leptos::*;
use ui::user_interface::UserInterface;

mod canvas;
mod ingest;
mod ui;

fn main() {
    console_error_panic_hook::set_once();

    leptos::mount_to_body(move || {
        view! {
            <div>
                <UserInterface></UserInterface>
                <Canvas></Canvas>
            </div>
        }
    })
}
