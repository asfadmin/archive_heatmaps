use leptos::*;

stylance::import_crate_style!(style, "src/ui/user_interface.module.scss");

#[component]
pub fn UserInterface() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <button
            on:click = move |_| {
                web_sys::console::log_1(&format!("{:?}", count()).into());
                set_count(count() + 1)
            }
            class=style::user_interface
        >"Click Me!"</button>

        <select
            class=style::colormap_selector
        >
        <option value="magma">"Magma"</option>
        <option value="rainbow-soft">"Rainbow Soft"</option>
        </select>
    }
}
