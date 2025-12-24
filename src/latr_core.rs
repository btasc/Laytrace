use crate::{
    error::{LatrError, WindowError, }, 
    config::LatrConfig, 
    engine::{
        engine_core::{ Engine, PhysicsLoop },
    }, 
    gpu::gpu_core::GpuCore, 
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
    pub fn start<T: PhysicsLoop + 'static + std::marker::Send>(self, state_tps_op: Option<(T, u32)>) -> Result<(), LatrError> {
        let LatrEngine {
            config,
            engine_core,
            gpu_core,
            window,
            event_loop,
        } = self;

        // tps = ticks per second
        let (mut state, mut tps) = (None, None);

        match state_tps_op {
            Some(state_tps) => {
                state = Some(state_tps.0);
                tps = Some(state_tps.1);
            },
            None => (),
        }

        run_event_loop::<T>(
           config,
           engine_core,
           gpu_core,
           window,
           event_loop,
           state,
           tps,
        )?;

        Ok(())
    }

    pub fn new(latr_config: LatrConfig) -> Result<Self, LatrError> {
        let (window, event_loop) = Self::make_window_event_loop(latr_config.resolution)?;

        let engine_core = Engine::new(&latr_config)?;
        
        let gpu_core = GpuCore::new(
            window.clone(),
        )?;

        let config = latr_config;

        Ok(Self {
            config,
            gpu_core, engine_core,
            window, event_loop,
        })
    }

    fn make_window_event_loop(resolution: (u32, u32)) -> Result<(Arc<winit::window::Window>, winit::event_loop::EventLoop<()>), WindowError> {
        let event_loop = winit::event_loop::EventLoop::new()?;

        let window_arc = Arc::new(winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(resolution.0, resolution.1))
            .build(&event_loop)?);

        Ok((window_arc, event_loop))
    }
}