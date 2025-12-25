use serde::Deserialize;

use crate::{
    error::LatrError,
    latr_core::LatrEngine,
    engine::{ 
        engine_core::Engine,
    },
};

// Config that specifies all settings for running
// Has default implemented so you can just select a few things
// Also contains all the blueprints for different models that we will use

// We use P so that the user can pass in almost anything that resembles a file path and it will still function
// We also set the default to PathBuf just because None is an Option<PathBuf> and PathBuf implements AsRef<Path>
// Normally I would use unit type for that sort of thing, but since () doesn't have AsRef, we can't
#[derive(Clone)]
pub struct LatrConfig {
    pub fps_cap: u32,
    pub model_file: Option<std::path::PathBuf>,
    pub resolution: (u32, u32),
    pub num_rays: (u32, u32),
    pub run_mode: RunMode,
}

impl LatrConfig {
    pub fn attach_models<P: AsRef<std::path::Path>>(&mut self, file_path: P) {
        let path_buf_file = file_path.as_ref().to_path_buf();
        self.model_file = Some(path_buf_file);
    }
}

impl Default for LatrConfig {
    fn default() -> Self {
        let resolution: (u32, u32) = (640, 360);
        let num_rays = resolution;

        Self {
            fps_cap: 60,
            resolution,
            num_rays,
            run_mode: RunMode::default(),
            model_file: None,
        }
    }
}


// This is an enum so more can be added later
#[derive(Debug, PartialEq, Default, Clone)]
pub enum RunMode {
    #[default] // Gui is now default
    // Gui is the normal run mode with full features and screen
    Gui,

    // NoWinit is a testing mode that removes the screen, allowing faster testing
    // Does still have the gpu and everything that comes with it, just no winit
    // Gpu writes to nothing
    NoWinit,
}

// All this code is here as it technically relates to a config, even though its used on engine
// It's a bit iffy, but I want to pad the size of this file a bit, 40 lines is too small
#[derive(Deserialize, Debug)]
struct ExplicitModelConfig {
    name: String,
    path: String,
}

#[derive(Deserialize, Debug)]
struct DirectoriesConfig {
    model_folders: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ModelConfig {
    directories: DirectoriesConfig,
    models: Vec<ExplicitModelConfig>,
}