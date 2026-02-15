use crate::{
    app::Dot,
};
use rand::thread_rng;
use rand::Rng;

use super::{DOT_RADIUS, HEIGHT, WIDTH}; // DOT_RADIUS, HEIGHT, WIDTH を親モジュールからインポート

// State::Liquid に対する update_state 処理
pub fn update_state_for_liquid(dot: &mut Dot, gravity: f64, dt: f64) {
    // State::Liquid は重力の影響を受ける
    dot.vy += gravity * dt;

    // 状態変化は update_state 関数全体で共通のロジックなので、
    // ここでは液体特有の状態変化処理は特にない
    // (高温で気体になるなどは update_state 全体で行う)
}

// State::Liquid に対する update_position 処理
pub fn update_position_for_liquid(dot: &mut Dot, dt: f64) {
    let mut rng = thread_rng();
    let elasticity = dot.material.elasticity as f64;

    // 境界との衝突処理
    if dot.y >= (HEIGHT as f64 - DOT_RADIUS) {
        dot.y = HEIGHT as f64 - DOT_RADIUS;

        // Liquids spread based on viscosity
        dot.vy *= -elasticity * (1.0 - dot.material.viscosity as f64); // More viscous liquids lose more vertical energy

        // Apply horizontal spreading based on viscosity
        if dot.material.viscosity < 0.7 {
            // Only spread if not highly viscous
            let spread_factor = (1.0 - dot.material.viscosity as f64) * 2.0;
            dot.vx += (rng.gen::<f64>() - 0.5) * spread_factor;
        }
    }

    if dot.y <= DOT_RADIUS {
        dot.y = DOT_RADIUS;

        // Apply viscosity effect when hitting top boundary too
        if dot.material.viscosity < 0.6 {
            let spread_factor = (1.0 - dot.material.viscosity as f64) * 0.3;
            dot.vx += (rng.gen::<f64>() - 0.5) * spread_factor * 0.3; // Very limited horizontal variability
        }
        dot.vy *= -elasticity;
    }

    if dot.x >= (WIDTH as f64 - DOT_RADIUS) {
        dot.x = WIDTH as f64 - DOT_RADIUS;

        // Apply viscosity effect when hitting side walls too
        if dot.material.viscosity < 0.6 {
            let spread_factor = (1.0 - dot.material.viscosity as f64) * 0.3;
            dot.vy += (rng.gen::<f64>() - 0.5) * spread_factor * 0.3; // Very limited vertical variability
        }
        dot.vx *= -elasticity;
    }

    if dot.x <= DOT_RADIUS {
        dot.x = DOT_RADIUS;

        // Apply viscosity effect when hitting side walls too
        if dot.material.viscosity < 0.6 {
            let spread_factor = (1.0 - dot.material.viscosity as f64) * 0.3;
            dot.vy += (rng.gen::<f64>() - 0.5) * spread_factor * 0.3; // Very limited vertical variability
        }
        dot.vx *= -elasticity;
    }

    // 減衰処理
    let damping_factor = 0.998; // 速度を99.8%に減衰
    dot.vx *= damping_factor;
    dot.vy *= damping_factor;
}

