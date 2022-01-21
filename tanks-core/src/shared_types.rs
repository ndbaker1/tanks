use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameData {
    pub player_data: HashMap<String, PlayerData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerData {
    pub position: Coord,
    pub direction: Option<Direction>,
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

    pub fn from_direction(dir: &Direction) -> Self {
        match dir {
            Direction::North => Coord { x: 0.0, y: -1.0 },
            Direction::NorthEast => Coord { x: 1.0, y: -1.0 },
            Direction::East => Coord { x: 1.0, y: 0.0 },
            Direction::SouthEast => Coord { x: 1.0, y: 1.0 },
            Direction::South => Coord { x: 0.0, y: 1.0 },
            Direction::SouthWest => Coord { x: -1.0, y: 1.0 },
            Direction::West => Coord { x: -1.0, y: 0.0 },
            Direction::NorthWest => Coord { x: -1.0, y: -1.0 },
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
