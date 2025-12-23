use crate::{
    error::{EngineError, LatrError},
    config::LatrConfig,
    gpu::buffers::GpuUniformCamera,
};

use std::{
    time::{Instant, Duration},
    thread,
};

use std::sync::mpsc;

pub trait PhysicsLoop {
    fn init(&mut self, en: &mut Engine) -> Result<(), LatrError>;
    fn update(&mut self, en: &mut Engine) -> Result<(), LatrError>;
}

// Methods open to the user return LatrErrors, methods only exposed to the engine return engine errors
pub struct Engine {
    pub gpu_cam: GpuUniformCamera,
}

impl Engine {
    pub fn new(config: &LatrConfig) -> Result<Self, EngineError> {
        let gpu_cam: GpuUniformCamera = GpuUniformCamera::default();

        Ok(Self {
            gpu_cam,
        })
    }

    pub fn move_camera(&mut self, dx: f32, dy: f32, dz: f32) {
        let pos = &mut self.gpu_cam.pos;

        pos[0] += dx;
        pos[1] += dy;
        pos[2] += dz;
    }

    pub fn start_physics_loop<T: PhysicsLoop + 'static>(
        &mut self,
        state: T, tps /* Tick rate per second of loop */: u32,
        order_sender: mpsc::Sender<Vec<TriangleWorkOrder>>,
    ) -> Result<(), LatrError> {
        let mut state = state;

        let tick_duration = Duration::from_secs_f64(1f64 / tps as f64);

        {
            state.init(self)?;

            order_sender.send(self.flush_ret_orders())
                .expect(EngineError::POISON_ERR);
        }

        loop {
            // Gets the start of the loop
            let loop_start = Instant::now();

            // Run the actual physics of the loop
            // This needs to be nested because borrowing rules and such (it's a mutable reference to self)
            {
                state.update(self)?;

                order_sender.send(self.flush_ret_orders())
                    .expect(EngineError::POISON_ERR);
            };

            //println!("Engine loop time: {:?}", loop_start.elapsed());
            //println!("{}", self.engine_params.camera.pos[0]);

            // Gets the time elapsed while running physics
            let loop_duration = loop_start.elapsed();

            // Subtracts the tick time from the time spent
            // If the time spent is greater, then it returns none, and we don't pause
            // If the time spent is less, we only pause for the extra bit as to not waste time
            match tick_duration.checked_sub(loop_duration) {
                Some(time) => thread::sleep(time),
                None => println!("Lag"),
            }
        }
    }

    fn flush_ret_orders(&mut self) -> Vec<TriangleWorkOrder> {
        std::mem::take(&mut self.orders)
    }
}