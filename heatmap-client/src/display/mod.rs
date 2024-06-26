// We allow expect for the whole module as winit makes it nearly impossible
// to properly manage error handling.
#![allow(clippy::expect_used)]

pub use canvas::Canvas;

pub mod app;
mod canvas;
pub mod geometry;
mod render_context;
mod state;
