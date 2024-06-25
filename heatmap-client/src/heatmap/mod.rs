// We allow expect for the whole module as winit makes it nearly impossible
// to properly manage error handling.
#![allow(clippy::expect_used)]

pub use canvas::Canvas;

mod app;
mod canvas;
mod geometry;
mod render_context;
mod state;
