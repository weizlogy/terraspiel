pub mod collision_helpers;
pub mod engine;
pub mod gas;
pub mod liquid;
pub mod solid;
pub mod state_manager;

pub use engine::{Physics, DOT_RADIUS, COOL_DOWN_SECONDS, GAS_REFERENCE_DENSITY, GAS_DIFFUSION_FACTOR, HEIGHT, WIDTH};

// 熱交換係数
pub const HEAT_TRANSFER_COEFFICIENT: f32 = 0.001;
