use leptos::{IntoView, component, prelude::*, view};
use leptos::logging::log;

use crate::types::{PopupTitle, PopupBody};

#[component]
pub fn Popup() -> impl IntoView {

    let PopupTitle(title) = use_context::<PopupTitle>().expect("Failed to get title signal in Popup");
    let PopupBody(body) = use_context::<PopupBody>().expect("Failed to get body signal in Popup");

    let (dismissed, set_dismissed) = signal(false);

    Effect::new(move |_| {
        title();
        body();
        set_dismissed(false);
    });

    view! {
        <Show when=move || { !dismissed() }>
            <div
                class="blur"
            >
                <div class="popup">
                    <button
                        class="exit-button"
                        on:click=move |_| set_dismissed(true)
                    >
                        "×"
                    </button>
                    <h3 class="popup-text header">
                        {title}
                    </h3>
                    <span class="popup-text" inner_html=move || body()/>
                </div>
            </div>
        </Show>
    }
}


/*
<h3 class="popup-text header">
    Disclaimer
</h3>
<span class="popup-text">
    This product is in early development, expect to see bugs! <br/>
    Generated data is not guaranteed to be accurate.
</span>
*/