
use latr::{ LatrEngine, LatrConfig, Engine, PhysicsLoop };

fn main() -> Result<(), latr::LatrError> {
    let mut config = LatrConfig {
        resolution: (1920u32, 1080u32),
        ..Default::default()
    };
    
    config.attach_models("./ModelConfig.toml");

    let state = SimState {
        x: "hello!",
        t: false,
    };

    let mut engine = LatrEngine::new(config)?
        .start(Some((state, 1)))?;

    Ok(())
}

struct SimState {
    x: &'static str,
    t: bool,
}

impl PhysicsLoop for SimState {
    fn init(&mut self, engine: &mut Engine) -> Result<(), latr::LatrError> {
        engine.move_camera(200.0, 0.0, 0.0);

        Ok(())
    }

    fn update(&mut self, engine: &mut Engine) -> Result<(), latr::LatrError> {
        if(self.t) {
            engine.move_camera(20.0, 0.0, 0.0);
            self.t = false;
        } else {
            engine.move_camera(-20.0, 0.0, 0.0);
            self.t = true;
        }

        Ok(())
    }
}