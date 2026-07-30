use leptos::{IntoView, component, prelude::*, view};

use crate::types::{PopupBody, PopupTitle};

#[component]
pub fn Popup() -> impl IntoView {
    let PopupTitle(title) =
        use_context::<PopupTitle>().expect("Failed to get title signal in Popup");
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
                        class="popup-button popup-button-open"
                        on:click=move |_| set_dismissed(true)
                    >
                        "×"
                    </button>
                    <h3 class="popup-text header">
                        {title}
                    </h3>
                    <span class="popup-text" inner_html=body/>
                </div>
            </div>
        </Show>
    }
}
