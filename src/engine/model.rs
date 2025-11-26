use tobj;

pub struct Model {
    name: Option<String>,
    verticies: Vec<f32>,
    triangles: Vec<[u32, 3]>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            name: None,
            verticies: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn from_obj(path: std::path::PathBuf) -> Self {
        // https://docs.rs/tobj/latest/tobj/
        let tobj_model = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);
        assert!(tobj_model.is_ok()); // Make this an error result

        let (models, materials) = tobj_model;
    }
}