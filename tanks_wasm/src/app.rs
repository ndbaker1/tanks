use std::collections::HashMap;
use tanks_core::{server_types::ServerEvent, shared_types::Coord};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{utils::Background, CONNECTION_STATE, GAME_STATE, USERNAME};

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
    context.set_fill_style(&"#222".into());
    context.fill_rect(0.0, 0.0, element.width().into(), element.height().into());

    let connected = CONNECTION_STATE.with(|state| match &state.borrow().ws {
        Some(ws) => ws.is_ready(),
        None => false,
    });

    match connected {
        true => render_game(element, context),
        false => render_login(element, context),
    };
}

fn render_game(element: &HtmlCanvasElement, context: &CanvasRenderingContext2d) {
    GAME_STATE.with(|state| {
        let game_state = state.borrow_mut();

        context.save();

        let (mid_width, mid_height) = (element.width() as f64 / 2.0, element.height() as f64 / 2.0);

        let player_coord = game_state.get_own_player_data();

        // drawing background color
        context
            .translate(mid_width - player_coord.x, mid_height - player_coord.y)
            .expect("failed to move camera");

        for (player, coord) in &game_state.player_data {
            context.set_fill_style(&"red".into());
            context.fill_rect(coord.x, coord.y, 40.0, 40.0);

            context.set_fill_style(&"white".into());
            context
                .fill_text(&player, coord.x, coord.y)
                .expect("text could not be drawn");
        }

        context.set_stroke_style(&"white".into());
        context.begin_path();
        context.move_to(player_coord.x, player_coord.y);
        context.line_to(
            game_state.mouse_pos.x + player_coord.x - mid_width,
            game_state.mouse_pos.y + player_coord.y - mid_height,
        );
        context.stroke();

        context.restore();
    });
}

fn render_login(element: &HtmlCanvasElement, context: &CanvasRenderingContext2d) {
    context.set_text_align("center");

    let (mid_width, mid_height) = (element.width() as f64 / 2.0, element.height() as f64 / 2.0);

    context.set_fill_style(&"white".into());
    context.set_font("32px monospace");

    context
        .fill_text("Enter a name:", mid_width, mid_height)
        .expect("text could not be drawn");

    context
        .fill_text("then press Enter", mid_width, mid_height * 1.9)
        .expect("text could not be drawn");

    USERNAME.with(|username| {
        context
            .fill_text(&username.borrow_mut(), mid_width, mid_height + 50.0)
            .expect("text could not be drawn");
    })
}
