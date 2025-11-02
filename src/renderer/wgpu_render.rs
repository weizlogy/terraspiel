use crate::app::Dot;
use bytemuck;
use wgpu::util::DeviceExt;

pub struct WgpuRenderer {
    render_pipeline: wgpu::RenderPipeline,
    square_vertex_buffer: wgpu::Buffer,
}

impl WgpuRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dot shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/dot.wgsl").into()),
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: (2 + 3 + 1) as wgpu::BufferAddress
                * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (2 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (5 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    shader_location: 4, // is_selected flag
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dot render pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    instance_layout,
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let square_vertex_data: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        let square_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Square Vertex Buffer"),
            contents: bytemuck::cast_slice(&square_vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            render_pipeline,
            square_vertex_buffer,
        }
    }

    fn create_dot_instance_data(dots: &[Dot]) -> Vec<f32> {
        let mut instance_data: Vec<f32> = Vec::with_capacity(dots.len() * 6); // 6 floats per dot
        for dot in dots {
            let (r, g, b) = dot.material.get_color_rgb();
            let color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];

            instance_data.push(dot.x as f32);
            instance_data.push(dot.y as f32);
            instance_data.extend_from_slice(&color);

            let is_selected = if dot.is_selected { 1.0 } else { 0.0 };
            instance_data.push(is_selected);
        }
        instance_data
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        dots: &[Dot],
    ) {
        let instance_data = Self::create_dot_instance_data(dots);

        if !instance_data.is_empty() {
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Dot Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.square_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw(0..4, 0..dots.len() as u32);
        } else {
            // Just clear the screen if there are no dots
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
    }
}