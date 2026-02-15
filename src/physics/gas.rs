use crate::{
    app::Dot,
};
use rand::thread_rng;
use rand::Rng;

use super::{DOT_RADIUS, HEIGHT, WIDTH, GAS_REFERENCE_DENSITY, GAS_DIFFUSION_FACTOR}; // 必要な定数をインポート

// State::Gas に対する update_state 処理
pub fn update_state_for_gas(dot: &mut Dot, gravity: f64, dt: f64) {
    // State::Gas は浮力の影響を受ける
    let buoyancy = (GAS_REFERENCE_DENSITY - dot.material.density) as f64 * gravity;
    dot.vy -= buoyancy * dt;
    let diffusion_strength =
        (1.0 - dot.material.viscosity) as f64 * GAS_DIFFUSION_FACTOR;
    let mut rng = thread_rng();
    dot.vx += (rng.gen::<f64>() - 0.5) * diffusion_strength * dt;
    dot.vy += (rng.gen::<f64>() - 0.5) * diffusion_strength * dt;

    // 状態変化は update_state 関数全体で共通のロジックなので、
    // ここでは気体特有の状態変化処理は特にない
    // (高温で発光状態になるなどは update_state 全体で行う)
}

// State::Gas に対する update_position 処理
pub fn update_position_for_gas(dot: &mut Dot, dt: f64) {
    let elasticity = dot.material.elasticity as f64;

    // 境界との衝突処理
    if dot.y >= (HEIGHT as f64 - DOT_RADIUS) {
        dot.y = HEIGHT as f64 - DOT_RADIUS;

        // Gases should have minimal interaction with bottom, just bounce slightly
        dot.vy *= -elasticity * 0.1; // Very little bounce to keep gases moving
    }

    if dot.y <= DOT_RADIUS {
        dot.y = DOT_RADIUS;

        // For gases, minimal bounce to keep them moving
        dot.vy *= -elasticity * 0.1;
    }

    if dot.x >= (WIDTH as f64 - DOT_RADIUS) {
        dot.x = WIDTH as f64 - DOT_RADIUS;

        // Handle gas differently - allow more energy to be preserved
        dot.vx *= -elasticity * 0.3; // Gases preserve more horizontal momentum
    }

    if dot.x <= DOT_RADIUS {
        dot.x = DOT_RADIUS;

        // Handle gas differently - allow more energy to be preserved
        dot.vx *= -elasticity * 0.3; // Gases preserve more horizontal momentum
    }

    // 減衰処理
    let damping_factor = 0.998; // 速度を99.8%に減衰
    dot.vx *= damping_factor;
    dot.vy *= damping_factor;
}

// State::Gas に対する衝突処理 (Gas-Gas)
pub fn handle_collision_for_gas(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
    let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;

    let m1 = dot1.material.density as f64;
    let m2 = dot2.material.density as f64;

    let v1n = dot1.vx * nx + dot1.vy * ny;
    let v2n = dot2.vx * nx + dot2.vy * ny;

    let v1n_new = (m1 * v1n + m2 * v2n - m2 * e * (v1n - v2n)) / (m1 + m2);
    let v2n_new = (m1 * v1n + m2 * v2n + m1 * e * (v1n - v2n)) / (m1 + m2);

    dot1.vx += (v1n_new - v1n) * nx;
    dot1.vy += (v1n_new - v1n) * ny;
    dot2.vx += (v2n_new - v2n) * nx;
    dot2.vy += (v2n_new - v2n) * ny;

    // --- 熱交換 ---
    let temp_diff = dot1.material.temperature - dot2.material.temperature;
    let avg_heat_conductivity =
        (dot1.material.heat_conductivity + dot2.material.heat_conductivity) / 2.0;
    let heat_transfer = (temp_diff * avg_heat_conductivity * 0.1).clamp(-1.0, 1.0); // NaNガード

    dot1.material.temperature = (dot1.material.temperature - heat_transfer).clamp(-1.0, 1.0);
    dot2.material.temperature = (dot2.material.temperature + heat_transfer).clamp(-1.0, 1.0);
}

// Gasが他の物体に押される処理 (Gas-Other)
pub fn handle_displacement_for_gas(gas: &mut Dot, other: &Dot, nx: f64, ny: f64) {
    let e = (gas.material.elasticity + other.material.elasticity) as f64 / 2.0;

    let v_gas_n = gas.vx * nx + gas.vy * ny;

    // nx,nyは常に gas->other を指すため、v_gas_nが正なら向かっている
    if v_gas_n > 0.0 {
        gas.vx -= (1.0 + e) * v_gas_n * nx;
        gas.vy -= (1.0 + e) * v_gas_n * ny;
    }
}