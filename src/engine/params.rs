use super::engine_core::Engine;


// ! -- Params that are just for the engine, not the gpu -- !

// This will store all of our data for the engine
// These don't go to the gpu, but you can create the gpu params from these
pub struct EngineCamera {
    pub pos: [f32; 3],
    pub pitch: f32,
    pub yaw: f32,
}

impl EngineCamera {
    // Returns the up, forward, and right vectors respectively
    pub fn get_unit_vectors() -> ([f32; 3], [f32; 3], [f32; 3]) {
        todo!()
    }
}

impl Default for EngineCamera {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

pub struct EngineParams {
    pub camera: EngineCamera,
}

impl Default for EngineParams {
    fn default() -> Self {
        let camera = EngineCamera::default();

        Self {
            camera,
        }
    }
}



// ! -- Params to be passed directly to the gpu -- !

// This is a uniform buffer, so it's all the small stuff
// We'll have a storage buffer for the vertices later
// This is what is actually passed into the compute shader
// This is in engine and not in gpu to make it easier to change something
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// For this to work as a uniform buffer, all the like memory addresses have to be in sets of like 16 bytes
// An f32 is 32 bits, or 4 bytes, so we need everything to be 4 f32's so that they follow this rule
// We can add _padding to get screen_dims to follow this, but everything else just has a trailing f32 that does nothing
pub struct GpuUniformParams {
    pub camera_pos: [f32; 4],
    pub camera_forward: [f32; 4],
    pub camera_up: [f32; 4],
    pub camera_right: [f32; 4],
    pub screen_dims: [f32; 2], // Two f32's need padding
    pub _padding: [f32; 2], // Makes up for screen_dims
}

impl Default for GpuUniformParams {
    fn default() -> Self {
        let arr4 = [0f32, 0f32, 0f32, 0f32];
        let arr2 = [0f32, 0f32];

        Self {
            camera_pos: arr4,
            camera_forward: arr4,
            camera_up: arr4,
            camera_right: arr4,
            screen_dims: arr2,
            _padding: arr2,
        }
    }
}