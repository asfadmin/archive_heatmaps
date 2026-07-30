use std::rc::Rc;
use std::sync::Arc;

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

    let calc_tex_coord = |x: f32, max: f32| (((x * 1.32) / max) * 480.0) as u32;

    // List of weights to use for the legend
    let weights = move || {
        log!("Updating Weights Signal");
        let max = max_weight();
        log!("Max weight in signal: {max}");
        let granularity = 8;
        match max {
            i if i <= granularity => (1..i).map(|n| n).collect::<Vec<u32>>(),
            i @ _ => (1..granularity)
                .filter_map(|n| {
                    let weight = (i as f32 * ((n as f32) / granularity as f32)) as u32;
                    let val = calc_tex_coord(weight as f32, i as f32);
                    log!("Weight: {weight}\tTex Coord: {val}");
                    if val > 0 && val < 480 {
                        Some(weight)
                    } else {
                        None
                    }
                })
                .collect::<Vec<u32>>(),
        }
        .iter()
        .map(|x| match x {
                n @ 0..10 => *n as usize,
                n => ((*n as f64 / 5.0).round() * 5.0) as usize,
            }
        )
        .enumerate()
        .collect::<Vec<(usize, usize)>>()
    };



    let colormap_bytes = include_bytes!("../../assets/magma.png");
    let colormap_image = Arc::new(
        image::load_from_memory(colormap_bytes)
            .expect("ERROR: Failed to generate image from colormap_bytes")
            .to_rgba8(),
    );

    // let colors = move || {
    //     let img = colormap_image.clone();
    //     weights()
    //         .iter()
    //         .enumerate()
    //         .map(|(i, x)| {
    //             let max = max_weight();
    //             let coord = calc_tex_coord(*x as f32, max as f32);
    //             let pixel = colormap_image.clone().get_pixel(coord, 0).0;

    //             (
    //                 i,
    //                 format!(
    //                     "background-color: #{:02x}{:02x}{:02x}",
    //                     pixel[0], pixel[1], pixel[2]
    //                 ),
    //             )
    //         })
    //         .collect::<Vec<(usize, String)>>()
    // };
    // let (csig, _) = signal(colors());

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
                        each=move || weights()
                        key=|x| x.1
                        children=move |(i, x)| {
                            let wvec = weights();
                            log!("wvec: {wvec:?}");
                            let row = format!("{}", i + 2);
                            log!("Handling: {}", i);

                            let legend_text = match i {
                                0 => {
                                    if x > 1 {
                                        format!("<= {}", x)
                                    } else {
                                        format!("{x}")
                                    }
                                }
                                i if i+1 == wvec.len() => {format!("> {}", wvec[i-1].1)}
                                _ => {
                                    let last = wvec[i-1].1;
                                    if x-last > 1 {
                                        format!("{} - {}", last+1, x)
                                    } else {
                                        format!("{x}")
                                    }
                                }
                            };

                            view!{
                                <div
                                    class="legend-box"
                                    // style=x.1
                                    style:grid-row=row.clone()
                                ></div>
                                <span
                                    style:grid-row=row
                                >{legend_text}</span>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
