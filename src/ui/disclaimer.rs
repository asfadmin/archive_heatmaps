use leptos::{view, component, IntoView, prelude::*};



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
                        X
                    </button>
                    <h1 class="disclaimer-text">
                        Disclaimer: <br/>
                        This is an early iteraton of this product, expect to see bugs! <br/>
                        Data is not guaranteed to be accurate in realtime.
                    </h1>
                </div>
            </div>
        </Show>
    }
}