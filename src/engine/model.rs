use tobj;

// This file is unfinished
// Ill get back to it when I start on gpu collision shaders

pub struct Model {
    name: Option<String>,
    vertices: Vec<f32>,
    triangles: Vec<[u32; 3]>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            name: None,
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn from_raw(vertices: Vec<f32>, triangles: Vec<[u32; 3]>) -> Self {
        let mut model = Self::new();
        model.vertices = vertices;
        model.triangles = triangles;

        model
    }

    pub fn from_obj(path: std::path::PathBuf) -> Self {
        // https://docs.rs/tobj/latest/tobj/
        let tobj_model = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);
        assert!(tobj_model.is_ok()); // Make this an error result
        todo!()
    }
}