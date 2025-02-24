use base64::Engine;
use heatmap_api::{Filter, PlatformType, ProductTypes};
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use image::{ImageBuffer, Rgba};
use leptos::{ReadSignal, WriteSignal};
use text_to_png::TextRenderer;

use super::render_context::RenderContext;
use crate::heatmap_api;

/// Generate the final png that will be exported
pub fn generate_heatmap_image(render_context: &mut RenderContext, filter: Filter) -> String {
    // We read the data contained in the buffer and convert it from &[u8] to Vec<u8>
    let raw_bytes: Vec<u8> = (&*render_context
        .copy_context
        .buffer
        .slice(..)
        .get_mapped_range())
        .into();

    // The buffer is formated for f32 but we pulled a Vec<u8>, we must reform the Vec<f32> from the bytes
    let mut color_data: Vec<f32> = Vec::new();
    let mut raw_iter = raw_bytes.iter();
    while let Ok(raw) = raw_iter.next_chunk::<4>() {
        // Read one channel into a f32
        color_data.push(f32::from_le_bytes([*raw[0], *raw[1], *raw[2], *raw[3]]));
    }

    web_sys::console::log_1(&format!("Freshley decoded: {:?}", color_data).into());

    // Convert the raw image data into an ImageBuffer that can be saved, must use copy texture width here,
    //     Copy Texture is 256 byte aligned so copy_texture.width() is larger than displayed size and so
    //     are the contents of our buffer
    let colormap_img = image::Rgba32FImage::from_vec(
        render_context.copy_context.texture.width(),
        render_context.copy_context.texture.height(),
        color_data,
    )
    .expect("Failed to convert parsed floats into an Rgba<f32> ImageBuffer");

    // Generate the image to be export
    let output_img = generate_export_image(
        &colormap_img,
        render_context
            .max_weight_context
            .value
            .expect("Failed to get max weight to generate output png"),
        filter,
    );

    // Release the copy buffer for later use
    render_context.copy_context.buffer.unmap();
    render_context.copy_context.buffer_mapped = false;

    let output_img_width = output_img.width();
    let output_img_height = output_img.height();

    let raw_image_data: Vec<u8> = image::DynamicImage::from(output_img).to_rgba8().into_raw();

    // Encode the generated ImageBuffer into a png
    let mut png_encoded = Vec::<u8>::new();
    let png_encoder = PngEncoder::new(&mut png_encoded);
    let _result = png_encoder.write_image(
        raw_image_data.as_slice(),
        output_img_width,
        output_img_height,
        image::ExtendedColorType::Rgba8,
    );

    // Encode the png in base64 to allow for download from an html anchor tag
    base64::engine::general_purpose::STANDARD.encode(&png_encoded)
}

