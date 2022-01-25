use app::render;
use interface::{setup_canvas, setup_window_listeners};
use std::panic;
use utils::*;
use wasm_bindgen::prelude::*;

pub mod app;
pub mod audio;
pub mod interface;
pub mod login;
pub mod socket;
mod utils;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// Initialization process for the window
///
/// setups up logging and the canvas to start rendering
#[wasm_bindgen(start)]
pub fn start() {
    setup_logging();

    let canvas_element = setup_canvas();
    setup_window_listeners();

    let draw_procedure = move || render(&canvas_element, &canvas_element.get_2d_context());
    start_animation_loop(Box::new(draw_procedure));
}

fn setup_logging() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}
