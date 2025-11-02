struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) is_selected: f32,
    @location(2) local_pos: vec2<f32>,
}

// 頂点シェーダー
const DOT_RADIUS: f32 = 2.0;

@vertex
fn vs_main(
    @location(0) vertex_offset: vec2<f32>,  // 基本形状の頂点オフセット (-1.0 〜 1.0)
    @location(1) instance_position: vec2<f32>,  // インスタンスの中心位置
    @location(2) instance_color: vec3<f32>,     // インスタンスの色
    @location(4) instance_is_selected: f32, // is_selected を受け取る
) -> VertexOutput {
    // 頂点の最終位置 = インスタンスの中心位置 + (基本形状の頂点オフセット * 半径)
    let final_pos = instance_position + (vertex_offset * DOT_RADIUS);
    
    // 画面サイズ (WIDTH x HEIGHT) から NDC (Normalized Device Coordinates) に変換
    var ndc_pos = vec2<f32>(
        (final_pos.x / 640.0) * 2.0 - 1.0,  // X座標を-1.0〜1.0に変換
        1.0 - (final_pos.y / 480.0) * 2.0   // Y座標を1.0〜-1.0に変換 (Y軸が上下反転しているため)
    );
    
    var output: VertexOutput;
    output.clip_position = vec4<f32>(ndc_pos, 0.0, 1.0);
    output.color = instance_color;
    output.is_selected = instance_is_selected;
    output.local_pos = vertex_offset; // ローカル座標を渡す
    
    return output;
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_sq = dot(in.local_pos, in.local_pos);

    if dist_sq > 1.0 {
        discard;
    }

    if (in.is_selected > 0.5) {
        let border_thickness = 0.4; // 縁の太さを増やす
        let inner_edge = 1.0 - border_thickness;

        if dist_sq > inner_edge * inner_edge {
            let inverse_color = vec3<f32>(1.0, 1.0, 1.0) - in.color;
            return vec4<f32>(inverse_color, 1.0); // 縁は反転色
        }
    }
    
    return vec4<f32>(in.color, 1.0);
}
