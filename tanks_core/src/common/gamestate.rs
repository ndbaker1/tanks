use std::collections::{BTreeSet, HashMap};

use crate::utils::{circle_circle_collision, circle_rect_collision, Vector2};

use super::{
    bullet::Bullet,
    constants::{BULLET_RADIUS, BULLET_SPEED, MAP_BLOCK_HEIGHT, MAP_BLOCK_WIDTH, PLAYER_RADIUS},
    environment::{Environment, Tile},
    player::{Player, TankState},
};

#[derive(Debug, Default)]
pub struct GameState {
    pub players: HashMap<String, Player>,
    pub bullets: Vec<Bullet>,
    pub environment: Environment,
}

/// Implementations for every sensical action that can be taken during the game
impl GameState {
    pub fn set_player_movement(&mut self, player_id: &str, radial: &Vector2) {
        if let Some(player) = self.players.get_mut(player_id) {
            // normal all points where the movement can be reassigned
            player.movement = radial.normalize();
        }
    }

    pub fn set_player_angle(&mut self, player_id: &str, angle: f64) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.angle = angle;
        }
    }

    /// Spawn a bullet from the player who invoked the shoot action
    /// Uses the angle and position of the indicated player
    pub fn player_shoot(&mut self, player_id: &str) {
        let Some(player) = self.players.get_mut(player_id) else {
            todo!() // trying to move a player thats not in the game?
        };

        if matches!(player.state, TankState::Idle) && player.bullets_remaining > 0 {
            // decrement available bullets
            player.bullets_remaining -= 1;

            let velocity = Vector2 {
                x: BULLET_SPEED * player.angle.cos(),
                y: BULLET_SPEED * player.angle.sin(),
            };

            self.bullets.push(Bullet {
                velocity,
                ricochets: 1,
                angle: player.angle,
                position: player.position,
                player_id: player.id.clone(),
            });

            player.state = TankState::Shooting(5);
        }
    }
}

/// Utility methods
impl GameState {
    /// Get list of references to player IDs
    pub fn get_client_ids(&self) -> Vec<&String> {
        self.players.iter().map(|(id, _)| id).collect()
    }
}

impl GameState {
    pub fn tick(&mut self) {
        // update physics steps for entities
        self.bullets.iter_mut().for_each(Bullet::physics_update);
        self.players
            .iter_mut()
            .map(|(_, player)| player)
            .for_each(Player::physics_update);

        // Process bullet collisions which results in the bullet exploding or dissappearing.
        // These interactions should also result in a reallocation of bullet shots to the
        // player who created the bullet.

        // collisions of bullets with players
        for player in self
            .players
            .iter_mut()
            // only compute collisions with bullets when the player is alive
            .filter_map(|(_, p)| if p.alive { Some(p) } else { None })
        {
            for bullet in self.bullets.iter() {
                if circle_circle_collision(
                    &player.position,
                    PLAYER_RADIUS,
                    &bullet.position,
                    BULLET_RADIUS,
                )
                .is_ok()
                {
                    player.alive = false;
                    break; // skip to another player
                }
            }
        }

        // collisions of bullets with other bullets
        for freed_bullet_player_id in self
            .collisions_between_bullets()
            .into_iter()
            .rev()
            .map(|i| self.bullets.remove(i).player_id)
        {
            if let Some(player) = self.players.get_mut(&freed_bullet_player_id) {
                player.bullets_remaining += 1;
            }
        }

        // collisions of bullets with tiles
        for freed_bullet_player_id in self
            .bullet_collisions_with_tiles()
            .into_iter()
            .rev()
            .map(|i| self.bullets.remove(i).player_id)
        {
            if let Some(player) = self.players.get_mut(&freed_bullet_player_id) {
                player.bullets_remaining += 1;
            }
        }

        // collisions of players with tiles
        for player in self.players.iter_mut().map(|(_, p)| p) {
            for (loc, tile) in &self.environment.tiles {
                if let Tile::DesructableWall((0, _)) = tile {
                    if let Err(Vector2 { x, y }) = circle_rect_collision(
                        &player.position,
                        PLAYER_RADIUS,
                        &Vector2 {
                            x: loc.0 as _,
                            y: loc.1 as _,
                        },
                        1.0,
                        1.0,
                    ) {
                        // push the tanks out of the collision box
                        player.position = player.position.plus(&Vector2 {
                            x: PLAYER_RADIUS - x,
                            y: PLAYER_RADIUS - y,
                        })
                    }
                }
            }
        }

        // collisions of bullets with bounds
        for freed_bullet_player_id in self
            .bullet_collisions_with_bounds()
            .into_iter()
            .rev()
            .map(|i| self.bullets.remove(i).player_id)
        {
            if let Some(player) = self.players.get_mut(&freed_bullet_player_id) {
                player.bullets_remaining += 1;
            }
        }

        // collisions of players with bounds
        for player in self.players.iter_mut().map(|(_, p)| p) {
            if player.position.x + PLAYER_RADIUS >= MAP_BLOCK_WIDTH as _ {
                player.position.x = MAP_BLOCK_WIDTH as f64 - PLAYER_RADIUS;
            } else if player.position.x - PLAYER_RADIUS <= 0.0 {
                player.position.x = PLAYER_RADIUS;
            }

            if player.position.y + PLAYER_RADIUS >= MAP_BLOCK_HEIGHT as _ {
                player.position.y = MAP_BLOCK_HEIGHT as f64 - PLAYER_RADIUS;
            } else if player.position.y - PLAYER_RADIUS <= 0.0 {
                player.position.y = PLAYER_RADIUS;
            }
        }
    }

