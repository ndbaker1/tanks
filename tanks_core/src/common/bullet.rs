//! Definitions for the primary damaging projectile in Tanks

use std::f64::consts::PI;

use crate::utils::Vector2;

use super::constants::BULLET_SPEED;

/// Projectile shot from a Tank that will bounce off walls and destroy other Tanks (Players)
#[derive(Debug)]
pub struct Bullet {
    /// The ID of the player who created the bullet
    pub player_id: String,
    /// Bullet Position
    pub position: Vector2,
    /// Speed of the Bullet
    pub velocity: Vector2,
    /// Angle of the Bullet
    pub angle: f64,
    /// The number of wall bounces untill the Bullet explodes
    pub ricochets: u8,
}

impl Bullet {
    pub fn add_angle(&mut self, turn: f64) {
        self.angle = (self.angle + turn) % (2.0 * PI);
        self.velocity.x = BULLET_SPEED * self.angle.cos();
        self.velocity.y = BULLET_SPEED * self.angle.sin();
    }
}

impl Bullet {
    pub fn physics_update(&mut self) {
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
    }
}
