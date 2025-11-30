use latr::{ LatrEngine, LatrConfig, PhysicsLoop, Physics };

fn main() -> Result<(), latr::LatrError> {
    let config = LatrConfig {
        ..Default::default()
    };

    let state = SimState {
        x: "hello!"
    };

    let physics = Physics::new(state);

    let mut engine = LatrEngine::new(config)?
        .start(physics)?;

    Ok(())
}

struct SimState {
    x: &'static str,
}

impl PhysicsLoop for SimState {
    fn init(&mut self, physics: Physics) -> Result<(), latr::LatrError> {

        Ok(())
    }

    fn update(&mut self, physics: Physics) -> Result<(), latr::LatrError> {


        Ok(())
    }
}