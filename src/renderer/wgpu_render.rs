use crate::app::{Dot, HEIGHT, WIDTH};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CompositeUniforms {
    falloff_exponent: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct DotUniforms {
    time: f32,
}

#[allow(dead_code)]
pub struct WgpuRenderer {
    dot_render_pipeline: wgpu::RenderPipeline,
    dot_pipeline_layout: wgpu::PipelineLayout,
    dot_bind_group: wgpu::BindGroup,
    dot_uniform_buffer: wgpu::Buffer,
    square_vertex_buffer: wgpu::Buffer,

    scene_texture: wgpu::Texture,
    scene_texture_view: wgpu::TextureView,
    glow_texture: wgpu::Texture,
    glow_texture_view: wgpu::TextureView,
    blur_ping_pong_texture: wgpu::Texture,
    blur_ping_pong_texture_view: wgpu::TextureView,

    texture_sampler: wgpu::Sampler,

    composite_pipeline: wgpu::RenderPipeline,
    composite_bind_group: wgpu::BindGroup,
    composite_uniform_buffer: wgpu::Buffer,

    blur_pipeline_layout: wgpu::PipelineLayout,
    blur_horizontal_pipeline: wgpu::RenderPipeline,
    blur_vertical_pipeline: wgpu::RenderPipeline,
    blur_bind_group_horizontal: wgpu::BindGroup,
    blur_bind_group_vertical: wgpu::BindGroup,
}

impl WgpuRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // --- テクスチャの作成 ---
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Scene Texture"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let scene_texture = device.create_texture(&texture_desc);
        let scene_texture_view = scene_texture.create_view(&Default::default());

        let glow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glow Texture"),
            ..texture_desc
        });
        let glow_texture_view = glow_texture.create_view(&Default::default());

        let blur_ping_pong_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Blur Ping-Pong Texture"),
            ..texture_desc
        });
        let blur_ping_pong_texture_view = blur_ping_pong_texture.create_view(&Default::default());

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            ..Default::default()
        });

        // --- ドット描画パイプライン ---
        let dot_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dot shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/dot.wgsl").into()),
        });

        let dot_uniforms = DotUniforms { time: 0.0 };
        let dot_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dot Uniform Buffer"),
            contents: bytemuck::bytes_of(&dot_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let dot_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Dot Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let dot_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dot Bind Group"),
            layout: &dot_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: dot_uniform_buffer.as_entire_binding(),
            }],
        });

        let dot_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Dot Pipeline Layout"),
            bind_group_layouts: &[&dot_bind_group_layout],
            push_constant_ranges: &[],
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: (2 + 3 + 1 + 1 + 2) as wgpu::BufferAddress
                * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2, },
                wgpu::VertexAttribute { offset: (2 * std::mem::size_of::<f32>()) as wgpu::BufferAddress, shader_location: 2, format: wgpu::VertexFormat::Float32x3, },
                wgpu::VertexAttribute { offset: (5 * std::mem::size_of::<f32>()) as wgpu::BufferAddress, shader_location: 3, format: wgpu::VertexFormat::Float32, }, // luminescence
                wgpu::VertexAttribute { offset: (6 * std::mem::size_of::<f32>()) as wgpu::BufferAddress, shader_location: 4, format: wgpu::VertexFormat::Float32, }, // is_selected
                wgpu::VertexAttribute { offset: (7 * std::mem::size_of::<f32>()) as wgpu::BufferAddress, shader_location: 5, format: wgpu::VertexFormat::Float32, }, // temperature
                wgpu::VertexAttribute { offset: (8 * std::mem::size_of::<f32>()) as wgpu::BufferAddress, shader_location: 6, format: wgpu::VertexFormat::Float32, }, // state
            ],
        };

        let dot_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dot render pipeline"),
            layout: Some(&dot_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &dot_shader_module,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    instance_layout,
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &dot_shader_module,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState { format: texture_desc.format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL, }),
                    Some(wgpu::ColorTargetState { format: texture_desc.format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL, }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
        });

        let square_vertex_data: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        let square_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Square Vertex Buffer"),
            contents: bytemuck::cast_slice(&square_vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // --- ブラーパイプライン ---
        let blur_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/blur.wgsl").into()),
        });

        let blur_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blur Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blur_horizontal_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blur Horizontal Pipeline"),
            layout: Some(&blur_pipeline_layout),
            vertex: wgpu::VertexState { module: &blur_shader_module, entry_point: "vs_main", buffers: &[], compilation_options: Default::default(), },
            fragment: Some(wgpu::FragmentState {
                module: &blur_shader_module,
                entry_point: "fs_horizontal_blur",
                targets: &[Some(wgpu::ColorTargetState { format: texture_desc.format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL, })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
        });

        let blur_vertical_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blur Vertical Pipeline"),
            layout: Some(&blur_pipeline_layout),
            vertex: wgpu::VertexState { module: &blur_shader_module, entry_point: "vs_main", buffers: &[], compilation_options: Default::default(), },
            fragment: Some(wgpu::FragmentState {
                module: &blur_shader_module,
                entry_point: "fs_vertical_blur",
                targets: &[Some(wgpu::ColorTargetState { format: texture_desc.format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL, })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
        });

        let blur_bind_group_horizontal = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur Horizontal Bind Group"),
            layout: &blur_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&glow_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&texture_sampler) },
            ],
        });

        let blur_bind_group_vertical = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur Vertical Bind Group"),
            layout: &blur_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&blur_ping_pong_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&texture_sampler) },
            ],
        });

        // --- 合成パイプライン ---
        let composite_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Composite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/composite.wgsl").into()),
        });

        let composite_uniforms = CompositeUniforms {
            falloff_exponent: 1.5, // デフォルト値
        };
        let composite_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Composite Uniform Buffer"),
            contents: bytemuck::bytes_of(&composite_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let composite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Composite Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false, }, count: None, },
                    wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false, }, count: None, },
                    wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None, },
                    wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None, }, count: None, },
                ],
            });

        let composite_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Composite Pipeline Layout"),
                bind_group_layouts: &[&composite_bind_group_layout],
                push_constant_ranges: &[],
            });

        let composite_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Composite render pipeline"),
                layout: Some(&composite_pipeline_layout),
                vertex: wgpu::VertexState { module: &composite_shader_module, entry_point: "vs_main", buffers: &[], compilation_options: Default::default(), },
                fragment: Some(wgpu::FragmentState {
                    module: &composite_shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL, })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
                depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
            });

        let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Composite Bind Group"),
            layout: &composite_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&scene_texture_view), },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&glow_texture_view), },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&texture_sampler), },
                wgpu::BindGroupEntry { binding: 3, resource: composite_uniform_buffer.as_entire_binding(), },
            ],
        });

        Self {
            dot_render_pipeline, dot_pipeline_layout, dot_bind_group, dot_uniform_buffer, square_vertex_buffer,
            scene_texture, scene_texture_view,
            glow_texture, glow_texture_view,
            blur_ping_pong_texture, blur_ping_pong_texture_view,
            texture_sampler,
            composite_pipeline, composite_bind_group, composite_uniform_buffer,
            blur_pipeline_layout, blur_horizontal_pipeline, blur_vertical_pipeline,
            blur_bind_group_horizontal, blur_bind_group_vertical,
        }
    }

    fn create_dot_instance_data(dots: &[Dot]) -> Vec<f32> {
        let mut instance_data: Vec<f32> = Vec::with_capacity(dots.len() * 9);
        for dot in dots {
            let (r, g, b) = dot.material.get_color_rgb();
            let color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
            let state_f32 = match dot.material.state {
                crate::material::State::Solid => 0.0,
                crate::material::State::Liquid => 1.0,
                crate::material::State::Gas => 2.0,
            };
            instance_data.push(dot.x as f32);
            instance_data.push(dot.y as f32);
            instance_data.extend_from_slice(&color);
            instance_data.push(dot.material.luminescence);
            let is_selected = if dot.is_selected { 1.0 } else { 0.0 };
            instance_data.push(is_selected);
            instance_data.push(dot.material.temperature);
            instance_data.push(state_f32);
        }
        instance_data
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, dots: &[Dot], time: f32) {
        // --- ユニフォームの更新 ---
        queue.write_buffer(&self.dot_uniform_buffer, 0, bytemuck::bytes_of(&DotUniforms { time }));

        // --- ドット描画パス ---
        if !dots.is_empty() {
            let instance_data = Self::create_dot_instance_data(dots);
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Dot Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment { view: &self.scene_texture_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store, }, }),
                    Some(wgpu::RenderPassColorAttachment { view: &self.glow_texture_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store, }, }),
                ],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.dot_render_pipeline);
            render_pass.set_bind_group(0, &self.dot_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.square_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw(0..4, 0..dots.len() as u32);
        } else {
            // ドットがない場合もテクスチャをクリアする
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.scene_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store, },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.glow_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store, },
                    }),
                ],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
        }

        // --- ブラーパス ---
        // 横ブラー
        let mut blur_pass_h = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Horizontal Blur Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.blur_ping_pong_texture_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store, },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        blur_pass_h.set_pipeline(&self.blur_horizontal_pipeline);
        blur_pass_h.set_bind_group(0, &self.blur_bind_group_horizontal, &[]);
        blur_pass_h.draw(0..3, 0..1);
        drop(blur_pass_h);

        // 縦ブラー
        let mut blur_pass_v = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Vertical Blur Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.glow_texture_view, // 結果をglow_textureに書き戻す
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store, },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        blur_pass_v.set_pipeline(&self.blur_vertical_pipeline);
        blur_pass_v.set_bind_group(0, &self.blur_bind_group_vertical, &[]);
        blur_pass_v.draw(0..3, 0..1);
        drop(blur_pass_v);

        // --- 合成パス ---
        let mut composite_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Composite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store, },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });

        composite_pass.set_pipeline(&self.composite_pipeline);
        composite_pass.set_bind_group(0, &self.composite_bind_group, &[]);
        composite_pass.draw(0..3, 0..1);
    }
}