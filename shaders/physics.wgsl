// physics.wgsl
struct PhysicsParams {
    delta_time: f32,
    gravity: f32,
    width: f32,
    height: f32,
    dot_radius: f32,
    dots_count: u32,
};

@group(0) @binding(0)
var<uniform> params: PhysicsParams;

@group(0) @binding(1)
var<storage, read_write> dots: array<Dot>;

struct Dot {
    position: vec2<f32>,
    velocity: vec2<f32>,
    mass: f32,
    state: u32,  // 0: solid, 1: liquid, 2: gas
    temperature: f32,
    density: f32,
    viscosity: f32,
    elasticity: f32,
    cohesion: f32,
    entropy_bias: f32,
    luminescence: f32,
    heat_capacity_high: f32,
    heat_capacity_low: f32,
    heat_conductivity: f32,
    hardness: f32,
    volatility: f32,
    id: u32,
    reaction_count: u32,
    is_selected: u32,
};

const DOT_RADIUS: f32 = 2.0f;
const GAS_REFERENCE_DENSITY: f32 = 0.5f;
const GAS_DIFFUSION_FACTOR: f32 = 5.0f;
const COOL_DOWN_SECONDS: f32 = 1.0f;
const INITIAL_WAIT_TIME: f32 = 0.1f;
const DECAY_FACTOR: f32 = 0.5f;

fn update_state(dot_data: Dot, gravity: f32, dt: f32) -> Dot {
    var dot = dot_data;
    // 状態変化と爆発の検出
    if (dot.state == 2u) { // Gas
        let buoyancy = (GAS_REFERENCE_DENSITY - dot.density) * gravity;
        dot.velocity.y -= buoyancy * dt;
        let diffusion_strength = (1.0f - dot.viscosity) * GAS_DIFFUSION_FACTOR;
        dot.velocity.x += (fract(sin(f32(dot.id) * 12.9898f + 78.233f) * 43758.5453f) - 0.5f) * diffusion_strength * dt;
        dot.velocity.y += (fract(cos(f32(dot.id) * 4.8980f + 59.123f) * 43758.5453f) - 0.5f) * diffusion_strength * dt;
    } else {
        dot.velocity.y += gravity * dt;
    }

    // 温度変化
    if dot.temperature > dot.heat_capacity_high {
        dot.heat_conductivity += 0.1 * dt;
        if dot.heat_conductivity > 1.0 {
            if dot.volatility >= 0.5 {
                if dot.state == 0u { // Solid
                    dot.state = 1u; // Liquid
                    dot.heat_capacity_high = fract(sin(f32(dot.id) * 12.9898f + 78.233f) * 43758.5453f);
                    dot.temperature = dot.heat_capacity_high * fract(cos(f32(dot.id) * 4.8980f + 59.123f) * 43758.5453f);
                    dot.heat_conductivity = fract(sin(f32(dot.id) * 73.8467f + 12.345f) * 43758.5453f);
                } else if dot.state == 1u { // Liquid
                    dot.state = 2u; // Gas
                    dot.heat_capacity_high = fract(sin(f32(dot.id) * 12.9898f + 78.233f) * 43758.5453f);
                    dot.temperature = dot.heat_capacity_high * fract(cos(f32(dot.id) * 4.8980f + 59.123f) * 43758.5453f);
                    dot.heat_conductivity = fract(sin(f32(dot.id) * 73.8467f + 12.345f) * 43758.5453f);
                } else if dot.state == 2u { // Gas
                    dot.luminescence = 1.0f;
                    // 発光状態の処理はここでは省略
                }
            }
        }
    }

    if dot.temperature < -dot.heat_capacity_low {
        if dot.state == 2u { // Gas
            dot.state = 1u; // Liquid
            dot.heat_capacity_low = fract(sin(f32(dot.id) * 12.9898f + 78.233f) * 43758.5453f) - 1.0f;
            dot.temperature = -dot.heat_capacity_low * fract(cos(f32(dot.id) * 4.8980f + 59.123f) * 43758.5453f);
            dot.heat_conductivity = fract(sin(f32(dot.id) * 73.8467f + 12.345f) * 43758.5453f);
        } else if dot.state == 1u { // Liquid
            dot.state = 0u; // Solid
            dot.heat_capacity_low = fract(sin(f32(dot.id) * 12.9898f + 78.233f) * 43758.5453f) - 1.0f;
            dot.temperature = -dot.heat_capacity_low * fract(cos(f32(dot.id) * 4.8980f + 59.123f) * 43758.5453f);
            dot.heat_conductivity = fract(sin(f32(dot.id) * 73.8467f + 12.345f) * 43758.5453f);
        }
    }

    return dot;
}

fn update_position(dot_data: Dot) -> Dot {
    var dot = dot_data;
    dot.position += dot.velocity * params.delta_time;

    let elasticity = dot.elasticity;

    // 壁との衝突処理
    if dot.position.y >= (params.height - params.dot_radius) {
        dot.position.y = params.height - params.dot_radius;
        dot.velocity.y *= -elasticity;

        // 摩擦
        let friction_factor = dot.viscosity * 0.7f;
        dot.velocity.x *= max(0.0f, 1.0f - friction_factor);
    }

    if dot.position.y <= params.dot_radius {
        dot.position.y = params.dot_radius;
        dot.velocity.y *= -elasticity;
    }

    if dot.position.x >= (params.width - params.dot_radius) {
        dot.position.x = params.width - params.dot_radius;
        dot.velocity.x *= -elasticity;
    }

    if dot.position.x <= params.dot_radius {
        dot.position.x = params.dot_radius;
        dot.velocity.x *= -elasticity;
    }

    // 減衰
    let damping_factor = 0.998f;
    dot.velocity *= damping_factor;

    return dot;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.dots_count) {
        return;
    }

    var dot = dots[idx];

    // 状態更新
    dot = update_state(dot, params.gravity, params.delta_time);

    // 位置更新
    dot = update_position(dot);

    dots[idx] = dot;
}