use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Trait for an Object that can update on the server Tick
pub trait Tick {
    fn tick(&mut self) -> bool;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerGameState {
    pub players: HashMap<String, PlayerData>,
    pub bullets: Vec<Bullet>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Bullet {
    /// Bullet Position
    pub pos: Vec2d,
    /// Speed of the Bullet
    pub velocity: Vec2d,
    /// Angle of the Bullet
    pub angle: f64,
}

impl Tick for Bullet {
    fn tick(&mut self) -> bool {
        self.pos.x += self.velocity.x;
        self.pos.y += self.velocity.y;
        true
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerData {
    pub position: Vec2d,
    pub keys_down: HashSet<String>,
}

impl PlayerData {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            position: Vec2d::zero(),
        }
    }

    pub fn move_based_on_keys(&mut self) {
        for key in &self.keys_down {
            let mut delta = match key.as_str() {
                "W" | "ARROWUP" => Vec2d::from_direction(&Direction::North),
                "A" | "ARROWLEFT" => Vec2d::from_direction(&Direction::West),
                "S" | "ARROWDOWN" => Vec2d::from_direction(&Direction::South),
                "D" | "ARROWRIGHT" => Vec2d::from_direction(&Direction::East),
                _ => Vec2d::zero(),
            };
            delta.scale(5.0);
            self.position.add(delta);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn add(&mut self, coord: Vec2d) {
        self.x += coord.x;
        self.y += coord.y;
    }

    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
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
