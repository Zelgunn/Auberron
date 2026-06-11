use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::fmt;
use godot::prelude::*;


#[derive(GodotConvert, Var, Export, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[godot(via = i64)]
pub enum GameResourceType
{
    Solarite = 0
}

pub struct GameResource
{
    id: GameResourceType,
    amount: f64
}

impl GameResource 
{
    pub fn new(id: GameResourceType) -> Self
    {
        return Self{id: id, amount: 0.0};
    }
}

impl Add<f64> for GameResource
{
    type Output = Self;

    fn add(self, rhs: f64) -> Self
    {
        return Self
        {
            id: self.id,
            amount: self.amount + rhs
        };
    }
}

impl Sub<f64> for GameResource
{
    type Output = Self;

    fn sub(self, rhs: f64) -> Self
    {
        return GameResource::add(self, -rhs);
    }
}

impl AddAssign<f64> for GameResource
{
    fn add_assign(&mut self, rhs: f64)
    {
        self.amount += rhs;
    }
}

impl SubAssign<f64> for GameResource
{
    fn sub_assign(&mut self, rhs: f64)
    {
        *self += -rhs;
    }
}

impl fmt::Debug for GameResource
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        return f.debug_struct("Resource").
            field("ID", &self.id).
            field("amount", &self.amount).
            finish();
    }
}