@group(0) @binding(0) var<uniform> uniforms: DotUniforms;

struct DotUniforms {
    time: f32,
    max_entropy_bias: f32,
    _padding: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) is_selected: f32,
    @location(2) local_pos: vec2<f32>,
    @location(3) luminescence: f32,
    @location(4) temperature: f32,
    @location(5) state: f32,
    @location(6) cohesion: f32,
    @location(7) entropy_bias: f32,
}

struct FragmentOutput {
    @location(0) scene: vec4<f32>,
    @location(1) glow: vec4<f32>,
}

// 頂点シェーダー
const DOT_RADIUS: f32 = 2.0;

// xorShift
fn xorshift32(p: vec2<u32>) -> u32 {
    var x = p.x * 12345u + p.y * 67890u;
    x = x ^ (x << 13u);
    x = x ^ (x >> 17u);
    x = x ^ (x << 5u);
    return x;
}

// u32 を [0, 1] の f32 に変換
fn to_f32_range01(n: u32) -> f32 {
    return (f32(n) / f32(0xFFFFFFFFu));
}

// 2Dベクトルから単純な擬似乱数を生成する
fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// --- 色空間変換 ---
fn rgb2hsv(c: vec3<f32>) -> vec3<f32> {
    let c_max = max(c.r, max(c.g, c.b));
    let c_min = min(c.r, min(c.g, c.b));
    let diff = c_max - c_min;
    var h: f32 = -1.0;
    var s: f32 = -1.0;

    if (c_max == c_min) {
        h = 0.0;
    } else if (c_max == c.r) {
        h = 60.0 * (0.0 + (c.g - c.b) / diff);
    } else if (c_max == c.g) {
        h = 60.0 * (2.0 + (c.b - c.r) / diff);
    } else if (c_max == c.b) {
        h = 60.0 * (4.0 + (c.r - c.g) / diff);
    }

    if (h < 0.0) { h += 360.0; }

    if (c_max == 0.0) {
        s = 0.0;
    } else {
        s = diff / c_max;
    }

    let v = c_max;
    return vec3<f32>(h, s, v);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let h = c.x;
    let s = c.y;
    let v = c.z;
    let hi = i32(floor(h / 60.0)) % 6;
    let f = h / 60.0 - floor(h / 60.0);
    let p = v * (1.0 - s);
    let q = v * (1.0 - (f * s));
    let t = v * (1.0 - ((1.0 - f) * s));

    if (hi == 0) { return vec3<f32>(v, t, p); }
    if (hi == 1) { return vec3<f32>(q, v, p); }
    if (hi == 2) { return vec3<f32>(p, v, t); }
    if (hi == 3) { return vec3<f32>(p, q, v); }
    if (hi == 4) { return vec3<f32>(t, p, v); }
    return vec3<f32>(v, p, q);
}


