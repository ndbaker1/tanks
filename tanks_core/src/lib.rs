pub mod map;
pub mod server_types;
pub mod shared_types;

/// The number of squares in the map horizontally
pub const MAP_WIDTH: usize = 24;
/// The number of squares in the map vertically
pub const MAP_HEIGHT: usize = 12;

/// The Relative Size for a Bullet
pub const BULLET_SIZE: f64 = 0.2;
/// Max number of bullets the player have out at once
pub const BULLET_COUNT: u8 = 5;
/// Speed of the Bullet relative to the map size
pub const BULLET_SPEED: f64 = 0.2;

/// Speed of the Player relative to the map size
pub const PLAYER_SPEED: f64 = 0.12;
