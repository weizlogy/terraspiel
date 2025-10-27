use crate::{
    app::{App, Dot, HEIGHT, WIDTH},
    material::State,
};
use rand::Rng;

const GAS_REFERENCE_DENSITY: f32 = 0.5;
const GAS_DIFFUSION_FACTOR: f64 = 5.0;

pub const DOT_RADIUS: f64 = 2.0;

pub struct Physics {}

impl Physics {
    pub fn new() -> Self {
        Physics {}
    }

    pub fn update_state(&self, app: &mut App, dt: f64) {
        let mut rng = rand::rng();

        for dot in &mut app.dots {
            match dot.material.state {
                State::Solid | State::Liquid | State::Particle => {
                    dot.vy += app.gravity * dt;
                }

                State::Gas => {
                    let buoyancy =
                        (GAS_REFERENCE_DENSITY - dot.material.density) as f64 * app.gravity;

                    dot.vy -= buoyancy * dt;

                    let diffusion_strength =
                        (1.0 - dot.material.viscosity) as f64 * GAS_DIFFUSION_FACTOR;

                    dot.vx += (rng.random::<f64>() - 0.5) * diffusion_strength * dt;

                    dot.vy += (rng.random::<f64>() - 0.5) * diffusion_strength * dt;
                }
            }
        }
    }

    pub fn update_collision(&self, app: &mut App, dt: f64) -> bool {
        let all_stopped = true;

        let num_dots = app.dots.len();

        for i in 0..num_dots {
            for j in (i + 1)..num_dots {
                let (dot1_slice, dot2_slice) = app.dots.split_at_mut(j);

                let dot1 = &mut dot1_slice[i];

                let dot2 = &mut dot2_slice[0];

                let dx = dot2.x - dot1.x;

                let dy = dot2.y - dot1.y;

                let distance_sq = dx * dx + dy * dy;

                let min_dist = DOT_RADIUS * 2.0;

                if distance_sq < min_dist * min_dist && distance_sq > 1e-6 {
                    let distance = distance_sq.sqrt();

                    let overlap = 0.5 * (min_dist - distance);

                    let nx = dx / distance;

                    let ny = dy / distance;

                    dot1.x -= overlap * nx;

                    dot1.y -= overlap * ny;

                    dot2.x += overlap * nx;

                    dot2.y += overlap * ny;

                    match (dot1.material.state, dot2.material.state) {
                        (State::Solid, State::Solid) | (State::Liquid, State::Liquid) => {
                            handle_detailed_collision(dot1, dot2, nx, ny, dt);
                        }

                        (State::Solid, State::Liquid) => {
                            if dot1.material.density > dot2.material.density
                                && dot1.material.viscosity > dot2.material.viscosity
                            {
                                // Solid is immovable, Liquid bounces off

                                let e = (dot1.material.elasticity + dot2.material.elasticity)
                                    as f64
                                    / 2.0;

                                let v_liquid_n = dot2.vx * nx + dot2.vy * ny;

                                if v_liquid_n < 0.0 {
                                    // Liquid moving towards Solid

                                    dot2.vx -= (1.0 + e) * v_liquid_n * nx;

                                    dot2.vy -= (1.0 + e) * v_liquid_n * ny;
                                }
                            } else {
                                handle_detailed_collision(dot1, dot2, nx, ny, dt);
                            }
                        }

                        (State::Liquid, State::Solid) => {
                            if dot2.material.density > dot1.material.density
                                && dot2.material.viscosity > dot1.material.viscosity
                            {
                                // Solid is immovable, Liquid bounces off

                                let e = (dot1.material.elasticity + dot2.material.elasticity)
                                    as f64
                                    / 2.0;

                                // Normal is from dot1->dot2. Liquid is dot1. Check against inverse normal.

                                let v_liquid_n = dot1.vx * (-nx) + dot1.vy * (-ny);

                                if v_liquid_n < 0.0 {
                                    // Liquid moving towards Solid

                                    dot1.vx -= (1.0 + e) * v_liquid_n * (-nx);

                                    dot1.vy -= (1.0 + e) * v_liquid_n * (-ny);
                                }
                            } else {
                                handle_detailed_collision(dot1, dot2, nx, ny, dt);
                            }
                        }

                        (State::Gas, State::Gas) => {
                            handle_gas_collision(dot1, dot2, nx, ny);
                        }

                        (State::Solid, State::Gas) | (State::Liquid, State::Gas) => {
                            handle_gas_displacement(dot2, dot1, nx, ny);
                        }

                        (State::Gas, State::Solid) | (State::Gas, State::Liquid) => {
                            handle_gas_displacement(dot1, dot2, nx, ny);
                        }

                        _ => {
                            handle_simple_collision(dot1, dot2, nx, ny);
                        }
                    }
                }
            }
        }
        all_stopped
    }

    pub fn update_position(&self, app: &mut App, dt: f64) -> bool {
        let mut all_stopped = true;

        for dot in &mut app.dots {
            dot.x += dot.vx * dt;

            dot.y += dot.vy * dt;

            let elasticity = dot.material.elasticity as f64;

            if dot.y >= (HEIGHT as f64 - DOT_RADIUS) {
                dot.y = HEIGHT as f64 - DOT_RADIUS;

                dot.vy *= -elasticity;
            }

            if dot.y <= DOT_RADIUS {
                dot.y = DOT_RADIUS;

                dot.vy *= -elasticity;
            }

            if dot.x >= (WIDTH as f64 - DOT_RADIUS) {
                dot.x = WIDTH as f64 - DOT_RADIUS;

                dot.vx *= -elasticity;
            }

            if dot.x <= DOT_RADIUS {
                dot.x = DOT_RADIUS;

                dot.vx *= -elasticity;
            }

            if dot.material.state != State::Gas {
                let velocity_small = dot.vy.abs() < 0.1 && dot.vx.abs() < 0.1;

                let at_bottom = dot.y >= (HEIGHT as f64 - DOT_RADIUS - 1.0);

                if !(velocity_small && at_bottom) {
                    all_stopped = false;
                }
            } else {
                all_stopped = false;
            }
        }
        all_stopped
    }
}

// --- ヘルパー関数 ---

// Solid/Liquid間の詳細な衝突処理

fn handle_detailed_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
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

    if avg_viscosity < 0.5 && ny.abs() > 0.8 {
        let spread_force = (1.0 - avg_viscosity) as f64 * 10.0 * dt;

        if dot1.x < dot2.x {
            dot1.vx -= spread_force;

            dot2.vx += spread_force;
        } else {
            dot1.vx += spread_force;

            dot2.vx -= spread_force;
        }
    }
}

// Gas間の衝突処理

fn handle_gas_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
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
}

// Gasが他の物体に押し出される処理

fn handle_gas_displacement(gas: &mut Dot, other: &Dot, nx: f64, ny: f64) {
    let e = (gas.material.elasticity + other.material.elasticity) as f64 / 2.0;

    let v_gas_n = gas.vx * nx + gas.vy * ny;

    // nx,nyは常に gas->other を指すため、v_gas_nが正なら向かっている

    if v_gas_n > 0.0 {
        gas.vx -= (1.0 + e) * v_gas_n * nx;

        gas.vy -= (1.0 + e) * v_gas_n * ny;
    }
}

// シンプルな速度交換

fn handle_simple_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
    let k = (dot2.vx - dot1.vx) * nx + (dot2.vy - dot1.vy) * ny;

    dot1.vx += k * nx;

    dot1.vy += k * ny;

    dot2.vx -= k * nx;

    dot2.vy -= k * ny;
}
