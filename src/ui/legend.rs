use leptos::logging::log;
use leptos::{IntoView, component, prelude::*, view};

use crate::MaxWeightSignal;
use crate::types::ExpansionSignal;
use crate::ui::expansion_button::ExpansionButton;

#[component]
pub fn Legend() -> impl IntoView {
    let (expanded, set_expanded) = signal(true);
    provide_context(ExpansionSignal(expanded));

    let MaxWeightSignal(max_weight) =
        use_context::<MaxWeightSignal>().expect("Failed to get max weight signal in Legend");

    let calc_tex_coord = |x: f32, max: f32| {
        (((x*1.32)/max) * 480.0) as u32
    };


    // List of evenly distributed weights that can be mapped to tex coords
    let weight_samples = move || {
        let granularity = 8;
        match max_weight() {
            i if i <= granularity => (1..i).map(|n| n).collect::<Vec<u32>>(),
            i @ _ => (1..granularity)
                .filter_map(|n| {
                    let weight = (i as f32 * ((n as f32) / granularity as f32)) as u32;
                    let val = calc_tex_coord(weight as f32, i as f32);
                    log!("Weight: {weight}\tTex Coord: {val}");
                    if val > 0 && val < 480 { Some(weight) } else { None }
                })
                .collect::<Vec<u32>>(),
        }
    };

    // List of tex coords we can map to weights
    let legend_weights: &[u32; 8] = &[30, 80, 130, 170, 230, 300, 370, 450];

    let colormap_bytes = include_bytes!("../../assets/magma.png");
    let colormap_image = image::load_from_memory(colormap_bytes)
        .expect("ERROR: Failed to generate image from colormap_bytes")
        .to_rgba8();

    let colors_vec: Vec<(usize, String)> = legend_weights
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let pixel = colormap_image.get_pixel(*x, 0).0;

            (
                i,
                format!(
                    "background-color: #{:02x}{:02x}{:02x}",
                    pixel[0], pixel[1], pixel[2]
                ),
            )
        })
        .collect();
    let legend_len = colors_vec.len() - 1;

    let weights = move || {
        let max = max_weight();
        log!("Weight Samples: {:?}", weight_samples());

        legend_weights
            .iter()
            .map(|x| {
                let val = (((x * max) as f32) / (480.0 * 1.32)) as usize;
                match val {
                    n @ 0..10 => n,
                    n => ((n as f64 / 5.0).round() * 5.0) as usize,
                }
            })
            .collect::<Vec<usize>>()
    };

    // Feels bad to wrap a derived signal in a signal to avoid type errors...
    let (weights_sig, _) = signal(weights);
    let (colors, _) = signal(colors_vec);

    view! {
        <div
            class="legend"
            class:floater-closed=move || !expanded()
        >
            <ExpansionButton set_expanded/>
            <Show when=move || { expanded() }>
                <div class="legend-container">
                    <h3
                        class="legend-header"
                    >
                        Acquisitions
                    </h3>
                    <For
                        each=move || colors()
                        key=|x| x.0
                        children=move |x| {
                            let row = format!("{}", x.0 + 2);
                            view!{
                                <div
                                    class="legend-box"
                                    style=x.1
                                    style:grid-row=row.clone()
                                ></div>
                                <span
                                    style:grid-row=row
                                >{ move || {
                                        let weights_vec = weights_sig()();
                                        log!("Handling: {}", x.0);
                                        match x.0 {
                                            0 => {format!("< {}", weights_vec[x.0])}
                                            i if i == legend_len => {format!("> {}", weights_vec[x.0])}
                                            _ => {format!("{} - {}", weights_vec[x.0 - 1], weights_vec[x.0])}
                                        }
                                    }
                                }</span>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
