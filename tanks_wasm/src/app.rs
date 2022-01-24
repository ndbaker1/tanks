use crate::{
    utils::{get_window_bounds, Prepared},
    CONNECTION_STATE, GAME_STATE, USERNAME,
};
use std::{collections::HashMap, f64::consts::PI};
use tanks_core::{server_types::ServerEvent, shared_types::Vec2d};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct ClientPlayerData {}
pub struct ClientGameState {
    pub id: String,
    /// Mouse Position relative to bounds of the window
    pub mouse_pos: Vec2d,
    pub player_data: HashMap<String, Vec2d>,
    pub projectile_data: Vec<Vec2d>,
}

impl ClientGameState {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            mouse_pos: Vec2d::zero(),
            player_data: [(id.to_string(), Vec2d::zero())].into_iter().collect(),
            projectile_data: Vec::new(),
        }
    }

    pub fn update(&mut self) {}

    pub fn get_mouse_angle(&self) -> f64 {
        let camera_pos = self.get_camera_pos();
        let player_pos = self.get_own_player_data();
        let delta_x = camera_pos.x + self.mouse_pos.x - player_pos.x;
        let delta_y = camera_pos.y + self.mouse_pos.y - player_pos.y;
        (delta_y).atan2(delta_x)
    }

    /// Get the Player Data corresponding to the current player using the saved id
    pub fn get_own_player_data(&self) -> &Vec2d {
        self.player_data
            .get(&self.id)
            .expect("player did not have their own data")
    }

    /// The Camera Position for the Player
    ///
    /// This is the Top-Left coordinate
    pub fn get_camera_pos(&self) -> Vec2d {
        Vec2d::zero()
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
        ServerEvent::BulletData(bullets) => {
            game_state.projectile_data = bullets.into_iter().map(|(pos, _)| pos).collect()
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
        true => render_game(context),
        false => render_login(context),
    };
}

fn render_game(context: &CanvasRenderingContext2d) {
    GAME_STATE.with(|state| {
        let game_state = state.borrow_mut();

        context.save();

        let player_coord = game_state.get_own_player_data();

        let camera_pos = game_state.get_camera_pos();
        // drawing background color
        context
            .translate(-camera_pos.x, -camera_pos.y)
            .expect("failed to move camera");

        let player_size = 40.0;
        for (player, coord) in &game_state.player_data {
            context.set_fill_style(&"red".into());
            context.fill_rect(
                coord.x - player_size / 2.0,
                coord.y - player_size / 2.0,
                player_size,
                player_size,
            );

            context.set_fill_style(&"white".into());
            context
                .fill_text(&player, coord.x, coord.y)
                .expect("text could not be drawn");
        }

        for bullet in &game_state.projectile_data {
            context.set_fill_style(&"grey".into());
            context.begin_path();
            context
                .arc(bullet.x, bullet.y, 5.0, 0.0, 2.0 * PI)
                .expect("bullet could not be drawn");
            context.fill();
        }

        context.set_stroke_style(&"white".into());
        context.begin_path();
        context.move_to(player_coord.x, player_coord.y);
        context.line_to(
            camera_pos.x + game_state.mouse_pos.x,
            camera_pos.y + game_state.mouse_pos.y,
        );
        context.stroke();

        context.restore();
    });
}

fn render_login(context: &CanvasRenderingContext2d) {
    context.set_text_align("center");

    let bounds = get_window_bounds();
    let (mid_width, mid_height) = (bounds.x / 2.0, bounds.y / 2.0);

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
