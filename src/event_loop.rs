use crate::{
    error::{LatrError, WindowError},
    latr_core::LatrEngine,
    config::LatrConfig,
    engine::{
        engine_core::{ Engine },
        params::TriangleWorkOrder,
    },
    gpu_utils::gpu_core::GpuCore,
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
    
    let (order_sender, order_recv) = mpsc::channel::<Vec<TriangleWorkOrder>>();

    match state {
        Some(state) => {
            // Never returns unless there's an error
            // Uses mpsc to message an error if it happens
            thread::spawn(move || {
                let mut engine = engine_core;
                let state = state;
                let tick_rate = tick_rate.expect("Unreachable: Tick rate is undefined, yet state is. This should not be the case, as they are both passed into start as an option.");

                let engine_res = engine.start_physics_loop(state, tick_rate, order_sender);
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
                        // order_recv
                        
                        let mut orders: Vec<TriangleWorkOrder> = Vec::new();
                        
                        match order_recv.try_recv() {
                            Ok(data) => {
                                orders = data;
                            },
                            Err(mpsc::TryRecvError::Empty) => {
                                // If its empty, we dont care and just ignore
                                // We use the default Vec::new()
                            },
                            Err(mpsc::TryRecvError::Disconnected) => {
                                // Handle the engine thread disconnecting
                                // For now we do nothing, implement later
                                // todo!()
                            }
                        }
                        
                        gpu_core.render(orders);
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