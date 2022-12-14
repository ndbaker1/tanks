use crate::utils::Vector2;

use super::constants::{BULLET_COUNT, PLAYER_SPEED};

#[derive(Debug)]
pub enum TankState {
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
    pub state: TankState,
    pub position: Vector2,
    /// Angle of the tank gun
    pub gun_angle: f64,
    /// Angle the player wants to move towards
    pub movement_dir: f64,
    /// Normalized Vector for direction of movement of the Player
    pub movement: Vector2,
    pub bullets_remaining: u8,
}

impl Player {
    pub const TURN_RATE: f64 = 0.01;
    pub fn physics_update(&mut self) {
        match self.state {
            TankState::Idle => {
                // determine which way to rotate the movement based on the
                // difference between the current movement and the desired movement direction
                // TODO
                // self.movement = self.movement.rotate(
                //     Self::TURN_RATE
                //         * (self.movement_dir.cos() * self.movement.x
                //             + self.movement_dir.sin() * self.movement.y)
                //             .signum(),
                // );

                // move the player by the movement vector
                self.position = self.position.plus(&self.movement.scale(PLAYER_SPEED))
            }
            TankState::Shooting(ref mut dur) => {
                if *dur > 0 {
                    *dur -= 1;
                } else {
                    self.state = TankState::Idle;
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
            gun_angle: 0.0,
            state: TankState::Idle,
            position: Vector2::zero(),
            movement_dir: 0.0,
            movement: Vector2::zero(),
            bullets_remaining: BULLET_COUNT,
        }
    }
}
