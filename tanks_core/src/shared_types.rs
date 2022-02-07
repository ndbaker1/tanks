use crate::{BULLET_COUNT, BULLET_SIZE, BULLET_SPEED, MAP_HEIGHT, MAP_WIDTH, PLAYER_SPEED};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    f64::consts::PI,
};

/// Trait for an Object that can update on the server Tick
pub trait Tickable {
    /// Update the Object
    fn tick(&mut self) -> bool;
}

/// Wall objects that should be drawn and collided with
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Tile {
    Indestructable(usize),
    Desructable(usize),
}

pub type MapLandmarks = HashMap<usize, HashMap<usize, Tile>>;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ServerGameState {
    pub players: HashMap<String, PlayerData>,
    pub map: MapLandmarks,
    pub bullets: Vec<Bullet>,
}

impl ServerGameState {
    /// Get list of references to player ID
    pub fn get_player_ids(&self) -> Vec<&String> {
        self.players.iter().map(|(id, _)| id).collect()
    }
}

/// Projectile shot from a Tank that will bounce off walls and destroy other Tanks (Players)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Bullet {
    /// The ID of the player who created the bullet
    pub player_id: String,
    /// Bullet Position
    pub pos: Vec2d,
    /// Speed of the Bullet
    pub velocity: Vec2d,
    /// Angle of the Bullet
    pub angle: f64,
    /// The number of wall bounces untill the Bullet explodes
    pub bounces: u8,
}

impl Bullet {
    pub fn collides_with(&self, other: &Bullet) -> bool {
        let delta = Vec2d {
            x: self.pos.x - other.pos.x,
            y: self.pos.y - other.pos.y,
        };

        delta.x * delta.x + delta.y * delta.y < BULLET_SIZE * BULLET_SIZE
    }

    pub fn collide_bounds(&mut self) -> bool {
        let collides_x = self.pos.x > MAP_WIDTH as f64 || self.pos.x < 0.0;
        let collides_y = self.pos.y > MAP_HEIGHT as f64 || self.pos.y < 0.0;

        if collides_x || collides_y {
            if collides_x {
                self.velocity.x *= -(collides_x as i8) as f64;
            }
            if collides_y {
                self.velocity.y *= -(collides_y as i8) as f64;
            }

            let has_bounces = self.bounces > 0;
            if has_bounces {
                self.bounces -= 1;
            }

            !has_bounces
        } else {
            false
        }
    }

    pub fn add_angle(&mut self, turn: f64) {
        self.angle = (self.angle + turn) % (2.0 * PI);
        self.velocity.x = BULLET_SPEED * self.angle.cos();
        self.velocity.y = BULLET_SPEED * self.angle.sin();
    }
}

impl Tickable for Bullet {
    fn tick(&mut self) -> bool {
        self.pos.x += self.velocity.x;
        self.pos.y += self.velocity.y;
        true
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PlayerState {
    /// Player movement delay after shooting a bullet
    Shooting(u32),
    Idle,
}

/// Data the Server has to track for a Player Tank
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerData {
    pub id: String,
    pub state: PlayerState,
    pub position: Vec2d,
    pub keys_down: HashSet<String>,
    pub bullets_left: u8,
}

impl Tickable for PlayerData {
    fn tick(&mut self) -> bool {
        if let PlayerState::Idle = self.state {
            if !self.keys_down.is_empty() {
                self.move_based_on_keys();
                return true;
            }
        } else if let PlayerState::Shooting(dur) = &mut self.state {
            if dur > &mut 0 {
                *dur -= 1;
            } else {
                self.state = PlayerState::Idle;
            }
        }

        false
    }
}

impl PlayerData {
    pub fn new(id: &str) -> Self {
        Self {
            id: String::from(id),
            state: PlayerState::Idle,
            keys_down: HashSet::new(),
            position: Vec2d::zero(),
            bullets_left: BULLET_COUNT,
        }
    }

    pub fn move_based_on_keys(&mut self) {
        let mut delta = Vec2d::zero();
        for key in &self.keys_down {
            match key.as_str() {
                "W" | "ARROWUP" => {
                    delta.add(&Vec2d::from_direction(&Direction::North));
                }
                "A" | "ARROWLEFT" => {
                    delta.add(&Vec2d::from_direction(&Direction::West));
                }
                "S" | "ARROWDOWN" => {
                    delta.add(&Vec2d::from_direction(&Direction::South));
                }
                "D" | "ARROWRIGHT" => {
                    delta.add(&Vec2d::from_direction(&Direction::East));
                }
                _ => {}
            };
        }

        self.position.add(delta.normalize().scale(PLAYER_SPEED));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

impl Vec2d {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Vec2d {
    pub fn add(&mut self, coord: &Vec2d) -> &mut Self {
        self.x += coord.x;
        self.y += coord.y;
        self
    }

    pub fn scale(&mut self, factor: f64) -> &mut Self {
        self.x *= factor;
        self.y *= factor;
        self
    }

    pub fn normalize(&mut self) -> &mut Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            self.x /= mag;
            self.y /= mag;
        }
        self
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn from_direction(dir: &Direction) -> Self {
        match dir {
            Direction::North => Vec2d { x: 0.0, y: -1.0 },
            Direction::NorthEast => Vec2d { x: 1.0, y: -1.0 },
            Direction::East => Vec2d { x: 1.0, y: 0.0 },
            Direction::SouthEast => Vec2d { x: 1.0, y: 1.0 },
            Direction::South => Vec2d { x: 0.0, y: 1.0 },
            Direction::SouthWest => Vec2d { x: -1.0, y: 1.0 },
            Direction::West => Vec2d { x: -1.0, y: 0.0 },
            Direction::NorthWest => Vec2d { x: -1.0, y: -1.0 },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}
