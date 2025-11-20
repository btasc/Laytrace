use crate::error::EngineError;
use crate::config::LatrConfig;

use std::sync::Arc;

use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub struct Engine {

}

impl Engine {
    pub fn new(config: &LatrConfig) -> Result<Self, EngineError> {

        Ok(Self {

        })
    }
}