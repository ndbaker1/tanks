use crate::utils::Prepared;
use tanks_core::server_types::{ClientEvent, ServerEvent};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{MessageEvent, WebSocket};

macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Connects to the WebSocket using the username entered by the User
pub fn setup_websocket_listeners<F: Fn(ServerEvent) + 'static>(ws: &WebSocket, event_handler: F) {
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        match e.data().dyn_into::<js_sys::JsString>() {
            Ok(event_message) => match serde_json::from_str(&event_message.as_string().unwrap()) {
                Ok(event) => event_handler(event),
                Err(e) => console_log!("failed conversion into Server Event :: {}", e),
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
}
