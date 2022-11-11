use crate::{
    interface::{CONNECTION_STATE, GAME_STATE},
    login::render_login,
    utils::{get_block_size, Prepared},
};
use std::{collections::HashMap, f64::consts::PI};
use tanks_core::{
    common::{
        constants::{BULLET_RADIUS, MAP_BLOCK_HEIGHT, MAP_BLOCK_WIDTH},
        environment::{Environment, Tile},
    },
    utils::Vector2,
};
use tanks_events::ServerEvent;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct ClientPlayerData {}

pub struct ClientGameState {
    pub id: String,
    /// Mouse Position relative to bounds of the window
    pub mouse_pos: Vector2,
    pub player_data: HashMap<String, Vector2>,
    pub projectile_data: Vec<Vector2>,
    pub map_landmarks: Environment,
}

impl ClientGameState {
    pub fn new(id: &str) -> Self {
        Self {
            id: String::from(id),
            mouse_pos: Vector2::zero(),
            player_data: [(String::from(id), Vector2::zero())].into_iter().collect(),
            projectile_data: Vec::new(),
            map_landmarks: Environment::default(),
        }
    }

    /// Gets the angle of the player to the mouse
    pub fn get_mouse_angle(&self) -> f64 {
        let player_pos = self.get_own_player_data();
        let delta_x = self.mouse_pos.x - player_pos.x;
        let delta_y = self.mouse_pos.y - player_pos.y;
        (delta_y).atan2(delta_x)
    }

    /// Get the Player Data corresponding to the current player using the saved id
    pub fn get_own_player_data(&self) -> &Vector2 {
        self.player_data
            .get(&self.id)
            .expect("player did not have their own data")
    }

    /// The Camera Position for the Player
    ///
    /// This is the Top-Left coordinate
    pub fn get_camera_pos(&self) -> Vector2 {
        Vector2::zero()
    }
}

pub fn handle_server_event(event: ServerEvent, game_state: &mut ClientGameState) {
    match event {
        ServerEvent::GameState { bullets, tanks } => {
            // either update the player or add them
            for tank in tanks {
                if let Some(player_data) = game_state.player_data.get_mut(&tank.id) {
                    *player_data = tank.position.scale(get_block_size());
                } else {
                    game_state.player_data.insert(tank.id, tank.position);
                }
            }

            game_state.projectile_data = bullets
                .into_iter()
                .map(|pos| pos.position.scale(get_block_size()))
                .collect()
        }
        ServerEvent::PlayerDisconnect { player } => {
            game_state.player_data.remove(&player);
        }
        ServerEvent::BulletExplode(_) => {
            // display some kind of animation to show that
            // the bullets exploded at these coordinates
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

    if connected {
        GAME_STATE.with(|state| render_game(context, &state.borrow()))
    } else {
        render_login(context)
    }
}

fn render_game(context: &CanvasRenderingContext2d, game_state: &ClientGameState) {
    context.save();

    let player_coord = game_state.get_own_player_data();

    let block_size = get_block_size();

    let colors = ["#5C6784", "#1D263B"];
    for col in 0..MAP_BLOCK_WIDTH {
        for row in 0..MAP_BLOCK_HEIGHT {
            context.set_fill_style(&colors[(col + row).rem_euclid(2)].into());

            if let Some(tile) = game_state.map_landmarks.tiles.get(&(row, col)) {
                match tile {
                    Tile::Empty => context.set_fill_style(&"grey".into()),
                    Tile::DesructableWall(_) => context.set_fill_style(&"orange".into()),
                    Tile::IndestructableWall(_) => context.set_fill_style(&"brown".into()),
                }
            }

            context.fill_rect(
                block_size * col as f64,
                block_size * row as f64,
                block_size,
                block_size,
            );
        }
    }

    for (player, coord) in &game_state.player_data {
        context.set_fill_style(&"red".into());
        context.fill_rect(
            coord.x - block_size / 2.0,
            coord.y - block_size / 2.0,
            block_size,
            block_size,
        );

        // I think we want this to be a fixed pixel size so that you can always see the name
        context.set_font("40px monospace");
        context.set_text_align("center");

        context.set_fill_style(&"white".into());
        context
            .fill_text(player, coord.x, coord.y)
            .expect("text could not be drawn");
    }

    for bullet in &game_state.projectile_data {
        context.set_fill_style(&"grey".into());
        context.begin_path();
        context
            .arc(
                bullet.x,
                bullet.y,
                block_size * BULLET_RADIUS,
                0.0,
                2.0 * PI,
            )
            .expect("bullet could not be drawn");
        context.fill();
    }

    context.set_stroke_style(&"white".into());
    context.begin_path();
    context.move_to(player_coord.x, player_coord.y);
    context.line_to(game_state.mouse_pos.x, game_state.mouse_pos.y);
    context.stroke();

    context.restore();
}
