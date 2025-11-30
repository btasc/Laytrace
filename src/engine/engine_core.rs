use crate::{
    error::{ EngineError, LatrError },
    config::LatrConfig,
};

use super::{
    params::{ GpuUniformParams, EngineParams },
};

// These return LatrErrors as the user will run most physics operations through the LatrEngine api
// This way the user doesn't have to be like "engine.engine.run_op", and doesn't have to deal with EngineError vs LatrError
pub struct Engine {
    gpu_params: GpuUniformParams,
    engine_params: EngineParams,
}

impl Engine {
    pub fn new(config: &LatrConfig) -> Result<Self, EngineError> {

        Ok(Self {
            gpu_params: GpuUniformParams::default(),
            engine_params: EngineParams::default(),
        })
    }
    
    pub fn get_gpu_uniform_params(&self) -> &GpuUniformParams { &self.gpu_params }
}