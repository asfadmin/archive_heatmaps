use leptos::{IntoView, component, prelude::*, view};

#[component]
pub fn Feedback() -> impl IntoView {
    view! {
        <div class="feedback" >
            <a
                class="feedback-button"
                target="_blank"
                href="https://github.com/asfadmin/archive_heatmaps/issues"
            >
            "💬"
            </a>
        </div>
    }
}
