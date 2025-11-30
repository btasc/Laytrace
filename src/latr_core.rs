use crate::{
    error::{LatrError, WindowError, }, 
    config::LatrConfig, 
    engine::{
        engine_core::Engine,
        physics::Physics,
    }, 
    gpu_utils::gpu_core::GpuCore, 
    event_loop::run_event_loop,
};

use std::sync::Arc;

pub struct LatrEngine {
    config: LatrConfig,

    engine_core: Engine,
    gpu_core: GpuCore,
    
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl LatrEngine {
    pub fn start(self, physics: Physics) -> Result<(), LatrError> {
        let LatrEngine {
            config,
            engine_core,
            gpu_core,
            window,
            event_loop,
        } = self;

        run_event_loop(
           config,
           engine_core,
           gpu_core,
           window,
           event_loop,
        )?;

        Ok(())
    }

    pub fn new(latr_config: LatrConfig) -> Result<Self, LatrError> {
        let (window, event_loop) = Self::make_window_event_loop()?;
        
        let gpu_core = GpuCore::new(&latr_config, window.clone())?;
        let engine_core = Engine::new(&latr_config)?;

        let config = latr_config;

        Ok(Self {
            config,
            gpu_core, engine_core,
            window, event_loop,
        })
    }

    fn make_window_event_loop() -> Result<(Arc<winit::window::Window>, winit::event_loop::EventLoop<()>), WindowError> {
        let event_loop = winit::event_loop::EventLoop::new()?;

        let window_arc = Arc::new(winit::window::WindowBuilder::new()
            .build(&event_loop)?);

        Ok((window_arc, event_loop))
    }
}