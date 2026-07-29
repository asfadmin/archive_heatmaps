use leptos::logging::log;
use leptos::{IntoView, component, prelude::*, view};

use crate::types::ExpansionSignal;
use crate::ui::expansion_button::ExpansionButton;

#[component]
pub fn Legend() -> impl IntoView {
    let (expanded, set_expanded) = signal(true);
    provide_context(ExpansionSignal(expanded));

    view! {
        <div
            class="user-interface legend"
            class:floater-closed=move || !expanded()
        >
            <ExpansionButton set_expanded/>
            <Show when=move || { expanded() }>
                <p>This will be the legend</p>
            </Show>
        </div>
    }
}

// +
