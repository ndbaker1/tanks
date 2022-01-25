use crate::utils::{get_websocket_uri, get_window_bounds};
use regex::Regex;
use std::cell::RefCell;
use web_sys::{CanvasRenderingContext2d, KeyboardEvent, WebSocket};

thread_local! {
  /// Editable Text for username
  pub static USERNAME: RefCell<String> = RefCell::new(String::new());
  /// Regex Match for valid usernames
  static TEXT_MATCHER: Regex = Regex::new(r"^[a-zA-Z0-9]$").unwrap();
}

pub fn process_login_keyevent(event: KeyboardEvent) -> Option<(String, WebSocket)> {
    USERNAME.with(|username| {
        let mut username = username.borrow_mut();
        let key = event.key();
        match key.as_str() {
            "Enter" => {
                let ws = WebSocket::new(&get_websocket_uri(&username))
                    .expect("failed to connect to websocket");

                return Some((username.clone(), ws));
            }
            "Backspace" => {
                username.pop();
            }
            _ => match TEXT_MATCHER.with(|re| re.is_match(&key)) {
                true => username.push_str(&event.key()),
                false => {}
            },
        }

        None
    })
}

pub fn render_login(context: &CanvasRenderingContext2d) {
    context.set_text_align("center");

    let bounds = get_window_bounds();
    let (mid_width, mid_height) = (bounds.x / 2.0, bounds.y / 2.0);

    context.set_fill_style(&"white".into());
    context.set_font("32px monospace");

    context
        .fill_text("Enter a name:", mid_width, mid_height)
        .expect("text could not be drawn");

    context.set_font("18px monospace");
    context
        .fill_text("then press Enter", mid_width, mid_height * 1.9)
        .expect("text could not be drawn");

    USERNAME.with(|username| {
        context.set_font("32px monospace");
        context
            .fill_text(&username.borrow_mut(), mid_width, mid_height + 50.0)
            .expect("text could not be drawn");
    })
}
