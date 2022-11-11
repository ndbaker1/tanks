use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    extract::ws::Message,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Extension, Router,
};
use futures::SinkExt;
use tanks_core::common::{bullet::Bullet, gamestate::GameState, player::Player};
use tanks_events::{BulletWrapper, ServerEvent, TankWrapper};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing::{info, Level};

use crate::state::SharedServerState;

mod state;
mod tanks;
mod ws;

#[derive(Default)]
pub struct SessionContainer {
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

    let state_copy = state.clone();
    tokio::spawn(async move {
        loop {
            for (_, session) in state_copy.sessions.lock().await.iter_mut() {
                let mut gamestate = session.data.gamestate.lock().await;
                gamestate.tick();
                // collect stats to deliver to clients

                let broadcast_data = convert_gamestate_to_broadcast(&gamestate);

                drop(gamestate);

                // broadcast to all clients
                for client_id in session.active_client_set() {
                    state_copy
                        .clients
                        .lock()
                        .await
                        .get_mut(client_id)
                        .unwrap()
                        .sender
                        .lock()
                        .await
                        .send(Message::Text(
                            serde_json::to_string(&broadcast_data).unwrap(),
                        ))
                        .await
                        .unwrap();
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs_f64(1.0 / 60.0)).await;
        }
    });

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/ws", get(ws::websocket_handler))
        .fallback(
            get_service(ServeDir::new("dist")).handle_error(|error| async move {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{error}"))
            }),
        )
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
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

fn convert_gamestate_to_broadcast(gs: &GameState) -> ServerEvent {
    ServerEvent::GameState {
        bullets: gs
            .bullets
            .iter()
            .map(
                |&Bullet {
                     angle, position, ..
                 }| BulletWrapper { angle, position },
            )
            .collect(),
        tanks: gs
            .players
            .iter()
            .map(
                |(
                    _,
                    Player {
                        id,
                        angle,
                        position,
                        movement,
                        ..
                    },
                )| TankWrapper {
                    angle: *angle,
                    id: id.clone(),
                    movement: *movement,
                    position: *position,
                },
            )
            .collect(),
    }
}
