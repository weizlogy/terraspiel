@group(0) @binding(0) var<uniform> uniforms: DotUniforms;

struct DotUniforms {
    time: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) is_selected: f32,
    @location(2) local_pos: vec2<f32>,
    @location(3) luminescence: f32,
    @location(4) temperature: f32,
    @location(5) state: f32,
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

// u32 を [-1, 1] の f32 に変換
fn to_f32_range(n: u32) -> f32 {
    return (f32(n) / f32(0xFFFFFFFFu)) * 2.0 - 1.0;
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
) -> VertexOutput {
    var final_pos = instance_position + (vertex_offset * DOT_RADIUS);

    // Gas (state > 1.5) の場合に揺れを追加
    if (instance_state > 1.5) {
        // instance_position と time からシードを生成
        let seed_pos = vec2<u32>(u32(instance_position.x), u32(instance_position.y));
        let seed_time = u32(uniforms.time * 10.0); // 揺れの速さを調整

        // 2つの異なるシードで乱数を生成してx, yのオフセットにする
        let rand_x = to_f32_range(xorshift32(seed_pos + vec2<u32>(seed_time, 0u)));
        let rand_y = to_f32_range(xorshift32(seed_pos + vec2<u32>(0u, seed_time)));

        // 揺れの大きさを調整
        let shake_amount = 0.5; // ピクセル単位
        final_pos += vec2<f32>(rand_x, rand_y) * shake_amount;
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
    
    return output;
}

// 簡易ノイズ関数
fn noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let dist_sq = dot(in.local_pos, in.local_pos);

    // 状態に応じて形状を決定
    if (in.state < 0.5) { // Solid (state == 0.0)
        // 四角形
        if (abs(in.local_pos.x) > 1.0 || abs(in.local_pos.y) > 1.0) {
            discard;
        }
    } else { // Liquid (state == 1.0) or Gas (state == 2.0)
        // 円形
        if dist_sq > 1.0 {
            discard;
        }
    }

    let RED = vec3<f32>(1.0, 0.1, 0.1);
    let ORANGE = vec3<f32>(1.0, 0.5, 0.1);
    let WHITE = vec3<f32>(1.0, 1.0, 1.0);
    let LIGHT_BLUE = vec3<f32>(0.8, 0.9, 1.0);

    var output: FragmentOutput;
    var scene_color = vec4<f32>(in.color, 1.0);
    var glow_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // 発光処理
    if (in.luminescence > 0.8) {
        var glow_intensity = (in.luminescence - 0.8) / 0.2;

        // --- 状態ごとのエフェクト ---
        // Gas (state == 2.0)
        if (in.state > 1.5) {
            let flicker = noise(in.clip_position.xy + uniforms.time) * 0.5 + 0.7; // 0.7-1.2の範囲で揺らぐ
            glow_intensity = glow_intensity * flicker;
        }
        // Liquid (state == 1.0)
        else if (in.state > 0.5) {
            glow_intensity = glow_intensity * 1.5; // 周囲をより明るく染める
        }
        // Solid (state == 0.0) - 何もしない

        // 温度に応じて色を決定
        let temp_norm = (in.temperature + 1.0) / 2.0;
        
        var temp_color: vec3<f32>;
        if (temp_norm < 0.33) {
            temp_color = mix(RED, ORANGE, temp_norm / 0.33);
        } else if (temp_norm < 0.66) {
            temp_color = mix(ORANGE, WHITE, (temp_norm - 0.33) / 0.33);
        } else {
            temp_color = mix(WHITE, LIGHT_BLUE, (temp_norm - 0.66) / 0.34);
        }

        glow_color = vec4<f32>(temp_color * glow_intensity, 1.0);
        
        // シーンの色にも少しだけ発光を加える
        scene_color = vec4<f32>(scene_color.rgb + glow_color.rgb * 0.2, 1.0);
    }

    // 選択時の縁取り処理 (scene_colorにのみ適用)
    if (in.is_selected > 0.5) {
        let border_thickness = 0.4;
        let inner_edge = 1.0 - border_thickness;

        if dist_sq > inner_edge * inner_edge {
            let inverse_color = vec3<f32>(1.0, 1.0, 1.0) - scene_color.rgb;
            scene_color = vec4<f32>(inverse_color, 1.0);
        }
    }
    
    output.scene = scene_color;
    output.glow = glow_color;
    
    return output;
}
