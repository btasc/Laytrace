use crate::{
    error::{EngineError, LatrError},
    config::LatrConfig,
};

use super::{
    params::{ GpuUniformParams, EngineParams, EngineCamera },
};

use std::{
    time::{Instant, Duration},
    thread,
    sync::{ Arc, RwLock, mpsc },
};
use crate::engine::params::TriangleBuffer;

pub trait PhysicsLoop {
    fn init(&mut self, en: &mut Engine) -> Result<(), LatrError>;
    fn update(&mut self, en: &mut Engine) -> Result<(), LatrError>;
}

// These return LatrErrors as the user will run most physics operations through the LatrEngine api
// This way the user doesn't have to be like "engine.engine.run_op", and doesn't have to deal with EngineError vs LatrError
pub struct Engine {
    pub engine_params: EngineParams,
    pub triangle_buffer: TriangleBuffer,

    pub double_buffer: Arc<[RwLock<TriangleBuffer>; 2]>,
}

#[derive(PartialEq)]
pub enum DoubleBuffer {
    Buf1,
    Buf2,
}

impl DoubleBuffer {
    fn flip(&mut self) {
        *self = self.get_flip();
    }

    fn get_flip(&self) -> Self {
        match self {
            DoubleBuffer::Buf1 => DoubleBuffer::Buf2,
            DoubleBuffer::Buf2 => DoubleBuffer::Buf1,
        }
    }

    fn to_index(&self) -> usize {
        match self {
            DoubleBuffer::Buf1 => 0,
            DoubleBuffer::Buf2 => 1,
        }
    }
}

impl Engine {
    pub fn new(config: &LatrConfig) -> Result<Self, EngineError> {
        let engine_params = EngineParams {
            camera: EngineCamera::default(),
            screen_dimensions: config.resolution,
        };

        let vertices = vec!([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        let triangles = vec!([0, 1, 2]);

        let triangle_buffer = TriangleBuffer {
            vertices,
            triangles,
        };

        let buf1 = RwLock::new(triangle_buffer.clone());
        let buf2 = RwLock::new(triangle_buffer.clone());

        let double_buffer = Arc::new([buf1, buf2]);

        Ok(Self {
            engine_params,
            triangle_buffer,
            double_buffer,
        })
    }

    pub fn move_camera(&mut self, dx: f32, dy: f32, dz: f32) {
        let pos = &mut self.engine_params.camera.pos;

        pos[0] += dx;
        pos[1] += dy;
        pos[2] += dz;
    }

    pub fn start_physics_loop<T: PhysicsLoop + 'static>(

        &mut self,
        state: T, tps /* Tick rate per second of loop */: u32,
        engine_params_sender: mpsc::Sender<EngineParams>,
        double_buf_sender: mpsc::Sender<DoubleBuffer>,

    ) -> Result<(), LatrError> {
        let mut state = state;
        let mut write_ready_double_buf = DoubleBuffer::Buf1;

        let tick_duration = Duration::from_secs_f64(1f64 / tps as f64);

        {
            state.init(self)?;
            self.send_frame(&mut write_ready_double_buf, &double_buf_sender, &engine_params_sender);
        }

        loop {
            // Gets the start of the loop
            let loop_start = Instant::now();

            // Run the actual physics of the loop
            // This needs to be nested because borrowing rules and such (it's a mutable reference to self)
            {
                state.update(self)?;
                self.send_frame(&mut write_ready_double_buf, &double_buf_sender, &engine_params_sender);
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
    
    // Helper function for physics loop
    // Sends params and buf index to visual thread
    fn send_frame(
        &self,
        write_tracker: &mut DoubleBuffer,
        buf_sender: &mpsc::Sender<DoubleBuffer>,
        params_sender: &mpsc::Sender<EngineParams>,
    ) {
        let mut writable_buf = self.double_buffer[write_tracker.to_index()].write()
            .expect(EngineError::POISON_ERR);

        writable_buf.clone_from(&self.triangle_buffer);

        // We flip it so that it's the opposite of the write buffer, which is the one we want to read from
        buf_sender.send(write_tracker.get_flip())
            .expect(EngineError::POISON_ERR);

        // We then actually flip it mutably for the next write
        write_tracker.flip();

        params_sender.send(self.engine_params)
            .expect(EngineError::POISON_ERR);
    }
}