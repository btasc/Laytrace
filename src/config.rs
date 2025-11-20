// Config that specifies all settings for running
// Has default implemented so you can just select a few things
#[derive(Debug)]
pub struct LatrConfig {
    pub fps_cap: u32,
    pub resolution: (u32, u32),
    pub num_rays: (u32, u32),
    pub run_mode: RunMode,
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
        }
    }
}


// This is an enum so more can be added later
#[derive(Debug, PartialEq, Default)]
pub enum RunMode {
    #[default] // Gui is now default
    // Gui is the normal run mode with full features and screen
    Gui,

    // Headless is a testing mode that removes the screen, allowing faster testing
    Headless,
}