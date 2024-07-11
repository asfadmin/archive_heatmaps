#![feature(let_chains)]
use canvas::Canvas;
use leptos::*;

mod canvas;
mod ingest;

fn main() {
    console_error_panic_hook::set_once();

    let (count, set_count) = create_signal(0);

    leptos::mount_to_body(move || {
        view! {
            <div>
                <button
                    on:click = move |_| {
                        web_sys::console::log_1(&format!("{:?}", count()).into());
                        set_count(count() + 1)
                    }
                    style="position:absolute;top:10px;left:10px;z-index:1"
                >"Click Me!"</button>
                <Canvas></Canvas>
            </div>
        }
    })
}
