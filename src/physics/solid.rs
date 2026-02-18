use crate::{
    app::Dot,
};
use rand::thread_rng;
use rand::Rng;
use std::time::Instant;

use super::{DOT_RADIUS, HEIGHT, WIDTH}; // DOT_RADIUS, HEIGHT, WIDTH を親モジュールからインポート

// State::Solid に対する update_state 処理
pub fn update_state_for_solid(dot: &mut Dot, gravity: f64, dt: f64) {
    // State::Solid は重力の影響を受ける
    dot.vy += gravity * dt;

    // 状態変化は update_state 関数全体で共通のロジックなので、
    // ここでは固体特有の状態変化処理は特にない
    // (高温で液体になるなどは update_state 全体で行う)

    // 低温時の状態変化 (plan.md L105-L111)
    // State::Solid -> State::Liquid は update_state 全体で処理
    // State::Solid -> 崩壊 (0.1%の確率)
    let mut rng = thread_rng();
    if dot.material.temperature < -dot.material.heat_capacity_low {
        // クールダウンチェック
        if dot.last_check_time.elapsed().as_secs_f64() > super::COOL_DOWN_SECONDS {
            if rng.gen::<f32>() < 0.001 {
                // 0.1%の確率で崩壊 (この処理はengine.rs側でdots_to_removeに追加する)
                // ここでは単に崩壊フラグを返すか、engine.rs側で処理する
                // 一旦、何もしない。engine.rs側で処理。
            }
            // 確率判定を行ったら時刻を更新
            dot.last_check_time = Instant::now();
        }
    }
}

// State::Solid に対する update_position 処理
pub fn update_position_for_solid(dot: &mut Dot, dt: f64) {
    let mut rng = thread_rng();
    let elasticity = dot.material.elasticity as f64;

    // 境界との衝突処理
    if dot.y >= (HEIGHT as f64 - DOT_RADIUS) {
        dot.y = HEIGHT as f64 - DOT_RADIUS;

        // Solids bounce with elasticity and stop when velocity is low
        dot.vy *= -elasticity;
        // 床との摩擦を適用
        let friction_factor = dot.material.viscosity as f64 * 0.7; // 係数を調整
        dot.vx *= (1.0 - friction_factor).max(0.0);
        // Particles behave similarly to solids but with more spreading
        if dot.material.viscosity < 0.5 {
            let spread_factor = (1.0 - dot.material.viscosity as f64) * 1.5;
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

// State::Solid に対する衝突処理 (詳細な衝突処理の一部)
pub fn handle_collision_for_solid(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    // 既存の handle_detailed_collision の固体部分をここに移植
    // ただし、引数が2つのドットなので、両方の状態がSolidであることを前提とする
    // または、Solid-Liquid, Solid-Gas の処理もここに含めるか別関数にするか考える必要がある

    // ここでは、Solid-Solid の衝突処理を実装する
    // (handle_detailed_collision の Solid 関連のコードを移植)

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

    // --- 凝集力 ---
    // 両方Solidの場合のみ凝集力が働く
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

    // Solid間の粘度に基づく広がり処理 (handle_solid_spreading)
    let avg_hardness = (dot1.material.hardness + dot2.material.hardness) / 2.0;

    // --- 摩擦処理 ---
    // 接線方向のベクトル (tangent)
    let tx = -ny;
    let ty = nx;

    // 接線方向の相対速度
    let v_rel_t = (dot2.vx - dot1.vx) * tx + (dot2.vy - dot1.vy) * ty;

    // 摩擦による速度変化量。v_rel_tを0に近づける方向に力を加える
    // 粘度が高いほど強くなる
    let friction_impulse = v_rel_t * avg_viscosity as f64 * 0.5; // 係数は要調整

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

    // --- 広がりと圧力の処理 ---
    // 粘度が低いほどわずかに広がる力を持たせる
    if avg_viscosity < 0.8 && avg_hardness < 0.5 {
        let spread_factor = (1.0 - avg_viscosity) * (1.0 - avg_hardness) * 0.01; // 係数をさらに小さく
        // ほぼ上下の衝突のときだけ、わずかに広げる
        if ny.abs() > 0.8 {
            let spread_force = spread_factor as f64 * dt;
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
    let vertical_factor = avg_viscosity as f64 * 0.05;
    dot1.vy += vertical_factor * dt;
    dot2.vy += vertical_factor * dt;
}