#![windows_subsystem = "windows"]

use latr::{ LatrEngine, LatrConfig, PhysicsLoop, Physics };

fn main() -> Result<(), latr::LatrError> {
    let config = LatrConfig {
        resolution: (1920u32, 1080u32),
        ..Default::default()
    };

    let state = SimState {
        x: "hello!"
    };

    let mut engine = LatrEngine::new(config)?
        .start(Some((state, 1)))?;

    Ok(())
}

struct SimState {
    x: &'static str,
}

impl PhysicsLoop for SimState {
    fn init(&mut self, physics: &mut Physics) -> Result<(), latr::LatrError> {

        println!("Init");
        Ok(())
    }

    fn update(&mut self, physics: &mut Physics) -> Result<(), latr::LatrError> {
        println!("Update");
        Ok(())
    }
}