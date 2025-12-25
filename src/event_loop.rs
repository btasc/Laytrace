use crate::{
    error::{LatrError, WindowError, EngineError},
    config::LatrConfig,
    engine::{
        engine_core::{ Engine },
        bvh::build_write_bvh,
    },
    gpu::gpu_core::GpuCore,
    PhysicsLoop
};

use winit::event_loop::EventLoopWindowTarget;

use std::{
    sync::{Arc, mpsc},
    cell::RefCell,
    rc::Rc,
    thread,
};

pub fn run_event_loop<T: PhysicsLoop + 'static + std::marker::Send>(
    config: LatrConfig,
    engine_core: Engine,
    gpu_core: GpuCore,
    window: Arc<winit::window::Window>,
    event_loop: winit::event_loop::EventLoop<()>,
    state_tps_op: Option<(T, u32)>,
) -> Result<(), LatrError> {
    let mut gpu_core = gpu_core;

    // Since event_loop.run returns an event loop err, to get a LatrErr, we need to store it somewhere
    // We use an Rc RefCell to be able to update the error if it occurs
    let gpu_err: Rc<RefCell<Option<LatrError>>> = Rc::new(RefCell::new(None));
    let gpu_err_clone = gpu_err.clone();

    // Now we spawn the secondary thread
    // This thread will first build the bvh, then run the engine
    // We also clone our buffers as to let both threads update them
    // Wgpu handles this very helpfully and treats a wgpu::Buffer as atomically ref counted (At least I believe so)
    let secondary_buffers = gpu_core.buffers.clone();
    let secondary_queue = gpu_core.queue.clone();

    let secondary_config = config.clone();

    let secondary_thread = thread::spawn(move || {

        // Move these explicitly
        let mut secondary_buffers = secondary_buffers;
        let mut secondary_queue = secondary_queue;

        let secondary_config = secondary_config;
        let mut engine = engine_core;
        
        let state_tps_op = state_tps_op;

        // We store a res to return if any programs run into an error
        let mut bvh_res: Result<(), EngineError> = Ok(());
        let mut engine_res: Result<(), LatrError> = Ok(());

        // Run our first bvh task
        match secondary_config.model_file {
            Some(model_file) => {
                bvh_res = build_write_bvh(model_file, &mut secondary_buffers, &mut secondary_queue);
            },
            None => (),
        }

        match bvh_res {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                return ();
            },
        }

        // After that is done, we run our engine

        match state_tps_op {
            Some((state, tps)) => {
                engine_res = engine.start_physics_loop(state, tps, &mut secondary_buffers, &mut secondary_queue);
            },
            None => (),
        }

        match engine_res {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                return ();
            },
        }

    });

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

                        match gpu_core.render() {
                            Ok(_) => {},
                            Err(e) => {
                                *gpu_err_clone.borrow_mut() = Some(LatrError::Gpu(e));
                                elwt.exit();
                            }
                        };

                        window.request_redraw();
                    }

                    _ => ()
                }
            }
            _ => ()
        }
    });

    if let Ok(err_cell) = Rc::try_unwrap(gpu_err) {
        if let Some(gpu_err) = err_cell.into_inner() {
            return Err(gpu_err);
        }
    }

    match render_res {
        Ok(()) => Ok(()),
        Err(e) => Err(LatrError::Window(WindowError::EventLoop(e))),
    }
}