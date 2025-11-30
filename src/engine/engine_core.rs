use crate::{
    error::{EngineError, LatrError},
    config::LatrConfig,
};

use super::{
    params::{ GpuUniformParams, EngineParams, EngineCamera },
    physics::{ PhysicsLoop, Physics },
};

use std::{
    time::Instant,
    thread,
    sync::{ Arc, Mutex },
};
use std::time::Duration;

// These return LatrErrors as the user will run most physics operations through the LatrEngine api
// This way the user doesn't have to be like "engine.engine.run_op", and doesn't have to deal with EngineError vs LatrError
pub struct Engine {
    gpu_params: GpuUniformParams,
    engine_params: Arc<Mutex<EngineParams>>,
}

impl Engine {
    pub fn new(config: &LatrConfig) -> Result<Self, EngineError> {
        let engine_params = EngineParams {
            camera: EngineCamera::default(),
            screen_dimensions: config.resolution,
        };

        Ok(Self {
            gpu_params: GpuUniformParams::default(),
            engine_params: Arc::new(Mutex::new(engine_params)),
        })
    }

    pub fn get_params_arc(&self) -> Arc<Mutex<EngineParams>> {
        Arc::clone(&self.engine_params)
    }
    
    pub fn get_gpu_uniform_params(&self) -> &GpuUniformParams { &self.gpu_params }

    pub fn start_physics_loop<T: PhysicsLoop + 'static>(&mut self, state: T, tps /* Tick rate per second of loop */: u32) -> Result<(), LatrError> {
        let mut physics = Physics::new(self);
        let mut state = state;

        let tick_duration = Duration::from_secs_f64(1f64 / tps as f64);

        { state.init(&mut physics)?; }

        loop {
            // Gets the start of the loop
            let loop_start = Instant::now();

            // Run the actual physics of the loop
            { state.update(&mut physics)? };

            // Gets the time elapsed while running physics
            let loop_duration = loop_start.elapsed();

            // Subtracts the tick time from the time spent
            // If the time spent is greater, then it returns none and we dont pause
            // If the time spent is less, we only pause for the extra bit as to not waste time
            match tick_duration.checked_sub(loop_duration) {
                Some(time) => thread::sleep(time),
                None => println!("Lag"),
            }
        }
    }
}