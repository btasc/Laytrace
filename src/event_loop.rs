use crate::{
    error::{LatrError, WindowError},
    config::LatrConfig,
    engine::{
        engine_core::{ Engine },
    },
    gpu::gpu_core::GpuCore,
    PhysicsLoop
};

use winit::event_loop::EventLoopWindowTarget;

use std::{
    sync::{Arc, mpsc},
    thread,
};

pub fn run_event_loop<T: PhysicsLoop + 'static + std::marker::Send>(
    config: LatrConfig,
    engine_core: Engine,
    gpu_core: GpuCore,
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
    state: Option<T>,
    tps: Option<u32>,
) -> Result<(), LatrError> {
    let tick_rate = tps;
    let mut gpu_core = gpu_core;

    match state {
        Some(state) => {
            // Never returns unless there's an error
            // Uses mpsc to message an error if it happens
            thread::spawn(move || {
                let mut engine = engine_core;
                let state = state;
                let tick_rate = tick_rate.expect("Unreachable: Tick rate is undefined, yet state is. This should not be the case, as they are both passed into start as an option.");
            });
        },
        None => (),
    }

    let render_res = event_loop.run(move |event, elwt: &EventLoopWindowTarget<()>| {
        match event {
            winit::event::Event::WindowEvent { window_id, event }
            if window_id == window.id() => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        println!("Close button was pressed - Exiting.");
                        elwt.exit();
                    }

                    winit::event::WindowEvent::RedrawRequested => {

                        gpu_core.render();
                        window.request_redraw();
                    }

                    _ => ()
                }
            }
            _ => ()
        }
    });

    match render_res {
        Ok(()) => Ok(()),
        Err(e) => Err(LatrError::Window(WindowError::EventLoop(e))),
    }
}