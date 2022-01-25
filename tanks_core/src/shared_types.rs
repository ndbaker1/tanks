use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Trait for an Object that can update on the server Tick
pub trait Tickable {
    /// Update the Object
    fn tick(&mut self) -> bool;
}

/// The number of squares in the map horizontally
pub const MAP_WIDTH: usize = 24;
/// The number of squares in the map vertically
pub const MAP_HEIGHT: usize = 12;

/// Wall objects that should be drawn and collided with
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Wall {
    Indestructable(usize),
    Desructable(usize),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerGameState {
    pub players: HashMap<String, PlayerData>,
    pub map: HashMap<(usize, usize), Wall>,
}

impl ServerGameState {
    /// Get list of mutable  bullets from players
    pub fn get_bullets_mut(&mut self) -> Vec<&mut Bullet> {
        self.players
            .iter_mut()
            .flat_map(|(_, data)| &mut data.bullets)
            .collect()
    }

    /// Get list of references bullets from players
    pub fn get_bullets(&self) -> Vec<&Bullet> {
        self.players
            .iter()
            .flat_map(|(_, data)| &data.bullets)
            .collect()
    }

    /// Get list of references to player IDsdd
    pub fn get_player_ids(&self) -> Vec<&String> {
        self.players.iter().map(|(id, _)| id).collect()
    }
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

impl Tickable for Bullet {
    fn tick(&mut self) -> bool {
        self.pos.x += self.velocity.x;
        self.pos.y += self.velocity.y;
        true
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PlayerState {
    /// Move Delay involved with shooting a bullet
    Shooting(u32),
    Idle,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerData {
    pub id: String,
    pub state: PlayerState,
    pub position: Vec2d,
    pub keys_down: HashSet<String>,
    pub bullets: Vec<Bullet>,
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
            bullets: Vec::new(),
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
