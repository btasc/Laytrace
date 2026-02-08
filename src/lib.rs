mod engine;
mod gpu;
mod core;

// Mark this as test so the module only compiles if we're on testing mode
#[cfg(test)]
mod private_tests;

pub use core::config::{LatrConfig, RunMode};
pub use core::error::LatrError;
pub use core::latr_core::LatrEngine;
pub use crate::engine::engine_core::{Engine, PhysicsLoop};