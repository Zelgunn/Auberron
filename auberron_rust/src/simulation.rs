mod game_resource;
use game_resource::{GameResourceLedger, GameResources};

pub struct Simulation {
    total_time: f64,

    // Resources
    resources: GameResources,
    positive_ledger: GameResourceLedger,
    negative_ledger: GameResourceLedger,
}

impl Simulation {
    pub fn new() -> Self {
        return Self {
            total_time: 0.0,

            resources: GameResources::default(),
            positive_ledger: GameResourceLedger::default(),
            negative_ledger: GameResourceLedger::default(),
        };
    }

    pub fn advance(&mut self, delta: f64) {
        self.total_time += delta;

        let positive_rates = self.positive_ledger.compute_resource_rates();
        let negative_rates = self.negative_ledger.compute_resource_rates();
        self.resources
            .integrate(positive_rates, negative_rates, delta);
    }

    // region: Getters
    pub fn get_total_time(&self) -> f64 {
        return self.total_time;
    }

    // endregion
}
