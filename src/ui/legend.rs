use leptos::html::div;
use leptos::{IntoView, component, prelude::*, view};
use leptos::logging::log;

use crate::types::ExpansionSignal;
use crate::ui::expansion_button::ExpansionButton;
use crate::MaxWeightSignal;

#[component]
pub fn Legend() -> impl IntoView {
    let (expanded, set_expanded) = signal(true);
    provide_context(ExpansionSignal(expanded));

    let MaxWeightSignal(max_weight) = use_context::<MaxWeightSignal>()
        .expect("Failed to get max weight signal in Legend");

    let legend_weights: &[u32; 8] = &[
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

    let colors_vec: Vec<(usize, String)> = legend_weights.iter().enumerate().map(|(i, x)| {

        let pixel = colormap_image.get_pixel(*x, 0).0;
        

        (i, format!("background-color: #{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2]))
    }).collect();
    let legend_len = colors_vec.len() - 1;

    let cloned_colors = colors_vec.clone();
    let weights = move || {
        let max = max_weight();

        let weights_vec = legend_weights.iter().map(|x| {
            ((x*(max) as u32) as f32)/(480.0*1.32)
        }).collect::<Vec<f32>>();

        cloned_colors.iter().map(|x| {
            match x.0 {
                0 => {format!("< {}", weights_vec[x.0])}
                i if i == legend_len => {format!("> {}", weights_vec[x.0])}
                _ => {format!("{} - {}", weights_vec[x.0 - 1], weights_vec[x.0])}
            }  
        }).collect::<Vec<String>>()
    };  

    let (weights_sig, _ ) = signal(weights);
    let (colors, _) = signal(colors_vec);

    view! {
        <div
            class="user-interface legend"
            class:floater-closed=move || !expanded()
        >
            <ExpansionButton set_expanded/>
            <Show when=move || { expanded() }>
                <div class="legend-container">
                    <For
                        each=move || colors()
                        key=|x| x.0
                        children=move |x| {
                            view!{
                                <div 
                                    class="legend-box"
                                    style=x.1
                                ></div>
                                <p>{ move || {
                                        format!("{}", weights_sig()()[x.0])
                                    }
                                }</p>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}