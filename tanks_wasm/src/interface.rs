use crate::{
    app::{handle_server_event, ClientGameState},
    log,
    login::process_login_keyevent,
    socket::setup_websocket_listeners,
    utils::{fetch_or_create_canvas, js_window, Canvas, Prepared},
};
use std::{cell::RefCell, rc::Rc};
use tanks_core::utils::Vector2;
use tanks_events::ClientEvent;
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
pub fn setup_window_listeners() {
    // Mouse Tracking callback
    let mousemove_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
        let pos = Vector2 {
            x: event.offset_x().into(),
            y: event.offset_y().into(),
        };

        GAME_STATE.with(|state| {
            state.borrow_mut().mouse_pos = pos;
            let state = state.borrow();
            let player_pos = state.get_own_player_data();

            CONNECTION_STATE.with(|state| {
                if let Some(ws) = &state.borrow_mut().ws {
                    if ws.is_ready() {
                        ws.send_with_str(
                            &serde_json::to_string(&ClientEvent::AimUpdate {
                                angle: (pos.y - player_pos.y).atan2(pos.x - player_pos.x),
                            })
                            .unwrap(),
                        )
                        .expect("websocket sent");
                    }
                }
            });
        });
    }) as Box<dyn FnMut(_)>);
    js_window()
        .add_event_listener_with_callback("mousemove", mousemove_callback.as_ref().unchecked_ref())
        .expect("failed to add listener");
    mousemove_callback.forget();

    // Mouse Click Click
    let click_callback = Closure::wrap(Box::new(move |_: MouseEvent| {
        CONNECTION_STATE.with(|state| {
            if let Some(ws) = &state.borrow_mut().ws {
                if ws.is_ready() {
                    ws.send_with_str(&serde_json::to_string(&ClientEvent::Shoot).unwrap())
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
                        let movement_update = ClientEvent::MovementUpdate {
                            direction: GAME_STATE.with(|gstate| {
                                gstate.borrow_mut().keysdown.insert(event.key());
                                gstate
                                    .borrow()
                                    .keysdown
                                    .iter()
                                    .fold(Vector2::zero(), |acc, cur| acc.plus(&from(cur)))
                                    .normalize()
                            }),
                        };

                        ws.send_with_str(&serde_json::to_string(&movement_update).unwrap())
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
        CONNECTION_STATE.with(|state| {
            if let Some(ws) = &state.borrow().ws {
                if ws.is_ready() {
                    let keyreleased_event = ClientEvent::MovementUpdate {
                        direction: GAME_STATE.with(|gstate| {
                            gstate.borrow_mut().keysdown.remove(&event.key());
                            gstate
                                .borrow()
                                .keysdown
                                .iter()
                                .fold(Vector2::zero(), |acc, cur| acc.plus(&from(cur)))
                                .normalize()
                        }),
                    };

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

fn from(dir: &str) -> Vector2 {
    match dir.to_uppercase().as_str() {
        "W" => Vector2 { x: 0.0, y: -1.0 },
        "D" => Vector2 { x: 1.0, y: 0.0 },
        "S" => Vector2 { x: 0.0, y: 1.0 },
        "A" => Vector2 { x: -1.0, y: 0.0 },

        _ => Vector2::zero(),
    }
}
