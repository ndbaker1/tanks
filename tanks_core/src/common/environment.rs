use std::collections::HashMap;

use crate::utils::Vector2;

#[derive(Debug, Default)]
pub struct Environment {
    pub tiles: HashMap<Vector2, Tile>,
}

/// Ground or Wall objects that get displayed
/// and have collisions for the Tanks or the Projectiles
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Tile {
    IndestructableWall(usize),
    // Left:    health of the wall
    // Right:   height of the wall
    DesructableWall((usize, usize)),
    Empty,
}
