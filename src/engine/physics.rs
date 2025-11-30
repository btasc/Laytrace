use super::engine_core::Engine;
use crate::error::LatrError;

pub trait PhysicsLoop {
    fn init(&mut self, physics: &mut Physics) -> Result<(), LatrError>;
    fn update(&mut self, physics: &mut Physics) -> Result<(), LatrError>;
}

// Physics is made to give a nice user handle to the Engine
// This way it's a bit easier for the user, and they have less things to import
pub struct Physics<'a> {
    engine: &'a Engine,
}

impl<'a> Physics<'a> {
    pub fn new(engine: &'a mut Engine) -> Self {
        Physics { engine, }
    }
}