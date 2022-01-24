use std::collections::HashMap;
use tanks_core::{server_types::ServerEvent, shared_types::Coord};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{CONNECTION_STATE, GAME_STATE};

pub struct ClientGameState {
    pub id: String,
    pub mouse_pos: Coord,
    pub player_data: HashMap<String, Coord>,
    pub projectile_data: Vec<Coord>,
}

impl ClientGameState {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            mouse_pos: Coord { x: 0.0, y: 0.0 },
            player_data: {
                let mut data = HashMap::new();
                data.insert(id.to_string(), Coord { x: 0.0, y: 0.0 });
                data
            },
            projectile_data: Vec::new(),
        }
    }

    pub fn update(&mut self) {}

    #[allow(dead_code)]
    fn delta(&mut self) -> f64 {
        10.0
    }

    /// Get the Player Data corresponding to the current player using the saved id
    pub fn get_own_player_data(&self) -> &Coord {
        self.player_data
            .get(&self.id)
            .expect("player did not have their own ")
    }
}

pub fn handle_server_event(event: ServerEvent, game_state: &mut ClientGameState) {
    match event {
        ServerEvent::PlayerPosUpdate { coord, player } => {
            // either update the player or add them
            if let Some(player_data) = game_state.player_data.get_mut(&player) {
                *player_data = coord;
            } else {
                game_state.player_data.insert(player, coord);
            }
        }
        ServerEvent::PlayerDisconnect { player } => {
            game_state.player_data.remove(&player);
        }
    }
}

pub fn render(element: &HtmlCanvasElement, context: &CanvasRenderingContext2d) {
    GAME_STATE.with(|state| {
        let game_state = state.borrow_mut();

        context.save();

        let (mid_width, mid_height) = (element.width() as f64 / 2.0, element.height() as f64 / 2.0);

        let player_coord = game_state.get_own_player_data();

        // drawing background color
        context
            .translate(mid_width - player_coord.x, mid_height - player_coord.y)
            .expect("failed to move camera");
        context.set_fill_style(&"#222".into());
        context.fill_rect(
            player_coord.x - mid_width,
            player_coord.y - mid_height,
            element.width().into(),
            element.height().into(),
        );

        for (player, coord) in &game_state.player_data {
            context.set_fill_style(&"red".into());
            context.fill_rect(coord.x, coord.y, 40.0, 40.0);

            context.set_fill_style(&"white".into());
            context.set_font("50px serif");
            context
                .fill_text(&player, coord.x, coord.y)
                .expect("text could not be drawn");
        }

        context.set_stroke_style(&"white".into());
        context.begin_path();
        context.move_to(
            game_state.get_own_player_data().x,
            game_state.get_own_player_data().y,
        );
        context.line_to(game_state.mouse_pos.x, game_state.mouse_pos.y);
        context.stroke();

        context.restore();
    });
}
