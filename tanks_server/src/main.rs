use std::{env, net::SocketAddr, sync::Arc};

use axum::{response::IntoResponse, routing::get, Extension, Router};
use tanks_core::common::gamestate::GameState;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{info, Level};

use crate::state::SharedServerState;

mod state;
mod tanks;
mod ws;

#[derive(Default)]
pub struct SessionContainer {
    task_handles: Vec<JoinHandle<()>>,
    gamestate: Arc<Mutex<GameState>>,
}

type SessionData = SessionContainer;

#[tokio::main]
async fn main() {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| String::from("8000"))
        .parse()
        .expect("PORT must be a number");

    let state = SharedServerState::<SessionData>::default();

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/ws", get(ws::websocket_handler))
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Health Check Endpoint used to verify the service is live
async fn health_handler() -> impl IntoResponse {
    info!("HEALTH_CHECK ✓");
    "health check ✓".into_response()
}
