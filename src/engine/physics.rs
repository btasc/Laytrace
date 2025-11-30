use super::engine_core::Engine;
use crate::error::LatrError;

pub trait PhysicsLoop {
    fn init(&mut self, physics: Physics) -> Result<(), LatrError>;
    fn update(&mut self, physics: Physics) -> Result<(), LatrError>;
}

// Physics is made to give a nice user handle to the Engine
// This way it's a bit easier for the user, and they have less things to import
pub struct Physics {
    physics_loop: Box<dyn PhysicsLoop>,
}

impl Physics {
    pub fn new<T: PhysicsLoop + 'static>(state: T) -> Self {
        Physics { physics_loop: Box::new(state), }
    }
}