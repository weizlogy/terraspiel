use crate::app::Dot;
use crate::material::State;
use std::time::Instant;
use rand::thread_rng;
use rand::Rng;

// physics/engine.rs で定義されている定数をインポート
use super::{DOT_RADIUS, HEIGHT, WIDTH, COOL_DOWN_SECONDS, GAS_REFERENCE_DENSITY, GAS_DIFFUSION_FACTOR};

// 各状態モジュールをインポート
use super::{solid, liquid, gas};

// State に応じた update_state 処理を呼び分ける
pub fn update_state_for_dot(dot: &mut Dot, gravity: f64, dt: f64) {
    match dot.material.state {
        State::Solid => solid::update_state_for_solid(dot, gravity, dt),
        State::Liquid => liquid::update_state_for_liquid(dot, gravity, dt),
        State::Gas => gas::update_state_for_gas(dot, gravity, dt),
    }
}

// State に応じた update_position 処理を呼び分ける
pub fn update_position_for_dot(dot: &mut Dot, dt: f64) {
    match dot.material.state {
        State::Solid => solid::update_position_for_solid(dot, dt),
        State::Liquid => liquid::update_position_for_liquid(dot, dt),
        State::Gas => gas::update_position_for_gas(dot, dt),
    }
}

// 2つのドットの状態に応じた衝突処理を呼び分ける
pub fn handle_collision_between_states(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    match (dot1.material.state, dot2.material.state) {
        (State::Solid, State::Solid) => {
            solid::handle_collision_for_solid(dot1, dot2, nx, ny, dt);
            // Solid-Solid の場合の蓄積や広がり処理 (handle_solid_spreading相当)
            // これは solid.rs に追加するか、state_manager.rs で行う
            // 今回は state_manager.rs で行う
            // handle_solid_spreading のコードをここに移植
            let avg_viscosity = (dot1.material.viscosity + dot2.material.viscosity) / 2.0;
            let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

            // --- 摩擦処理 ---
            let tx = -ny;
            let ty = nx;
            let v_rel_t = (dot2.vx - dot1.vx) * tx + (dot2.vy - dot1.vy) * ty;
            let friction_impulse = v_rel_t * avg_viscosity as f64 * 0.5;
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
            if avg_viscosity < 0.8 && avg_hardness < 0.5 {
                let spread_factor = (1.0 - avg_viscosity as f64) * (1.0 - avg_hardness as f64) * 0.01;
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

            let vertical_factor = avg_viscosity as f64 * 0.05;
            dot1.vy += vertical_factor * dt;
            dot2.vy += vertical_factor * dt;
        }
        (State::Liquid, State::Liquid) => {
            liquid::handle_collision_for_liquid(dot1, dot2, nx, ny, dt);
            // Liquid-Liquid の場合の蓄積処理 (handle_liquid_accumulation相当)
            // これは liquid.rs に追加するか、state_manager.rs で行う
            // 今回は state_manager.rs で行う
            // handle_liquid_accumulation のコードをここに移植
            let near_bottom = dot1.y >= (HEIGHT as f64 - DOT_RADIUS - 5.0)
                || dot2.y >= (HEIGHT as f64 - DOT_RADIUS - 5.0);

            if near_bottom {
                let avg_viscosity_calc = ((dot1.material.viscosity + dot2.material.viscosity) / 2.0) as f64;
                let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

                let spread_factor = (1.0 - avg_viscosity_calc) * (1.0 - avg_hardness) * 0.5;

                if nx.abs() > ny.abs() {
                    if dot1.x < dot2.x {
                        dot1.vx -= spread_factor * dt * 10.0;
                        dot2.vx += spread_factor * dt * 10.0;
                    } else {
                        dot1.vx += spread_factor * dt * 10.0;
                        dot2.vx -= spread_factor * dt * 10.0;
                    }
                }

                let vertical_factor = avg_viscosity_calc * 0.1;
                dot1.vy += vertical_factor * dt * 5.0;
                dot2.vy += vertical_factor * dt * 5.0;
            }
        }
        (State::Gas, State::Gas) => {
            gas::handle_collision_for_gas(dot1, dot2, nx, ny);
        }
        (State::Solid, State::Liquid) => {
            // Solid-Liquid 衝突処理 (engine.rs の元の処理を参考に)
            if dot1.material.density > dot2.material.density
                && dot1.material.viscosity > dot2.material.viscosity
            {
                let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;
                let v_liquid_n = dot2.vx * nx + dot2.vy * ny;
                if v_liquid_n < 0.0 {
                    dot2.vx -= (1.0 + e) * v_liquid_n * nx;
                    dot2.vy -= (1.0 + e) * v_liquid_n * ny;
                }
            } else {
                // 互いに影響を与える処理 (例: Solid-Liquid 間の熱交換、凝集力)
                // ここでは、liquid の衝突処理を適用する (これは仮の実装)
                liquid::handle_collision_for_liquid(dot1, dot2, nx, ny, dt);
            }
        }
        (State::Liquid, State::Solid) => {
            // Liquid-Solid 衝突処理
            if dot2.material.density > dot1.material.density
                && dot2.material.viscosity > dot1.material.viscosity
            {
                let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;
                let v_liquid_n = dot1.vx * (-nx) + dot1.vy * (-ny);
                if v_liquid_n < 0.0 {
                    dot1.vx -= (1.0 + e) * v_liquid_n * (-nx);
                    dot1.vy -= (1.0 + e) * v_liquid_n * (-ny);
                }
            } else {
                // 互いに影響を与える処理 (例: Liquid-Solid 間の熱交換、凝集力)
                // ここでは、liquid の衝突処理を適用する (これは仮の実装)
                liquid::handle_collision_for_liquid(dot1, dot2, nx, ny, dt);
            }
        }
        (State::Solid, State::Gas) => {
            // Solid が Gas に押される
            gas::handle_displacement_for_gas(dot2, dot1, nx, ny);
        }
        (State::Liquid, State::Gas) => {
            // Liquid が Gas に押される
            gas::handle_displacement_for_gas(dot2, dot1, nx, ny);
        }
        (State::Gas, State::Solid) => {
            // Gas が Solid に押される
            gas::handle_displacement_for_gas(dot1, dot2, nx, ny);
        }
        (State::Gas, State::Liquid) => {
            // Gas が Liquid に押される
            gas::handle_displacement_for_gas(dot1, dot2, nx, ny);
        }
    }
}

// 低温時の状態変化処理 (State::Solid に特化)
// これは update_state_for_dot から呼び出すか、state_manager.rs で共通処理として定義する
// 今回は共通処理として定義し、update_state_for_dot から呼び出す
pub fn handle_cool_down_for_solid(dot: &mut Dot) {
    let mut rng = thread_rng();
    // State::Solid のみが崩壊の対象
    if dot.material.state == State::Solid && dot.material.temperature < -dot.material.heat_capacity_low {
        // クールダウンチェック
        if dot.last_check_time.elapsed().as_secs_f64() > COOL_DOWN_SECONDS {
            if rng.gen::<f32>() < 0.001 {
                // 0.1%の確率で崩壊 (engine.rs側でdots_to_removeに追加する)
                // ここでは単に崩壊フラグを返すか、engine.rs側で処理する
                // 一旦、何もしない。engine.rs側で処理。
            }
            // 確率判定を行ったら時刻を更新
            dot.last_check_time = Instant::now();
        }
    }
}