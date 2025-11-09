// shaders/blur.wgsl

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// 画面全体を覆う三角形を生成する共通の頂点シェーダー
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let x = f32(in_vertex_index / 2u) * 4.0 - 1.0;
    let y = f32(in_vertex_index % 2u) * 4.0 - 1.0;
    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    output.tex_coords = vec2<f32>(x * 0.5 + 0.5, -y * 0.5 + 0.5);
    return output;
}

// ガウス重み (5x1カーネル)
const GAUSS_WEIGHTS: array<f32, 3> = array<f32, 3>(0.227027, 0.316216, 0.070270);

// 横方向ブラー
@fragment
fn fs_horizontal_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let texel_size = 1.0 / texture_size;
    
    var result = textureSample(input_texture, texture_sampler, in.tex_coords) * GAUSS_WEIGHTS[0];
    
    // 横方向にサンプリング (ループ展開)
    let offset1 = 1.0 * texel_size.x;
    result += textureSample(input_texture, texture_sampler, in.tex_coords + vec2<f32>(offset1, 0.0)) * GAUSS_WEIGHTS[1];
    result += textureSample(input_texture, texture_sampler, in.tex_coords - vec2<f32>(offset1, 0.0)) * GAUSS_WEIGHTS[1];

    let offset2 = 2.0 * texel_size.x;
    result += textureSample(input_texture, texture_sampler, in.tex_coords + vec2<f32>(offset2, 0.0)) * GAUSS_WEIGHTS[2];
    result += textureSample(input_texture, texture_sampler, in.tex_coords - vec2<f32>(offset2, 0.0)) * GAUSS_WEIGHTS[2];
    
    return result;
}

// 縦方向ブラー
@fragment
fn fs_vertical_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let texel_size = 1.0 / texture_size;

    var result = textureSample(input_texture, texture_sampler, in.tex_coords) * GAUSS_WEIGHTS[0];

    // 縦方向にサンプリング (ループ展開)
    let offset1 = 1.0 * texel_size.y;
    result += textureSample(input_texture, texture_sampler, in.tex_coords + vec2<f32>(0.0, offset1)) * GAUSS_WEIGHTS[1];
    result += textureSample(input_texture, texture_sampler, in.tex_coords - vec2<f32>(0.0, offset1)) * GAUSS_WEIGHTS[1];

    let offset2 = 2.0 * texel_size.y;
    result += textureSample(input_texture, texture_sampler, in.tex_coords + vec2<f32>(0.0, offset2)) * GAUSS_WEIGHTS[2];
    result += textureSample(input_texture, texture_sampler, in.tex_coords - vec2<f32>(0.0, offset2)) * GAUSS_WEIGHTS[2];

    return result;
}