// State::Liquid に対する衝突処理 (詳細な衝突処理の一部)
pub fn handle_collision_for_liquid(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    // 既存の handle_detailed_collision の液体部分をここに移植
    // ただし、引数が2つのドットなので、両方の状態がLiquidであることを前提とする
    // または、Liquid-Liquid, Liquid-Solid, Liquid-Gas の処理もここに含めるか別関数にするか考える必要がある

    // ここでは、Liquid-Liquid の衝突処理を実装する
    // (handle_detailed_collision の Liquid 関連のコードを移植)

    let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;

    let m1 = dot1.material.density as f64 * (1.0 + dot1.material.hardness as f64);
    let m2 = dot2.material.density as f64 * (1.0 + dot2.material.hardness as f64);

    let v1n = dot1.vx * nx + dot1.vy * ny;
    let v2n = dot2.vx * nx + dot2.vy * ny;

    let v1n_new = (m1 * v1n + m2 * v2n - m2 * e * (v1n - v2n)) / (m1 + m2);
    let v2n_new = (m1 * v1n + m2 * v2n + m1 * e * (v1n - v2n)) / (m1 + m2);

    dot1.vx += (v1n_new - v1n) * nx;
    dot1.vy += (v1n_new - v1n) * ny;
    dot2.vx += (v2n_new - v2n) * nx;
    dot2.vy += (v2n_new - v2n) * ny;

    let density_diff = dot1.material.density - dot2.material.density;

    if density_diff.abs() > 0.1 {
        if dot1.y < dot2.y && density_diff > 0.0 {
            dot1.vy += density_diff as f64 * 5.0 * dt;
            dot2.vy -= density_diff as f64 * 5.0 * dt;
        } else if dot2.y < dot1.y && density_diff < 0.0 {
            dot2.vy += density_diff.abs() as f64 * 5.0 * dt;
            dot1.vy -= density_diff.abs() as f64 * 5.0 * dt;
        }
    }

    let avg_viscosity = (dot1.material.viscosity + dot2.material.viscosity) / 2.0;

    // --- 熱交換 ---
    let temp_diff = dot1.material.temperature - dot2.material.temperature;
    let avg_heat_conductivity =
        (dot1.material.heat_conductivity + dot2.material.heat_conductivity) / 2.0;
    let heat_transfer = (temp_diff * avg_heat_conductivity * 0.1).clamp(-1.0, 1.0); // NaNガード

    dot1.material.temperature = (dot1.material.temperature - heat_transfer).clamp(-1.0, 1.0);
    dot2.material.temperature = (dot2.material.temperature + heat_transfer).clamp(-1.0, 1.0);

    // --- 凝集力 ---
    // 両方Liquidの場合のみ凝集力が働く
    let avg_cohesion = (dot1.material.cohesion + dot2.material.cohesion) / 2.0;
    if avg_cohesion > 0.01 { // 計算負荷を減らすために閾値を設ける
        let ideal_dist = DOT_RADIUS * 1.5; // この距離に近づけようとする
        let dist_sq = (dot1.x - dot2.x).powi(2) + (dot1.y - dot2.y).powi(2);
        let dist = dist_sq.sqrt();

        // 凝集力が働く範囲 (e.g., DOT_RADIUS * 4)
        let effective_range = DOT_RADIUS * 4.0;

        if dist < effective_range && dist > 1e-6 {
            // 理想的な距離との差に基づいて力を計算
            let force_magnitude = (ideal_dist - dist) * (avg_cohesion as f64) * 0.01; // 係数は要調整

            let nx = (dot2.x - dot1.x) / dist;
            let ny = (dot2.y - dot1.y) / dist;

            let force_x = nx * force_magnitude;
            let force_y = ny * force_magnitude;

            // 質量に応じて力を適用
            let m1 = dot1.material.density as f64;
            let m2 = dot2.material.density as f64;
            let total_mass = m1 + m2;
            if total_mass > 1e-6 {
                dot1.vx += force_x * (m2 / total_mass);
                dot1.vy += force_y * (m2 / total_mass);
                dot2.vx -= force_x * (m1 / total_mass);
                dot2.vy -= force_y * (m1 / total_mass);
            }
        }
    }

    // Liquids spread based on viscosity (handle_detailed_collision 内の処理)
    if avg_viscosity < 0.5 { // Viscosity threshold for spreading
        let spread_factor = (1.0 - avg_viscosity as f64) * 10.0; // Normal spreading force for liquids
        let spread_force = spread_factor * dt;

        if ny.abs() > 0.8 { // Vertical collision
            if dot1.x < dot2.x {
                dot1.vx -= spread_force;
                dot2.vx += spread_force;
            } else {
                dot1.vx += spread_force;
                dot2.vx -= spread_force;
            }
        }
    }

    // 液体の蓄積の処理 (handle_liquid_accumulation)
    // Only apply accumulation if both dots are near the bottom
    let near_bottom = dot1.y >= (HEIGHT as f64 - DOT_RADIUS - 5.0)
        || dot2.y >= (HEIGHT as f64 - DOT_RADIUS - 5.0);

    if near_bottom {
        // Calculate average viscosity of the two liquid dots (convert f32 to f64)
        let avg_viscosity_calc = ((dot1.material.viscosity + dot2.material.viscosity) / 2.0) as f64;
        let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

        // Calculate how much to spread based on viscosity (less viscous spreads more)
        let spread_factor = (1.0 - avg_viscosity_calc) * (1.0 - avg_hardness) * 0.5; // Reduced factor to make it more natural

        // If the collision is more horizontal than vertical, apply spreading force
        if nx.abs() > ny.abs() {
            // Apply lateral spreading based on viscosity
            if dot1.x < dot2.x {
                dot1.vx -= spread_factor * dt * 10.0;
                dot2.vx += spread_factor * dt * 10.0;
            } else {
                dot1.vx += spread_factor * dt * 10.0;
                dot2.vx -= spread_factor * dt * 10.0;
            }
        }

        // Apply small vertical force to simulate liquid pressure
        // Less viscous liquids will have more vertical movement
        let vertical_factor = avg_viscosity_calc * 0.1; // Very small vertical movement
        dot1.vy += vertical_factor * dt * 5.0;
        dot2.vy += vertical_factor * dt * 5.0;
    }
}