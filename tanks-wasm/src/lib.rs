use app::{handle_server_event, render, ClientGameState};
use std::panic;
use std::{cell::RefCell, rc::Rc};
use tanks_core::{
    server_types::{ClientEvent, ServerEvent},
    shared_types::Coord,
};
use utils::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, KeyboardEvent, MessageEvent, MouseEvent, WebSocket};

pub mod app;
mod utils;

thread_local! {
    /// Global State for the Game
    ///
    /// Do not panic while using this data, otherwise you may
    /// encounter a permanent locking of the Data
    static DATA: RefCell<ClientGameState> = RefCell::new(ClientGameState::new(""));

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
}

#[wasm_bindgen]
pub fn connect(socket_uri: &str) {
    let params = get_query_params();

    let username = params
        .get("username")
        .expect("no username given in query params");

    //======================================================
    //
    // WebSocket Setup
    //
    //======================================================
    let ws =
        WebSocket::new(&format!("{}/{}", socket_uri, username)).expect("connection established");

    // Update the state once we have connected to the server
    DATA.with(|data| *data.borrow_mut() = ClientGameState::new(username));

    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(a) = e.data().dyn_into::<js_sys::JsString>() {
            let event: ServerEvent = match serde_json::from_str(&a.as_string().unwrap()) {
                Ok(event) => event,
                Err(e) => return console_log!("{}", e), // failed conversion
            };

            DATA.with(|state| handle_server_event(event, &mut state.borrow_mut()));
        } else {
            console_log!("event: {:#?}", e.data());
        }
    }) as Box<dyn FnMut(_)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    // Signal the server to place the client into a session
    let onopen_callback = Closure::wrap(Box::new(move |_: MessageEvent| {
        cloned_ws
            .send_with_str(&serde_json::to_string(&ClientEvent::JoinSession).unwrap())
            .unwrap();
    }) as Box<dyn FnMut(_)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onopen_callback.forget();

    //======================================================
    //
    // Cavnas Setup
    //
    //======================================================
    let canvas_element = Rc::new(fetch_or_create_canvas());
    canvas_element.set_fullscreen();

    let cloned_canvas_element = canvas_element.clone();
    let resize_callback = Closure::wrap(Box::new(move |_: Event| {
        console_log!("resize");
        cloned_canvas_element.set_fullscreen();
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    resize_callback.forget();

    //======================================================
    //
    // Cavnas Listers Setup
    //
    //======================================================

    // Mouse Tracking callback
    let mousemove_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
        DATA.with(|state| {
            state.borrow_mut().mouse_pos = Coord {
                x: event.offset_x().into(),
                y: event.offset_y().into(),
            }
        })
    }) as Box<dyn FnMut(_)>);
    canvas_element
        .add_event_listener_with_callback("mousemove", mousemove_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    mousemove_callback.forget();

    // Key Pressing Callback
    let cloned_ws = ws.clone();
    let keydown_callback = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        console_log!("erer");
        let keypressed_event = ClientEvent::PlayerControlUpdate {
            press: true,
            key: event.key().to_uppercase(),
        };
        cloned_ws
            .send_with_str(&serde_json::to_string(&keypressed_event).unwrap())
            .expect("websocket sent");
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    keydown_callback.forget();

    // Key Releasing Callback
    let cloned_ws = ws.clone();
    let keyup_callback = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let keyreleased_event = ClientEvent::PlayerControlUpdate {
            press: false,
            key: event.key().to_uppercase(),
        };
        cloned_ws
            .send_with_str(&serde_json::to_string(&keyreleased_event).unwrap())
            .expect("websocket sent");
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("keyup", keyup_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    keyup_callback.forget();

    //======================================================
    // Cavnas Rendering Setup
    //======================================================
    let canvas_context = canvas_element.get_2d_context();

    let render_loop = Box::new(move || {
        DATA.with(|state| render(&canvas_element, &canvas_context, &mut state.borrow_mut()))
    });

    start_animation_loop(render_loop);
}

fn setup_logging() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}
