use app::TanksState;
use std::cell::RefCell;
use std::panic;
use tanks_core::shared_types::Coord;
use utils::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{MessageEvent, MouseEvent, WebSocket};

pub mod app;
mod utils;

thread_local! {
    /// Global State for the Game
    ///
    /// Do not panic while using this data, otherwise you may
    /// encounter a permanent locking of the Data
    static DATA: RefCell<TanksState> = RefCell::new(TanksState::new());

    /// Globally Store Mouse position
    static MOUSE_POS: RefCell<(f64,f64)> = RefCell::new((0.0,0.0));
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
#[allow(unused_variables)]
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
        if let Ok(a) = e.data().dyn_into::<js_sys::JsString>() {
            let movement: Coord =
                serde_json::from_str(&a.as_string().unwrap()).expect("invalid conversion");

            DATA.with(|state| {
                let mut game_state = state.borrow_mut();
                // game_state.update();
                game_state.add((movement.x, movement.y));
            });
            // console_log!("pos: {:#?}", game_state.pos);
        } else {
            console_log!("event: {:#?}", e.data());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    //
    // Signal the server to place the client into a session
    //
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

    // Mouse Tracking callback
    let mousemove_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
        MOUSE_POS.with(|pos| *pos.borrow_mut() = (event.offset_x() as f64, event.offset_y() as f64))
    }) as Box<dyn FnMut(_)>);
    canvas_element
        .add_event_listener_with_callback("mousemove", mousemove_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    mousemove_callback.forget();

    let canvas_context = canvas_element.get_2d_context();

    let render_loop = Box::new(move || {
        DATA.with(|state| {
            let game_state = state.borrow_mut();

            canvas_context.set_fill_style(&"#222".into());
            canvas_context.fill_rect(
                0.0,
                0.0,
                canvas_element.width().into(),
                canvas_element.height().into(),
            );

            canvas_context.set_fill_style(&"red".into());
            canvas_context.fill_rect(
                game_state.pos.0.into(),
                game_state.pos.1.into(),
                200.0,
                200.0,
            );

            canvas_context.set_stroke_style(&"white".into());
            canvas_context.begin_path();
            canvas_context.move_to(game_state.pos.0.into(), game_state.pos.1.into());
            MOUSE_POS.with(|pos| {
                let mouse_pos = pos.borrow();
                canvas_context.line_to(mouse_pos.0.into(), mouse_pos.1.into());
            });
            canvas_context.stroke();
        })
    });

    start_animation_loop(render_loop);
}
