use crate::app::Dot;
use crate::material::State;
use crate::physics::engine::DOT_RADIUS;
use crate::physics::HEAT_TRANSFER_COEFFICIENT;

// 熱交換の最小間隔（秒）
const HEAT_EXCHANGE_INTERVAL: f64 = 0.1; // 0.1 秒に 1 回だけ熱交換

// Solid/Liquid 間の詳細な衝突処理
pub fn handle_detailed_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
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
    // 熱交換の頻度を制限
    let now = std::time::Instant::now();
    let elapsed1 = now.duration_since(dot1.last_heat_exchange_time).as_secs_f64();
    let elapsed2 = now.duration_since(dot2.last_heat_exchange_time).as_secs_f64();
    
    if elapsed1 >= HEAT_EXCHANGE_INTERVAL && elapsed2 >= HEAT_EXCHANGE_INTERVAL {
        let temp_diff = dot1.material.temperature - dot2.material.temperature;
        let avg_heat_conductivity =
            (dot1.material.heat_conductivity + dot2.material.heat_conductivity) / 2.0;
        let heat_transfer = (temp_diff * avg_heat_conductivity * HEAT_TRANSFER_COEFFICIENT).clamp(-1.0, 1.0); // NaN ガード

        // エネルギー保存：dot1 が失う熱 = dot2 が得る熱
        dot1.material.temperature = (dot1.material.temperature - heat_transfer).clamp(-1.0, 1.0);
        dot2.material.temperature = (dot2.material.temperature + heat_transfer).clamp(-1.0, 1.0);

        // 熱交換時刻を更新
        dot1.last_heat_exchange_time = now;
        dot2.last_heat_exchange_time = now;
    }

    // --- 凝集力 ---
    if dot1.material.state == dot2.material.state {
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
    }

    // Liquids spread based on viscosity
    if dot1.material.state == State::Liquid && dot2.material.state == State::Liquid {
        let viscosity_threshold = 0.5; // Liquids require lower viscosity to spread

        if avg_viscosity < viscosity_threshold && ny.abs() > 0.8 {
            let spread_factor = (1.0 - avg_viscosity as f64) * 10.0; // Normal spreading force for liquids
            let spread_force = spread_factor * dt;

            if dot1.x < dot2.x {
                dot1.vx -= spread_force;
                dot2.vx += spread_force;
            } else {
                dot1.vx += spread_force;
                dot2.vx -= spread_force;
            }
        }
    }
}

// Gas 間の衝突処理
pub fn handle_gas_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
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
    // 熱交換の頻度を制限
    let now = std::time::Instant::now();
    let elapsed1 = now.duration_since(dot1.last_heat_exchange_time).as_secs_f64();
    let elapsed2 = now.duration_since(dot2.last_heat_exchange_time).as_secs_f64();
    
    if elapsed1 >= HEAT_EXCHANGE_INTERVAL && elapsed2 >= HEAT_EXCHANGE_INTERVAL {
        let temp_diff = dot1.material.temperature - dot2.material.temperature;
        let avg_heat_conductivity =
            (dot1.material.heat_conductivity + dot2.material.heat_conductivity) / 2.0;
        let heat_transfer = (temp_diff * avg_heat_conductivity * HEAT_TRANSFER_COEFFICIENT).clamp(-1.0, 1.0); // NaN ガード

        // エネルギー保存：dot1 が失う熱 = dot2 が得る熱
        dot1.material.temperature = (dot1.material.temperature - heat_transfer).clamp(-1.0, 1.0);
        dot2.material.temperature = (dot2.material.temperature + heat_transfer).clamp(-1.0, 1.0);

        // 熱交換時刻を更新
        dot1.last_heat_exchange_time = now;
        dot2.last_heat_exchange_time = now;
    }
}

// Gas が他の物体に押される処理
pub fn handle_gas_displacement(gas: &mut Dot, other: &Dot, nx: f64, ny: f64) {
    let e = (gas.material.elasticity + other.material.elasticity) as f64 / 2.0;

    let v_gas_n = gas.vx * nx + gas.vy * ny;

    // nx,ny は常に gas->other を指すため、v_gas_n が正なら向かっている
    if v_gas_n > 0.0 {
        gas.vx -= (1.0 + e) * v_gas_n * nx;

        gas.vy -= (1.0 + e) * v_gas_n * ny;
    }
}

// 液体の蓄積の処理
pub fn handle_liquid_accumulation(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    // Only apply accumulation if both dots are near the bottom
    let near_bottom = dot1.y >= (crate::app::HEIGHT as f64 - DOT_RADIUS - 5.0)
        || dot2.y >= (crate::app::HEIGHT as f64 - DOT_RADIUS - 5.0);

    if !near_bottom {
        return;
    }

    // Calculate average viscosity of the two liquid dots (convert f32 to f64)
    let avg_viscosity = ((dot1.material.viscosity + dot2.material.viscosity) / 2.0) as f64;
    let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

    // Calculate how much to spread based on viscosity (less viscous spreads more)
    let spread_factor = (1.0 - avg_viscosity) * (1.0 - avg_hardness) * 0.5; // Reduced factor to make it more natural

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
    let vertical_factor = avg_viscosity * 0.1; // Very small vertical movement
    dot1.vy += vertical_factor * dt * 5.0;
    dot2.vy += vertical_factor * dt * 5.0;
}

// 固体間の粘度に基づく広がり処理
pub fn handle_solid_spreading(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    let avg_viscosity = ((dot1.material.viscosity + dot2.material.viscosity) / 2.0) as f64;

    // --- 摩擦処理 ---
    // 接線方向のベクトル (tangent)
    let tx = -ny;
    let ty = nx;

    // 接線方向の相対速度
    let v_rel_t = (dot2.vx - dot1.vx) * tx + (dot2.vy - dot1.vy) * ty;

    // 摩擦による速度変化量。v_rel_t を 0 に近づける方向に力を加える
    // 粘度が高いほど強くなる
    let friction_impulse = v_rel_t * avg_viscosity * 0.5; // 係数は要調整

    // 質量に応じて摩擦力積を適用
    let m1 = dot1.material.density as f64;
    let m2 = dot2.material.density as f64;
    let total_mass = m1 + m2;
    if total_mass > 1e-6 {
        dot1.vx += friction_impulse * (m2 / total_mass) * tx;
        dot1.vy += friction_impulse * (m2 / total_mass) * ty;
        dot2.vx -= friction_impulse * (m1 / total_mass) * tx;
        dot2.vy -= friction_impulse * (m1 / total_mass) * ty;
    }

    let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

    // --- 広がりと圧力の処理 ---
    // 粘度が低いほどわずかに広がる力を持たせる
    if avg_viscosity < 0.8 && avg_hardness < 0.5 {
        let spread_factor = (1.0 - avg_viscosity) * (1.0 - avg_hardness) * 0.01; // 係数をさらに小さく
                                                                                 // ほぼ上下の衝突のときだけ、わずかに広げる
        if ny.abs() > 0.8 {
            let spread_force = spread_factor * dt;
            if dot1.x < dot2.x {
                dot1.vx -= spread_force;
                dot2.vx += spread_force;
            } else {
                dot1.vx += spread_force;
                dot2.vx -= spread_force;
            }
        }
    }

    // 縦方向の圧力。粘度が高いほど強くかかる
    // これが積み重なる効果を生むはず
    let vertical_factor = avg_viscosity * 0.05;
    dot1.vy += vertical_factor * dt;
    dot2.vy += vertical_factor * dt;
}