    /// Processes collisions between bullets and collects unqiue items
    /// Returns the indicies of bullets which should need to be removed
    fn collisions_between_bullets(&mut self) -> BTreeSet<usize> {
        let mut set = BTreeSet::new();

        // TODO: try to optimize this later from O(n^2)
        for i in 0..self.bullets.len() {
            for j in 0..self.bullets.len() {
                if i != j
                    && circle_circle_collision(
                        &self.bullets[i].position,
                        BULLET_RADIUS,
                        &self.bullets[j].position,
                        BULLET_RADIUS,
                    )
                    .is_err()
                {
                    set.insert(i);
                    set.insert(j);
                }
            }
        }

        set
    }

    fn bullet_collisions_with_tiles(&mut self) -> BTreeSet<usize> {
        let mut set = BTreeSet::new();

        for (i, bullet) in &mut self.bullets.iter_mut().enumerate() {
            for (loc, tile) in &self.environment.tiles {
                if let Tile::DesructableWall((0, _)) = tile {
                    if let Err(collision) = circle_rect_collision(
                        &bullet.position,
                        PLAYER_RADIUS,
                        &Vector2 {
                            x: loc.0 as _,
                            y: loc.1 as _,
                        },
                        1.0,
                        1.0,
                    ) {
                        if bullet.ricochets > 0 {
                            bullet.ricochets -= 1;
                            // this is faster due to the assumption that all walls are either horizontal or vertically aligned
                            bullet.velocity.x *= match collision.x > 0.0 {
                                true => collision.x.signum(),
                                false => 1.0,
                            };
                            bullet.velocity.y *= match collision.y > 0.0 {
                                true => collision.y.signum(),
                                false => 1.0,
                            }
                        } else {
                            set.insert(i);
                        }
                        // reverse the velocity of the bullet
                    }
                }
            }
        }

        set
    }

    fn bullet_collisions_with_bounds(&mut self) -> BTreeSet<usize> {
        let mut set = BTreeSet::new();

        for (i, bullet) in &mut self.bullets.iter_mut().enumerate() {
            let collides_x = bullet.position.x + BULLET_RADIUS > MAP_BLOCK_WIDTH as _
                || bullet.position.x - BULLET_RADIUS < 0.0;
            let collides_y = bullet.position.y + BULLET_RADIUS > MAP_BLOCK_HEIGHT as _
                || bullet.position.y - BULLET_RADIUS < 0.0;

            if collides_x || collides_y {
                if bullet.ricochets > 0 {
                    bullet.ricochets -= 1;
                    bullet.velocity.x *= -(collides_x as i8) as f64;
                    bullet.velocity.y *= -(collides_y as i8) as f64;
                } else {
                    set.insert(i);
                }
            }
        }

        set
    }
}
