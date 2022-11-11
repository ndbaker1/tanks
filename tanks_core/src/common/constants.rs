//! Up front configuration values

/// The number of squares in the map horizontally
pub const MAP_BLOCK_WIDTH: usize = 22;
/// The number of squares in the map vertically
pub const MAP_BLOCK_HEIGHT: usize = 17;

/// The Relative Size for a Bullet compared to a map block
pub const BULLET_RADIUS: f64 = 0.12;
/// Max number of bullets the player have out at once
pub const BULLET_COUNT: u8 = 5;
/// Speed of the Bullet relative to the map size
pub const BULLET_SPEED: f64 = 0.12;

/// Speed of the Player relative to the map size
pub const PLAYER_RADIUS: f64 = 0.4;
/// Speed of the Player relative to the map size
pub const PLAYER_SPEED: f64 = 0.08;
