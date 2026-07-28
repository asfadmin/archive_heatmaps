use leptos::{IntoView, component, prelude::*, view};

#[component]
pub fn Disclaimer() -> impl IntoView {
    let (dismissed, set_dismissed) = signal(false);
    view! {
        <Show when=move || { !dismissed() }>
            <div
                class="blur"
            >
                <div class="disclaimer">
                    <button
                        class="exit-button"
                        on:click=move |_| set_dismissed(true)
                    >
                        "×"
                    </button>
                    <h3 class="disclaimer-text header">
                        Disclaimer
                    </h3>
                    <span class="disclaimer-text">
                        This product is in early development, expect to see bugs! <br/>
                        Generated data is not guaranteed to be accurate.
                    </span>
                </div>
            </div>
        </Show>
    }
}
