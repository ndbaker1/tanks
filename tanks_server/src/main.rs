use std::env;
use websocket_server::{server, ServerConfig};

pub mod tanks;
pub mod utils;

#[tokio::main]
async fn main() {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| String::from("8000"))
        .parse()
        .expect("PORT must be a number");

    // initialize env_logging backend for logging
    env_logger::init();

    // Pass handlers for the server into the ServerConfig to get them initialized with the application
    let server_config = ServerConfig {
        tick_server: true,
        tick_handler: tanks::tick_handler,
        message_handler: tanks::handle_event,
    };

    warp::serve(server(server_config))
        .run(([0, 0, 0, 0], port))
        .await;
}
