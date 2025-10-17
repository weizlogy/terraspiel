use crate::ui;
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;
use pixels::wgpu;
use pixels::{Pixels, PixelsContext};
use winit::window::Window;

pub fn render(
    egui_state: &mut EguiState,
    egui_rpass: &mut EguiRenderer,
    pixels: &Pixels,
    window: &Window,
    encoder: &mut wgpu::CommandEncoder,
    render_target: &wgpu::TextureView,
    context: &PixelsContext,
) -> Result<bool, Box<dyn std::error::Error>> {
    let raw_input = egui_state.take_egui_input(window);
    let mut should_quit = false;
    let full_output = egui_state.egui_ctx().run(raw_input, |ctx| {
        should_quit = ui::controls::draw_ui(ctx);
    });

    let clipped_primitives = egui_state
        .egui_ctx()
        .tessellate(full_output.shapes, full_output.pixels_per_point);

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [pixels.texture().size().width, pixels.texture().size().height],
        pixels_per_point: window.scale_factor() as f32,
    };

    for (id, image_delta) in &full_output.textures_delta.set {
        egui_rpass.update_texture(&context.device, &context.queue, *id, image_delta);
    }
    for id in &full_output.textures_delta.free {
        egui_rpass.free_texture(id);
    }

    egui_rpass.update_buffers(
        &context.device,
        &context.queue,
        encoder,
        &clipped_primitives,
        &screen_descriptor,
    );

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: render_target,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    egui_rpass.render(&mut rpass, &clipped_primitives, &screen_descriptor);

    Ok(should_quit)
}
