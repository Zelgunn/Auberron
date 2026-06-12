mod game_resource;
use game_resource::GameResources;

pub struct Simulation {
    total_time: f64,

    // Resources
    resources: GameResources,
    base_income_buffer: GameResources,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            total_time: 0.0,
            resources: GameResources::new(),
            base_income_buffer: GameResources::new(),
        }
    }

    pub fn advance(&mut self, delta: f64) {
        self.total_time += delta;
    }

    // region: Getters
    pub fn get_total_time(&self) -> f64 {
        self.total_time
    }

    // endregion
}
