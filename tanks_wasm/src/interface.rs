use crate::{
    app::{handle_server_event, ClientGameState},
    audio::AUDIO,
    log,
    login::process_login_keyevent,
    socket::setup_websocket_listeners,
    utils::{fetch_or_create_canvas, js_window, Canvas, Prepared},
};
use std::{cell::RefCell, rc::Rc};
use tanks_core::{server_types::ClientEvent, shared_types::Vec2d};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, HtmlCanvasElement, KeyboardEvent, MouseEvent, WebSocket};

macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Default)]
pub struct ConnectionState {
    pub ws: Option<WebSocket>,
}

thread_local! {
  /// Global State for the Game
  ///
  /// Do not panic while using this data, otherwise you may
  /// encounter a permanent locking of the Data
  pub static GAME_STATE: RefCell<ClientGameState> = RefCell::new(ClientGameState::new(""));

  /// The state of the websocket connection
  pub static CONNECTION_STATE: RefCell<ConnectionState> = RefCell::new(ConnectionState::default());

}

/// Canvas Listeners Setup
pub fn setup_canvas() -> Rc<HtmlCanvasElement> {
    let canvas_element = Rc::new(fetch_or_create_canvas());
    canvas_element.set_fullscreen();

    // Resize Callback
    let cloned_canvas_element = canvas_element.clone();
    let resize_callback = Closure::wrap(Box::new(move |_: Event| {
        console_log!("resize");
        cloned_canvas_element.set_fullscreen();
    }) as Box<dyn FnMut(_)>);
    js_window()
        .add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    resize_callback.forget();

    canvas_element
}

/// Window Listeners Setup
///
pub fn setup_window_listeners() {
    // Mouse Tracking callback
    let mousemove_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
        GAME_STATE.with(|state| {
            state.borrow_mut().mouse_pos = Vec2d {
                x: event.offset_x().into(),
                y: event.offset_y().into(),
            }
        })
    }) as Box<dyn FnMut(_)>);
    js_window()
        .add_event_listener_with_callback("mousemove", mousemove_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    mousemove_callback.forget();

    // Mouse Click Click
    let click_callback = Closure::wrap(Box::new(move |_: MouseEvent| {
        AUDIO.with(|a| a.borrow().get("song").unwrap().play());

        CONNECTION_STATE.with(|state| {
            if let Some(ws) = &state.borrow_mut().ws {
                if ws.is_ready() {
                    let tank_shoot = ClientEvent::PlayerShoot {
                        angle: GAME_STATE.with(|state| state.borrow().get_mouse_angle()),
                    };

                    ws.send_with_str(&serde_json::to_string(&tank_shoot).unwrap())
                        .expect("websocket sent");
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    js_window()
        .add_event_listener_with_callback("mousedown", click_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    click_callback.forget();

    // Key Pressing Callback
    let keydown_callback = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        CONNECTION_STATE.with(|state| {
            let mut connection_state = state.borrow_mut();
            match &connection_state.ws {
                Some(ws) => {
                    if ws.is_ready() {
                        let keypressed_event = ClientEvent::PlayerControlUpdate {
                            press: true,
                            key: event.key().to_uppercase(),
                        };

                        ws.send_with_str(&serde_json::to_string(&keypressed_event).unwrap())
                            .expect("websocket sent");
                    }
                }
                None => {
                    if let Some((username, ws)) = process_login_keyevent(event) {
                        setup_websocket_listeners(&ws, |event| {
                            GAME_STATE
                                .with(|state| handle_server_event(event, &mut state.borrow_mut()))
                        });
                        connection_state.ws = Some(ws);
                        GAME_STATE
                            .with(|data| *data.borrow_mut() = ClientGameState::new(&username));
                    }
                }
            };
        });
    }) as Box<dyn FnMut(_)>);
    js_window()
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
    js_window()
        .add_event_listener_with_callback("keyup", keyup_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    keyup_callback.forget();
}
