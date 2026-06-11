mod game_resource;
use game_resource::GameResource;

pub struct Simulation
{
    total_time: f64,
    resources: Vec<GameResource>
}

impl Simulation
{
    pub fn new() -> Self
    {
        Self 
        {
            total_time: 0.0,
            resources: Vec::with_capacity(16)
        }
    }

    pub fn advance(&mut self, delta: f64)
    {
        self.total_time += delta;
    }

    fn compute_income(&self) -> Vec<GameResource>
    {
        return Vec::new();
    }

    // region: Getters
    pub fn get_total_time(&self) -> f64
    {
        self.total_time
    }

    // endregion
}