pub fn generate_export_image(
    colormap_img: &ImageBuffer<Rgba<f32>, Vec<f32>>,
    max_weight: f32,
    filter: Filter,
) -> ImageBuffer<Rgba<f32>, Vec<f32>> {
    let colormap_img_width = 3083;
    let colormap_img_height = 1551;

    /////////////////////
    // Read Image Data //
    /////////////////////

    let resized_colormap_img = image::imageops::resize(
        colormap_img,
        colormap_img_width,
        colormap_img_height,
        image::imageops::FilterType::Nearest,
    );

    // Store image template in memory
    let template_bytes = include_bytes!("../../assets/export_template.png");
    let mut template_img = image::load_from_memory(template_bytes)
        .expect("ERROR: Failed to load export_template.png")
        .to_rgba32f();

    // Store the world outline in memory, must be overlayed after heatmap image to appear on final image
    let outline_bytes = include_bytes!("../../assets/export_outline.png");
    let outline_img = image::load_from_memory(outline_bytes)
        .expect("ERROR: Failed to load export_outline.png")
        .to_rgba32f();
    let resized_outline_img = image::imageops::resize(
        &outline_img,
        colormap_img_width,
        colormap_img_height,
        image::imageops::FilterType::Nearest,
    );

    ///////////////////////
    // Create the Legend //
    ///////////////////////

    // Renders text into a png we can overlay on the rest of our image
    let text_renderer = TextRenderer::try_new_with_ttf_font_data(include_bytes!(
        "../../assets/times_new_roman.ttf"
    ))
    .expect("ERROR: Failed to read times new roman font when creating text renderer");

    // Store the legend in memory
    let legend_bytes = include_bytes!("../../assets/export_legend.png");
    let mut legend_img = image::load_from_memory(legend_bytes)
        .expect("ERROR: Failed to load export_legend")
        .to_rgba32f();

    // These numbers correspond to the ratio of each color on the export colormap, ie num_pixels_of_color/length_of_texture
    let legend_weights = [
        0.004166667,
        0.010416667,
        0.010416667,
        0.020833333,
        0.022916667,
        0.075,
        0.15,
        0.714583,
    ];

    // Adds numbers to the legend, each range is added individually to allow
    // for proper line spacing. Renders text of size font_size into a png format
    // this is then converted into a ImageBuffer and overlayed onto the legend
    let font_size = 56;
    let mut layer = 0;
    let mut last_upper = 1.0; // Everywhere with color on the heatmap has >=1 images
    while layer < 7 {
        let upper = ((max_weight * legend_weights[layer]) + last_upper).ceil();
        let text_data = text_renderer
            .render_text_to_png_data(
                format!("{:?}-{:?}", last_upper as u32, upper as u32),
                font_size,
                0x0,
            )
            .expect("ERROR: Failed to create text_data png")
            .data;
        let text_img = image::load_from_memory(&text_data)
            .expect("ERROR: Failed to create dynamic image for data_text")
            .to_rgba32f();
        image::imageops::overlay(&mut legend_img, &text_img, 152, 197 + (53 * layer as i64));

        last_upper = upper;
        layer += 1;

        web_sys::console::log_1(
            &format!("Upper: {:?}\nRunning Total: {:?}", upper, last_upper).into(),
        );
    }

    // Last range in legend, formatting is unique so it cant be done in the loop
    let text_data = text_renderer
        .render_text_to_png_data(format!("> {:?}", last_upper as u32), font_size, 0x0)
        .expect("ERROR: Failed to create final text_data png")
        .data;
    let text_img = image::load_from_memory(&text_data)
        .expect("ERROR: Failed to create dynamic image for text_data")
        .to_rgba32f();
    image::imageops::overlay(&mut legend_img, &text_img, 152, 197 + (53 * layer as i64));

    /////////////////////////////////
    // Create Labels and Date Text //
    /////////////////////////////////

    // Parse the filter into text that can be turned into images
    let filter_text = filter_to_text(filter);
    let font_size = 68;
    let line1_data = text_renderer
        .render_text_to_png_data(
            "Copernicus ".to_owned() + &filter_text.1 + " data,",
            font_size,
            0x0,
        )
        .expect("ERROR: Failed to create date range text")
        .data;
    let line2_data = text_renderer
        .render_text_to_png_data(filter_text.2 + "-" + &filter_text.3, font_size, 0x0)
        .expect("ERROR: Failed to create date range text")
        .data;
    let line3_data = text_renderer
        .render_text_to_png_data(
            "Current: 30 April ".to_owned() + &filter_text.3,
            font_size,
            0x0,
        )
        .expect("ERROR: Failed to create date range text")
        .data;
    let line4_data = text_renderer
        .render_text_to_png_data(
            "Map shows Sentinel-1 ".to_owned() + &filter_text.0,
            font_size,
            0x0,
        )
        .expect("ERROR: Failed to create date range text")
        .data;
    let line5_data = text_renderer
        .render_text_to_png_data("product global coverage", font_size, 0x0)
        .expect("ERROR: Failed to create date range text")
        .data;

    let line1_img = image::load_from_memory(&line1_data)
        .expect("ERROR: Failed to create dynamic image for line1 text")
        .to_rgba32f();
    let line2_img = image::load_from_memory(&line2_data)
        .expect("ERROR: Failed to create dynamic image for line2 text")
        .to_rgba32f();
    let line3_img = image::load_from_memory(&line3_data)
        .expect("ERROR: Failed to create dynamic image for line3 text")
        .to_rgba32f();
    let line4_img: ImageBuffer<Rgba<f32>, Vec<f32>> = image::load_from_memory(&line4_data)
        .expect("ERROR: Failed to create dynamic image for line4 text")
        .to_rgba32f();
    let line5_img: ImageBuffer<Rgba<f32>, Vec<f32>> = image::load_from_memory(&line5_data)
        .expect("ERROR: Failed to create dynamic image for line5 text")
        .to_rgba32f();

    // Create the blank template to write the date and info text onto
    let date_img_width: u32 = 1240;
    let mut date_img = image::ImageBuffer::<Rgba<f32>, Vec<f32>>::new(date_img_width, 400);
    date_img.pixels_mut().for_each(|x| {
        x.0 = [1.0, 1.0, 1.0, 1.0];
    });

    image::imageops::overlay(
        &mut date_img,
        &line1_img,
        center_img(date_img_width, &line1_img).into(),
        0,
    );
    image::imageops::overlay(
        &mut date_img,
        &line2_img,
        center_img(date_img_width, &line2_img).into(),
        font_size,
    );
    image::imageops::overlay(
        &mut date_img,
        &line3_img,
        center_img(date_img_width, &line3_img).into(),
        font_size * 2,
    );
    image::imageops::overlay(
        &mut date_img,
        &line4_img,
        center_img(date_img_width, &line4_img).into(),
        400 - font_size * 2,
    );
    image::imageops::overlay(
        &mut date_img,
        &line5_img,
        center_img(date_img_width, &line5_img).into(),
        400 - font_size,
    );

    ////////////////////////
    // Create Final Image //
    ////////////////////////

    // resized_colormap_img must come before resized_outline_img for the world outline to display properly
    image::imageops::overlay(&mut template_img, &resized_colormap_img, 216, 186);
    image::imageops::overlay(&mut template_img, &resized_outline_img, 216, 186);
    image::imageops::overlay(&mut template_img, &legend_img, 2804, 1750);
    image::imageops::overlay(&mut template_img, &date_img, 1563, 1796);

    template_img
}

