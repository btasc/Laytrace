use crate::{
    error::{ LatrError, WindowError },
    latr_core::LatrEngine,
    config::LatrConfig,
    engine::{
        engine_core::Engine,
        physics::Physics,
    },
    gpu_utils::gpu_core::GpuCore,
};

use winit::event_loop::EventLoopWindowTarget;

use std::sync::Arc;

pub fn run_event_loop(
    config: LatrConfig,
    engine_core: Engine,
    gpu_core: GpuCore,
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
    physics: Physics,
) -> Result<(), LatrError> {

    let res = event_loop.run(move |event, elwt: &EventLoopWindowTarget<()>| {
        match event {

            winit::event::Event::WindowEvent { window_id, event }
            if window_id == window.id() =>
                {
                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            println!("Close button was pressed - Exiting.");
                            elwt.exit();
                        }

                        winit::event::WindowEvent::RedrawRequested => {

                        }

                        _ => ()
                    }
                }
            

            _ => ()
        }
    });

    match res {
        Ok(()) => Ok(()),
        Err(e) => Err(LatrError::Window(WindowError::EventLoop(e))),
    }
}