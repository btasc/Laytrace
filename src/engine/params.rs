use super::engine_core::Engine;


// ! -- Params that are just for the engine, not the gpu -- !

// This will store all of our data for the engine
// These don't go to the gpu, but you can create the gpu params from these
#[derive(Clone, Copy)]
pub struct EngineCamera {
    pub pos: [f32; 3],
    pub pitch: f32,
    pub yaw: f32,
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

#[derive(Clone, Copy)]
pub struct EngineParams {
    pub camera: EngineCamera,
    pub screen_dimensions: (u32, u32),
}

// Even though we directly upload vertices to the gpu, we dont need to pad it
// This is because we upload it as a Vec<f32>, then stitch it on the gpu
// Also, a Vec<[f32; 3]> is the same as a Vec<f32> because "zero-cost zero-copy cast" apparently
// That means that on the gpu, we need to stitch it by making a helper get_vertex function
#[derive(Clone)]
pub struct TriangleBuffer {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<TriangleData>,
}

impl TriangleBuffer {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.triangles.clear();
    }
}

// This will go directly to the gpu, so we bytemuck it
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleData {
    pub vertices: [u32; 3], // Index to the vertex
    pub _pad: u32,
    pub color: [f32; 4], // RGBA
}

impl Default for TriangleData {
    fn default() -> Self {
        Self {
            vertices: [0, 0, 0],
            _pad: 0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

// This is a uniform buffer, so it's all the small stuff
// We'll have a storage buffer for the vertices later
// This is what is actually passed into the compute shader
// This is in engine and not in gpu to make it easier to change something
// For this to work as a uniform buffer, all the like memory addresses have to be in sets of like 16 bytes
// An f32 is 32 bits, or 4 bytes, so we need everything to be 4 f32's so that they follow this rule
// We can add _padding to get screen_dims to follow this, but everything else just has a trailing f32 that does nothing
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuUniformParams {
    pub camera_pos: [f32; 4],
    pub camera_forward: [f32; 4],
    pub camera_up: [f32; 4],
    pub camera_right: [f32; 4],
    pub screen_dims: [u32; 2], // Two f32's need padding
    pub _padding: [f32; 2], // Makes up for screen_dims
}

impl GpuUniformParams {
    pub fn from_engine_params(engine_params: &EngineParams) -> Self {
        // Init the camera
        let engine_camera = &engine_params.camera;
        let [x, y, z] = engine_camera.pos;

        // Again the 0 is just padding
        let camera_pos = [x, y, z, 0.0];

        let (res_x, res_y) = engine_params.screen_dimensions;
        let screen_dims = [res_x, res_y];

        // Get the basis vectors for the camera
        // We assume that Y is up, X is right, and Z is towards the user
        // By towards the user, imagine you are looking at a 2d graph
        // A point coming out of the screen towards you is positive Z, and a point going through your monitor away is negative Z
        // Also a lot of this math code is copy and pasted, so there is a high chance of glitches

        // Precalculate sin and cos values since we need them a lot
        let (p, y) = (engine_camera.pitch, engine_camera.yaw);

        let cos_p = p.cos();
        let sin_p = p.sin();
        let cos_y = y.cos();
        let sin_y = y.sin();

        // Calculate forward vector
        // 0 padded at the end, other vectors will follow this convention for the 16 byte rule
        let forward = [
            cos_y * cos_p,  // X
            sin_p,          // Y
            -sin_y * cos_p, // Z
            0.0,            // Padding
        ];

        // Calculate right vector
        let right = [
            -sin_y, // X
            0.0,    // Y
            -cos_y, // Z
            0.0,    // Pading
        ];

        // Calculate up vector
        let up = [
            -cos_y * sin_p, // X
            cos_p,          // Y
            sin_y * sin_p,  // Z
            0.0,            // Padding
        ];

        Self {
            camera_pos,
            camera_forward: forward,
            camera_right: right,
            camera_up: up,
            screen_dims,
            _padding: [0.0, 0.0],
        }
    }
}

impl Default for GpuUniformParams {
    fn default() -> Self {
        let arr4 = [0f32, 0f32, 0f32, 0f32];

        Self {
            camera_pos: arr4,
            camera_forward: arr4,
            camera_up: arr4,
            camera_right: arr4,
            screen_dims: [0, 0],
            _padding: [0.0, 0.0],
        }
    }
}