// We allow expect for the whole module as winit makes it nearly impossible
// to properly manage error handling.
#![allow(clippy::expect_used)]

pub use canvas::Canvas;

pub mod app;
mod camera;
mod canvas;
pub mod geometry;
mod input;
mod render_context;
mod state;
