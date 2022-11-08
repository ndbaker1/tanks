use crate::utils::Vector2;

use super::constants::{BULLET_COUNT, PLAYER_SPEED};

#[derive(Debug)]
pub enum PlayerState {
    /// Stores clock in GameState ticks for shooting behavior.
    /// This is when the Tank cannot move directly after shooting.
    Shooting(u32),
    /// Default state for the player
    Idle,
}

/// Data the Server has to track for a Player Tank
#[derive(Debug)]
pub struct Player {
    /// ID of the Player, taken from Client ID
    pub id: String,
    pub alive: bool,
    /// Angle of the tanks in radians
    pub angle: f64,
    pub state: PlayerState,
    pub position: Vector2,
    /// Normalized Vector for direction of movement of the Player
    pub movement: Vector2,
    pub bullets_remaining: u8,
}

impl Player {
    pub fn physics_update(&mut self) {
        match self.state {
            PlayerState::Idle => {
                self.position = self.position.plus(&self.movement.scale(PLAYER_SPEED))
            }
            PlayerState::Shooting(ref mut dur) => {
                if *dur > 0 {
                    *dur -= 1;
                } else {
                    self.state = PlayerState::Idle;
                }
            }
        }
    }
}

impl Player {
    pub fn new(id: String) -> Self {
        Self {
            id,
            alive: true,
            angle: 0.0,
            state: PlayerState::Idle,
            position: Vector2::zero(),
            movement: Vector2::zero(),
            bullets_remaining: BULLET_COUNT,
        }
    }
}
