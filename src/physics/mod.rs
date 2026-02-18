pub mod collision_helpers;
pub mod engine;
pub mod gas;
pub mod liquid;
pub mod solid;
pub mod state_manager;

pub use engine::{Physics, update_state, update_position, DOT_RADIUS, COOL_DOWN_SECONDS, GAS_REFERENCE_DENSITY, GAS_DIFFUSION_FACTOR, HEIGHT, WIDTH};
pub use state_manager::{update_state_for_dot, update_position_for_dot, handle_collision_between_states, handle_cool_down_for_solid};

// 熱交換係数
pub const HEAT_TRANSFER_COEFFICIENT: f32 = 0.001;