@vertex
fn vs_main(
    @location(0) vertex_offset: vec2<f32>,
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_color: vec3<f32>,
    @location(3) instance_luminescence: f32,
    @location(4) instance_is_selected: f32,
    @location(5) instance_temperature: f32,
    @location(6) instance_state: f32,
    @location(7) instance_cohesion: f32,
    @location(8) instance_entropy_bias: f32,
) -> VertexOutput {
    var final_pos = instance_position + (vertex_offset * DOT_RADIUS);

    // --- 状態やプロパティに応じた揺れ ---
    let seed_pos = vec2<u32>(u32(instance_position.x), u32(instance_position.y));
    let seed_time = u32(uniforms.time * 10.0);
    
    var total_shake_amount = 0.0;
    
    // Gasの揺れ
    if (instance_state > 1.5) {
        total_shake_amount += 0.5;
    }

    // entropy_biasの揺れ
    if (instance_entropy_bias > 0.5) {
        let effect_strength = (uniforms.max_entropy_bias - instance_entropy_bias) * 4.0; // 係数
        total_shake_amount += effect_strength;
    }

    let time_wave_shake = sin(uniforms.time + f32(seed_pos.x));
    if (total_shake_amount > 0.0 && time_wave_shake > 0.95) { // ピーク時のみ揺らす
        let rand_x = (to_f32_range01(xorshift32(seed_pos + vec2<u32>(seed_time, 0u))) - 0.5) * 2.0;
        let rand_y = (to_f32_range01(xorshift32(seed_pos + vec2<u32>(0u, seed_time))) - 0.5) * 2.0;
        final_pos += vec2<f32>(rand_x, rand_y) * total_shake_amount;
    }
    
    var ndc_pos = vec2<f32>(
        (final_pos.x / 640.0) * 2.0 - 1.0,
        1.0 - (final_pos.y / 480.0) * 2.0
    );
    
    var output: VertexOutput;
    output.clip_position = vec4<f32>(ndc_pos, 0.0, 1.0);
    output.color = instance_color;
    output.luminescence = instance_luminescence;
    output.is_selected = instance_is_selected;
    output.temperature = instance_temperature;
    output.state = instance_state;
    output.local_pos = vertex_offset;
    output.cohesion = instance_cohesion;
    output.entropy_bias = instance_entropy_bias;
    
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let dist_sq = dot(in.local_pos, in.local_pos);

    if (in.state < 0.5) { // Solid
        if (abs(in.local_pos.x) > 1.0 || abs(in.local_pos.y) > 1.0) { discard; }
    } else { // Liquid or Gas
        if (dist_sq > 1.0) { discard; }
    }

    let RED = vec3<f32>(1.0, 0.1, 0.1);
    let ORANGE = vec3<f32>(1.0, 0.5, 0.1);
    let WHITE = vec3<f32>(1.0, 1.0, 1.0);
    let LIGHT_BLUE = vec3<f32>(0.8, 0.9, 1.0);

    var output: FragmentOutput;
    var scene_color = vec4<f32>(in.color, 1.0);
    var glow_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // entropy_biasによる色不安定化
    let time_wave_color = sin(uniforms.time * 1.5 + in.clip_position.y * 5.0);
    if (in.entropy_bias > 0.5 && time_wave_color > 0.9) {
        let effect_strength = (uniforms.max_entropy_bias - in.entropy_bias);
        var hsv = rgb2hsv(scene_color.rgb);
        
        let seed1 = in.clip_position.xy + uniforms.time;
        let seed2 = in.clip_position.yx * 1.5 + uniforms.time * 0.8;
        let seed3 = (in.clip_position.xy - uniforms.time) * 0.5;

        let hue_shift = (rand(seed1) - 0.5) * 90.0; // 色相の変化を±45度まで
        let sat_factor = 1.0 + (rand(seed2) - 0.5) * 1.0;  // 彩度の変化を±50%まで
        let val_factor = 1.0 + (rand(seed3) - 0.5) * 0.6;  // 明度の変化を±30%まで

        hsv.x = (hsv.x + hue_shift * effect_strength);
        hsv.y *= mix(1.0, sat_factor, effect_strength);
        hsv.z *= mix(1.0, val_factor, effect_strength);
        
        scene_color = vec4<f32>(hsv2rgb(hsv), scene_color.a);
    }

    // 発光処理
    if (in.luminescence > 0.8) {
        var glow_intensity = (in.luminescence - 0.8) / 0.2;
        if (in.state > 1.5) { // Gas
            let flicker_strength = select(0.0, 0.5 - in.cohesion, in.cohesion < 0.5);
            let flicker = rand(in.clip_position.xy + uniforms.time) * flicker_strength + (1.0 - flicker_strength);
            glow_intensity *= flicker;
        } else if (in.state > 0.5) { // Liquid
            glow_intensity *= 1.5;
        }

        let temp_norm = (in.temperature + 1.0) / 2.0;
        var temp_color: vec3<f32>;
        if (temp_norm < 0.33) { temp_color = mix(RED, ORANGE, temp_norm / 0.33); }
        else if (temp_norm < 0.66) { temp_color = mix(ORANGE, WHITE, (temp_norm - 0.33) / 0.33); }
        else { temp_color = mix(WHITE, LIGHT_BLUE, (temp_norm - 0.66) / 0.34); }
        glow_color = vec4<f32>(temp_color * glow_intensity, 1.0);
        scene_color.r += glow_color.r * 0.2;
        scene_color.g += glow_color.g * 0.2;
        scene_color.b += glow_color.b * 0.2;
    }

    // 選択時の縁取り
    if (in.is_selected > 0.5) {
        let border_thickness = 0.4;
        let inner_edge = 1.0 - border_thickness;
        if dist_sq > inner_edge * inner_edge {
            scene_color = vec4<f32>(vec3<f32>(1.0, 1.0, 1.0) - scene_color.rgb, scene_color.a);
        }
    }
    
    // cohesionが低いdotにノイズ
    if (in.cohesion < 0.5) {
        let noise_strength = (0.5 - in.cohesion) * 0.2;
        let noise_val = (rand(in.clip_position.xy) - 0.5) * noise_strength;
        scene_color.r += noise_val;
        scene_color.g += noise_val;
        scene_color.b += noise_val;
    }

    output.scene = scene_color;
    output.glow = glow_color;
    
    return output;
}