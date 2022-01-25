use std::{cell::RefCell, rc::Rc};
use tanks_core::shared_types::{Vec2d, MAP_HEIGHT, MAP_WIDTH};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebSocket};

pub const WEBSOCKET_PATH: &str = "api/ws";

pub fn js_window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    js_window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn document() -> web_sys::Document {
    js_window()
        .document()
        .expect("should have a document on window")
}

pub fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

pub fn fetch_or_create_canvas() -> web_sys::HtmlCanvasElement {
    body()
        .query_selector("canvas")
        .expect("query tries to fetch element")
        .unwrap_or_else(|| {
            let canvas = document()
                .create_element("canvas")
                .expect("document should create canvas");

            body()
                .append_child(&canvas)
                .expect("added canvas to the DOM");

            canvas
        })
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("element is canvas element")
}

/// Trait that allows for a resize of an Element to fullscreen
pub trait Canvas {
    /// Sets the Element to the full size of the window
    fn set_fullscreen(&self);
    fn add_js_listener(&self, event: &str, func: Box<dyn FnMut()>);
    fn get_2d_context(&self) -> CanvasRenderingContext2d;
}

pub fn get_window_bounds() -> Vec2d {
    Vec2d {
        x: js_window()
            .inner_width()
            .expect("valid window width")
            .as_f64()
            .unwrap_or_default(),
        y: js_window()
            .inner_height()
            .expect("valid window height")
            .as_f64()
            .unwrap_or_default(),
    }
}

impl Canvas for HtmlCanvasElement {
    fn set_fullscreen(&self) {
        let bounds = get_window_bounds();
        self.set_width(bounds.x as u32);
        self.set_height(bounds.y as u32);
    }

    fn add_js_listener(&self, event: &str, func: Box<dyn FnMut()>) {
        let onmousemove = Closure::wrap(func);

        self.add_event_listener_with_callback(event, onmousemove.as_ref().unchecked_ref())
            .expect("added mouse listenter");

        onmousemove.forget();
    }

    fn get_2d_context(&self) -> CanvasRenderingContext2d {
        self.get_context("2d")
            .expect("canvas has 2d context")
            .expect("valid context")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("root canvas")
    }
}

pub fn start_animation_loop(mut draw_call: Box<dyn FnMut()>) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let closure = Closure::wrap(Box::new(move || {
        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        draw_call();
        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>);

    *g.borrow_mut() = Some(closure);

    request_animation_frame(g.borrow().as_ref().unwrap());
}

/// Fetch the appropriate websocket uri according to the protocol of the url
pub fn get_websocket_uri(username: &str) -> String {
    let websocket_protocol = match js_window()
        .location()
        .protocol()
        .expect("no valid protocol for url")
        .contains("https")
    {
        true => "wss",
        false => "ws",
    };

    format!(
        "{}://{}/{}/{}",
        websocket_protocol,
        js_window()
            .location()
            .host()
            .expect("no valid host for url"),
        WEBSOCKET_PATH,
        username,
    )
}

/// Trait for an object that can be checked whether it is ready to use
pub trait Prepared {
    fn is_ready(&self) -> bool;
}

impl Prepared for WebSocket {
    /// More readable way of checking that the READY_STATE of the websocket is OPEN
    fn is_ready(&self) -> bool {
        self.ready_state() == 1
    }
}

pub fn get_block_size() -> f64 {
    let bounds = get_window_bounds();
    (bounds.x / MAP_WIDTH as f64).min(bounds.y / MAP_HEIGHT as f64)
}
