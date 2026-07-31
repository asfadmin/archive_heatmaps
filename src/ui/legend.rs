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
        let max = max_weight();
        let granularity = 8; // Maximum number of legend entries
        match max {
            i if i <= granularity => (1..i).collect::<Vec<u32>>(),
            i => (1..granularity)
                .map(|n| (i as f32 * ((n as f32) / granularity as f32)) as u32)
                .collect::<Vec<u32>>(),
        }
        .iter()
        .filter_map(|x| {
            let scaled_weight = match x {
                n @ 0..10 => *n as usize,
                n => ((f64::from(*n) / 5.0).round() * 5.0) as usize,
            };
            let val = calc_tex_coord(scaled_weight as f32, max as f32);
            if val > 0 && val < 480 {
                Some(scaled_weight)
            } else {
                None
            }
        })
        .collect::<Vec<usize>>()
    };

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
                        each=move || { weights().iter().enumerate().map(|(i, x)| (i, *x)).collect::<Vec<(usize, usize)>>() }
                        key=|x| x.0 + x.1
                        children=move |(i, x)| {
                            let wvec = weights();
                            let row = format!("{}", i + 2);

                            // Reading the image every time the signal update feels bad...
                            let colormap_bytes = include_bytes!("../../assets/magma.png");
                            let colormap_image = image::load_from_memory(colormap_bytes)
                                .expect("ERROR: Failed to generate image from colormap_bytes")
                                .to_rgba8();

                            let max = max_weight();
                            let coord = calc_tex_coord(x as f32, max as f32);
                            let pixel = colormap_image.get_pixel(coord, 0).0;

                            let background_color = format!(
                                "background-color: #{:02x}{:02x}{:02x}",
                                pixel[0], pixel[1], pixel[2]
                            );

                            let legend_text = create_legend_text(i, x, &wvec);

                            view!{
                                <div
                                    class="legend-box"
                                    style=background_color
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

/// Format weight into text for the legend.
fn create_legend_text(i: usize, weight: usize, wvec: &[usize]) -> String {
    match i {
        0 => {
            if weight > 1 {
                format!("<= {weight}")
            } else {
                format!("{weight}")
            }
        }
        i if i + 1 == wvec.len() => {
            let last = wvec[i - 1];
            if weight - last > 1 {
                format!("> {}", wvec[i - 1])
            } else {
                format!("{weight}")
            }
        }
        _ => {
            let last = wvec[i - 1];
            if weight - last > 1 {
                format!("{} - {weight}", last + 1)
            } else {
                format!("{weight}")
            }
        }
    }
}
