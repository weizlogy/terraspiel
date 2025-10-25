struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// 頂点シェーダー
@vertex
fn vs_main(
    @location(0) vertex_offset: vec2<f32>,  // 基本形状の頂点オフセット (-1.0 〜 1.0)
    @location(1) instance_position: vec2<f32>,  // インスタンスの中心位置
    @location(2) instance_color: vec3<f32>     // インスタンスの色
) -> VertexOutput {
    // 頂点の最終位置 = インスタンスの中心位置 + 基本形状の頂点オフセット
    let final_pos = instance_position + vertex_offset;
    
    // 画面サイズ (WIDTH x HEIGHT) から NDC (Normalized Device Coordinates) に変換
    var ndc_pos = vec2<f32>(
        (final_pos.x / 640.0) * 2.0 - 1.0,  // X座標を-1.0〜1.0に変換
        1.0 - (final_pos.y / 480.0) * 2.0   // Y座標を1.0〜-1.0に変換 (Y軸が上下反転しているため)
    );
    
    var output: VertexOutput;
    output.clip_position = vec4<f32>(ndc_pos, 0.0, 1.0);
    output.color = instance_color;
    
    return output;
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}