use latr::{ LatrEngine, LatrConfig, PhysicsLoop, Physics };

fn main() -> Result<(), latr::LatrError> {
    let config = LatrConfig {
        ..Default::default()
    };

    let mut state = SimState {
        x: "hello!"
    };

    let mut engine = LatrEngine::new(config)?
        .start(Some((state, 10)))?;

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