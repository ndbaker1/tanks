use server::game_engine;
use server::{
    data_types::{SafeClients, SafeSessions},
    notify_client, server, ServerConfig,
};
use std::env;
use std::time::Duration;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| String::from("8000"))
        .parse()
        .expect("PORT must be a number");

    env_logger::init();

    let config = ServerConfig {
        event_handler: game_engine::handle_event,
        tick_handler,
    };

    warp::serve(server(config)).run(([0, 0, 0, 0], port)).await;
}

async fn tick_handler(clients: SafeClients, sessions: SafeSessions<u32>) {
    loop {
        for session in sessions.read().await.values() {
            for (client_id, _) in &session.client_statuses {
                if let Some(client) = clients.read().await.get(client_id) {
                    notify_client(client, &String::from("sdsd"));
                }
            }
        }
        delay_for(Duration::from_millis(1000 / 60)).await;
    }
}
