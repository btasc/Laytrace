use crate::{
    error::{
        LatrError,
        WindowError,
    },
    engine::engine_core::Engine,
    gpu_utils::gpu_core::GpuCore,
};

use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::{ResizeDirection, Window, WindowBuilder};
use crate::config::LatrConfig;

pub struct LatrEngine {
    engine_core: Engine,
    gpu_core: GpuCore,
    
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl LatrEngine {
    pub fn new(latr_config: &LatrConfig) -> Result<Self, LatrError> {
        let (window, event_loop) = Self::make_window_event_loop()?;
        
        let gpu_core = GpuCore::new(&latr_config, window.clone())?;
        let engine_core = Engine::new(&latr_config)?;

        Ok(Self {
            gpu_core, engine_core,
            window, event_loop,
        })
    }

    fn make_window_event_loop() -> Result<(Arc<winit::window::Window>, winit::event_loop::EventLoop<()>), WindowError> {
        let event_loop = EventLoop::new()?;

        let window_arc = Arc::new(WindowBuilder::new()
            .build(&event_loop)?);

        Ok((window_arc, event_loop))
    }
}