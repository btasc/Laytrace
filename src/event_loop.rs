use crate::{
    error::{LatrError, WindowError},
    latr_core::LatrEngine,
    config::LatrConfig,
    engine::{
        engine_core::{ Engine, DoubleBuffer },
    },
    gpu_utils::gpu_core::GpuCore,
    PhysicsLoop
};

use winit::event_loop::EventLoopWindowTarget;

use std::{
    sync::{Arc, mpsc},
    thread,
};

use crate::engine::params::{EngineParams, GpuUniformParams};

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

    // Send is moved to engine thread
    let (engine_send, engine_rec) = mpsc::channel::<EngineParams>();
    let (double_buf_index_send, double_buf_index_rec) = mpsc::channel::<DoubleBuffer>();

    let mut gpu_core = gpu_core;

    match state {
        Some(state) => {
            // Never returns unless there's an error
            // Uses mpsc to message an error if it happens
            thread::spawn(move || {
                let mut engine = engine_core;
                let state = state;
                let tick_rate = tick_rate.expect("Unreachable: Tick rate is undefined, yet state is. This should not be the case, as they are both passed into start as an option.");

                let engine_res = engine.start_physics_loop(state, tick_rate, engine_send, double_buf_index_send);
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
                            let mut latest_params: Option<EngineParams> = None;
                            let mut latest_double_buffer: Option<DoubleBuffer> = None;

                            //println!("REDRAW REQUESTED");

                            while let Ok(data) = engine_rec.try_recv() {
                                latest_params = Some(data);
                            }

                            while let Ok(data) = double_buf_index_rec.try_recv() {
                                latest_double_buffer = Some(data);
                            }

                            if let Some(data) = latest_params {
                                let gpu_uniform_params = GpuUniformParams::from_engine_params(&data);
                                let gpu_triangle_params = todo!();

                                //println!("{}", data.camera.pos[0]);

                                gpu_core.render(&gpu_uniform_params);
                            } else {
                                // Render with old data todo!()

                            }

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