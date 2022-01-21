struct PlayerData {}
struct ProjectileData {}

pub struct TanksState {
    pub pos: (f64, f64),
    player_data: Vec<PlayerData>,
    projectile_data: Vec<ProjectileData>,
}

impl TanksState {
    pub fn new() -> Self {
        Self {
            pos: (0.0, 0.0),
            player_data: Vec::new(),
            projectile_data: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let delta = self.delta();
        self.pos.0 += delta;
    }

    fn delta(&mut self) -> f64 {
        10.0
    }
}