// Helper Function:
//     Returns the x coordinate that centers one image on another image
fn center_img(destination_width: u32, text: &ImageBuffer<Rgba<f32>, Vec<f32>>) -> u32 {
    let dest_center = destination_width / 2;
    let text_offset = text.width() / 2;

    // Text is to wide to center, place start of text at 0
    if text_offset > dest_center {
        return 0;
    }

    dest_center - text_offset
}

// Parse a filter into a tuple of four strings, the contents of each string are:
//      0. Product Type
//      1. Platform
//      2. Start Year
//      3. End Year
fn filter_to_text(filter: Filter) -> (String, String, String, String) {
    let mut product_string = "".to_owned();
    for product_type in filter.product_type.iter().enumerate() {
        if product_type.0 != 0 && product_type.0 < filter.product_type.len() - 1 {
            product_string += ", "
        } else if product_type.0 == filter.product_type.len() - 1 {
            product_string += " and "
        }
        match product_type.1 {
            ProductTypes::GroundRangeDetected => product_string += "GRD",
            ProductTypes::SingleLookComplex => product_string += "SLC",
            ProductTypes::Ocean => product_string += "OCN",
        }
    }

    let mut platform_string = "".to_owned();
    for platform_type in filter.platform_type.iter().enumerate() {
        if platform_type.0 != 0 && platform_type.0 < filter.platform_type.len() - 1 {
            platform_string += ", "
        } else if platform_type.0 == filter.platform_type.len() - 1 {
            platform_string += " and "
        }
        match platform_type.1 {
            PlatformType::Sentinel1A => platform_string += "Sentinel-1A",
            PlatformType::Sentinel1B => platform_string += "Sentinel-1B",
        }
    }

    let start_year: String = filter
        .start_date
        .chars()
        .take_while(|x| *x != '-')
        .collect();
    let end_year: String = filter.end_date.chars().take_while(|x| *x != '-').collect();

    (product_string, platform_string, start_year, end_year)
}

/// Tracks setup stage of state and png generations
#[derive(Default, PartialEq, Eq, Clone)]
pub enum InitStage {
    #[default]
    Incomplete,
    InProgress,
    Complete,
}
/// Contains a tracker for the state of png generation and the image that was generated
#[derive(Clone)]
pub struct ExportContext {
    pub generate_img: ReadSignal<bool>,
    pub set_generate_img: WriteSignal<bool>,
    pub stage: InitStage,
    pub base64_png: Option<String>,
}
