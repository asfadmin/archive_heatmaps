use leptos::logging::log;
use leptos::{IntoView, component, prelude::*, view};
use crate::types::ExpansionSignal;

#[component]
pub fn ExpansionButton(set_expanded: WriteSignal<bool>) -> impl IntoView {
    let ExpansionSignal(expanded) = use_context::<ExpansionSignal>()
        .expect("Failed to get expanded signal in ExpansionButton");

    let expansion_symbol = move || {
        match expanded() {
            true => { "-" },
            false => { "+" },
        }
    };

    view! {
        <button
            class="popup-button"
            class:popup-button-open=move || expanded()
            on:click=move |_| set_expanded(!expanded())
        >
        {expansion_symbol}
        </button>
    }
}

// +