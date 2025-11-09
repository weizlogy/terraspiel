struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) is_selected: f32,
    @location(2) local_pos: vec2<f32>,
    @location(3) luminescence: f32,
}

struct FragmentOutput {
    @location(0) scene: vec4<f32>,
    @location(1) glow: vec4<f32>,
}

// 頂点シェーダー
const DOT_RADIUS: f32 = 2.0;

@vertex
fn vs_main(
    @location(0) vertex_offset: vec2<f32>,
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_color: vec3<f32>,
    @location(3) instance_luminescence: f32,
    @location(4) instance_is_selected: f32,
) -> VertexOutput {
    let final_pos = instance_position + (vertex_offset * DOT_RADIUS);
    
    var ndc_pos = vec2<f32>(
        (final_pos.x / 640.0) * 2.0 - 1.0,
        1.0 - (final_pos.y / 480.0) * 2.0
    );
    
    var output: VertexOutput;
    output.clip_position = vec4<f32>(ndc_pos, 0.0, 1.0);
    output.color = instance_color;
    output.luminescence = instance_luminescence;
    output.is_selected = instance_is_selected;
    output.local_pos = vertex_offset;
    
    return output;
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let dist_sq = dot(in.local_pos, in.local_pos);

    if dist_sq > 1.0 {
        discard;
    }

    var output: FragmentOutput;
    var scene_color = vec4<f32>(in.color, 1.0);
    var glow_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // 発光処理
    if (in.luminescence > 0.8) {
        let glow_intensity = (in.luminescence - 0.8) / 0.2;
        // 要求: 元の色も少し混ぜる -> ドットの色を使って光らせる
        glow_color = vec4<f32>(in.color * glow_intensity, 1.0);
        // シーンの色にも少しだけ発光を加える（HDRテクスチャなので1.0を超えても良い）
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
