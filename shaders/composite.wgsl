// shaders/composite.wgsl

@group(0) @binding(0) var scene_texture: texture_2d<f32>;
@group(0) @binding(1) var glow_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct Uniforms {
    falloff_exponent: f32,
}
@group(0) @binding(3) var<uniform> uniforms: Uniforms;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    // 画面全体を覆う三角形を生成
    let x = f32(in_vertex_index / 2u) * 4.0 - 1.0;
    let y = f32(in_vertex_index % 2u) * 4.0 - 1.0;
    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    output.tex_coords = vec2<f32>(x * 0.5 + 0.5, -y * 0.5 + 0.5);
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(scene_texture, texture_sampler, in.tex_coords).rgb;
    var glow_color = textureSample(glow_texture, texture_sampler, in.tex_coords).rgb;

    // Falloffカーブを適用
    glow_color = pow(glow_color, vec3<f32>(uniforms.falloff_exponent));

    // シーンとグローを加算合成
    let hdr_color = scene_color + glow_color;

    // HDRからSDRへの簡易トーンマッピング (Reinhard)
    let mapped_color = hdr_color / (hdr_color + vec3<f32>(1.0));
    
    return vec4<f32>(mapped_color, 1.0);
}
