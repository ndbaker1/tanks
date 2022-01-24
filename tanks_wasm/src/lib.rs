use app::{handle_server_event, render, ClientGameState};
use std::panic;
use std::{cell::RefCell, rc::Rc};
use tanks_core::{
    server_types::{ClientEvent, ServerEvent},
    shared_types::Coord,
};
use utils::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, HtmlCanvasElement, KeyboardEvent, MessageEvent, MouseEvent, WebSocket};

pub mod app;
mod utils;

#[derive(Default)]
pub struct ConnectionState {
    ws: Option<WebSocket>,
}

thread_local! {
    /// Global State for the Game
    ///
    /// Do not panic while using this data, otherwise you may
    /// encounter a permanent locking of the Data
    static GAME_STATE: RefCell<ClientGameState> = RefCell::new(ClientGameState::new(""));

    static CONNECTION_STATE: RefCell<ConnectionState> = RefCell::new(ConnectionState::default());
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
#[wasm_bindgen(start)]
pub fn start() {
    setup_logging();

    let canvas_element = setup_canvas();
    setup_window_listeners();

    let draw_proc = move || render(&canvas_element, &canvas_element.get_2d_context());
    start_animation_loop(Box::new(draw_proc));
}

fn setup_logging() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

/// Connects to the WebSocket using the username entered by the User
fn connect_to_server() {
    let username = get_query_params()
        .get("username")
        .expect("no username given in query params")
        .clone();

    let ws = WebSocket::new(&get_websocket_uri(&username)).expect("failed to connect to websocket");

    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        match e.data().dyn_into::<js_sys::JsString>() {
            Ok(event_message) => match serde_json::from_str(&event_message.as_string().unwrap()) {
                Ok(event) => {
                    GAME_STATE.with(|state| handle_server_event(event, &mut state.borrow_mut()))
                }
                Err(e) => console_log!("failed convserion into Server Event :: {}", e),
            },
            Err(_) => console_log!("what is that event? => {:#?}", e.data()),
        }
    }) as Box<dyn FnMut(_)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    // Signal the server to place the client into a session
    let cloned_ws = ws.clone();
    let onopen_callback = Closure::wrap(Box::new(move |_: MessageEvent| {
        if cloned_ws.is_ready() {
            cloned_ws
                .send_with_str(&serde_json::to_string(&ClientEvent::JoinSession).unwrap())
                .unwrap();
        }
    }) as Box<dyn FnMut(_)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onopen_callback.forget();

    // hold websocket reference
    CONNECTION_STATE.with(|state| state.borrow_mut().ws = Some(ws));
    // update game state
    GAME_STATE.with(|data| *data.borrow_mut() = ClientGameState::new(&username));
}

/// Canvas Listeners Setup
fn setup_canvas() -> Rc<HtmlCanvasElement> {
    let canvas_element = Rc::new(fetch_or_create_canvas());
    canvas_element.set_fullscreen();

    // Resize Callback
    let cloned_canvas_element = canvas_element.clone();
    let resize_callback = Closure::wrap(Box::new(move |_: Event| {
        console_log!("resize");
        cloned_canvas_element.set_fullscreen();
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    resize_callback.forget();

    canvas_element
}

/// Window Listeners Setup
///
fn setup_window_listeners() {
    // Mouse Tracking callback
    let mousemove_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
        GAME_STATE.with(|state| {
            state.borrow_mut().mouse_pos = Coord {
                x: event.offset_x().into(),
                y: event.offset_y().into(),
            }
        })
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("mousemove", mousemove_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    mousemove_callback.forget();

    // Key Pressing Callback
    let keydown_callback = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.key() == "Enter" {
            connect_to_server();
        }

        let keypressed_event = ClientEvent::PlayerControlUpdate {
            press: true,
            key: event.key().to_uppercase(),
        };
        CONNECTION_STATE.with(|state| {
            if let Some(ws) = &state.borrow().ws {
                if ws.is_ready() {
                    ws.send_with_str(&serde_json::to_string(&keypressed_event).unwrap())
                        .expect("websocket sent");
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    keydown_callback.forget();

    // Key Releasing Callback
    let keyup_callback = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let keyreleased_event = ClientEvent::PlayerControlUpdate {
            press: false,
            key: event.key().to_uppercase(),
        };
        CONNECTION_STATE.with(|state| {
            if let Some(ws) = &state.borrow().ws {
                if ws.is_ready() {
                    ws.send_with_str(&serde_json::to_string(&keyreleased_event).unwrap())
                        .expect("websocket sent");
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    window()
        .add_event_listener_with_callback("keyup", keyup_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    keyup_callback.forget();
}
