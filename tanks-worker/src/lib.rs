use app::TanksState;
use std::cell::RefCell;
use std::panic;
use utils::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub mod app;
mod utils;

thread_local! {
    static DATA: RefCell<TanksState> = RefCell::new(TanksState::new());
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Initialization process for the window
///
/// setups up logging and the canvas to start rendering
/// additional setup like the websocket connection occur in function call with separate intent
#[wasm_bindgen(start)]
pub fn start() {
    setup_logging();
    setup_window();
}

#[wasm_bindgen]
pub fn connect(socket_uri: &str) {
    let params = get_query_params();

    let ws = WebSocket::new(&format!(
        "{}/{}",
        socket_uri,
        params
            .get("username")
            .expect("no username given in query params")
    ))
    .expect("connection established");

    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        DATA.with(|state| {
            let mut game_state = state.borrow_mut();
            game_state.update();
        });
        console_log!("event");
        // console_log!("event: {:#?}", e);
        // console_log!("event: {:#?}", e.data());
    }) as Box<dyn FnMut(MessageEvent)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onopen_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        cloned_ws.send_with_str(r#"{ "event_code": 2 }"#).unwrap();
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onopen_callback.forget();
}

fn setup_logging() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

fn setup_window() {
    let canvas_element = fetch_or_create_canvas();
    canvas_element.set_fullscreen();
    canvas_element.add_js_listener("mousemove", Box::new(|| {}));

    let canvas_context = canvas_element.get_2d_context();

    let render_loop = Box::new(move || {
        DATA.with(|state| {
            canvas_context.set_fill_style(&"#222".into());
            canvas_context.fill_rect(
                0.0,
                0.0,
                canvas_element.width().into(),
                canvas_element.height().into(),
            );

            let game_state = state.borrow_mut();
            canvas_context.set_fill_style(&"red".into());
            canvas_context.fill_rect(
                game_state.pos.0.into(),
                game_state.pos.1.into(),
                200.0,
                200.0,
            );
        })
    });

    start_animation_loop(render_loop);
}
