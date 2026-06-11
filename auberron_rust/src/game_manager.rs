use godot::prelude::*;
use crate::simulation::Simulation;

#[derive(GodotClass)]
#[class(base=Node)]
struct GameManager
{
    simulation: Simulation,

    base: Base<Node>
}

#[godot_api]
impl INode for GameManager
{
    fn init(base: Base<Node>) -> Self
    {
        Self {
            simulation: Simulation::new(),
            base
        }
    }

    fn process(&mut self, delta: f64)
    {
        self.simulation.advance(delta);

        godot_print!("{}", self.simulation.get_total_time());
    }
}
