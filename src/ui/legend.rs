use leptos::html::div;
use leptos::{IntoView, component, prelude::*, view};

use crate::types::ExpansionSignal;
use crate::ui::expansion_button::ExpansionButton;
use crate::MaxWeightSignal;

#[component]
pub fn Legend() -> impl IntoView {
    let (expanded, set_expanded) = signal(true);
    provide_context(ExpansionSignal(expanded));

    let MaxWeightSignal(max_weight) = use_context::<MaxWeightSignal>()
        .expect("Failed to get max weight signal in Legend");

    let LEGEND_WEIGHTS: &[u32; 8] = &[
        30,
        80,
        130,
        170,
        230,
        300,
        370,
        450,
    ];

    let colormap_bytes = include_bytes!("../../assets/magma.png");
    let colormap_image = image::load_from_memory(colormap_bytes)
        .expect("ERROR: Failed to generate image from colormap_bytes")
        .to_rgba8();

    let colors: Vec<(usize, String, _)> = LEGEND_WEIGHTS.iter().enumerate().map(|(i, x)| {
        let weight = move || ((x*max_weight() as u32) as f32)/(480.0*1.32);
        let pixel = colormap_image.get_pixel(*x, 0).0;
        (i, format!("background-color: #{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2]), weight)
    }).collect();

    let (sig, set_sig) = signal(colors);

    view! {
        <div
            class="user-interface legend"
            class:floater-closed=move || !expanded()
        >
            <ExpansionButton set_expanded/>
            <Show when=move || { expanded() }>
                <div class="legend-container">
                    <For
                        each=move || sig()
                        key=|x| x.0
                        children=move |x| {
                            view!{
                                <div 
                                    class="legend-box"
                                    style=x.1
                                ></div>
                                <p>{x.2}</p>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}