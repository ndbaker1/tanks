use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerGameState {
    pub player_data: HashMap<String, PlayerData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerData {
    pub position: Coord,
    pub keys_down: HashSet<String>,
}

impl PlayerData {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            position: Coord { x: 0.0, y: 0.0 },
        }
    }

    pub fn move_based_on_keys(&mut self) {
        for key in &self.keys_down {
            let mut delta = match key.as_str() {
                "W" => Coord::from_direction(&Direction::North),
                "A" => Coord::from_direction(&Direction::West),
                "S" => Coord::from_direction(&Direction::South),
                "D" => Coord::from_direction(&Direction::East),
                _ => Coord { x: 0.0, y: 0.0 },
            };
            delta.scale(5.0);
            self.position.add(delta);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl Coord {
    pub fn add(&mut self, coord: Coord) {
        self.x += coord.x;
        self.y += coord.y;
    }

    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }

    pub fn from_direction(dir: &Direction) -> Self {
        match dir {
            Direction::North => Coord { x: 0.0, y: 1.0 },
            Direction::NorthEast => Coord { x: -1.0, y: 1.0 },
            Direction::East => Coord { x: -1.0, y: 0.0 },
            Direction::SouthEast => Coord { x: -1.0, y: -1.0 },
            Direction::South => Coord { x: 0.0, y: -1.0 },
            Direction::SouthWest => Coord { x: 1.0, y: -1.0 },
            Direction::West => Coord { x: 1.0, y: 0.0 },
            Direction::NorthWest => Coord { x: 1.0, y: 1.0 },
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
