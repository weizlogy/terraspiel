use crate::material::BaseMaterialParams;
use egui_wgpu::{wgpu, Renderer, ScreenDescriptor};
use egui_winit::winit;

pub struct UiData {
    pub fps: f64,
    pub dot_count: usize,
    pub hovered_material: Option<BaseMaterialParams>,
}

pub struct Gui {
    pub ctx: egui::Context,
    pub state: egui_winit::State,
    pub renderer: Renderer,
}

impl Gui {
    pub fn new(
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let ctx = egui::Context::default();
        let state =
            egui_winit::State::new(ctx.clone(), egui::ViewportId::ROOT, event_loop, None, None);
        let renderer = Renderer::new(device, surface_format, None, 1);

        Self {
            ctx,
            state,
            renderer,
        }
    }

    pub fn handle_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    // render関数は、ランダム化ボタンが押された場合にtrueを返す
    pub fn render(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        ui_data: &UiData,
    ) -> bool {
        let mut randomize_button_clicked = false;

        let raw_input = self.state.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            // FPSとドット数を表示するウィンドウ
            egui::Window::new("Info")
                .title_bar(false)
                .movable(false)
                .resizable(false)
                .default_pos(egui::pos2(10.0, 10.0))
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {:.2}", ui_data.fps));
                    ui.label(format!("Dots: {}", ui_data.dot_count));
                    if ui
                        .button("RND")
                        .on_hover_text("Randomize brush material")
                        .clicked()
                    {
                        randomize_button_clicked = true;
                    }
                });

            // ホバーした物質の情報を表示するウィンドウ
            if let Some(material) = &ui_data.hovered_material {
                egui::Window::new("Hovered Material")
                    .default_pos(egui::pos2(10.0, 80.0))
                    .show(ctx, |ui| {
                        ui.heading("Material Properties");
                        ui.label(format!("State: {:?}", material.state));
                        ui.label(format!("Density: {:.2}", material.density));
                        ui.label(format!("Viscosity: {:.2}", material.viscosity));
                        ui.label(format!("Hardness: {:.2}", material.hardness));
                        ui.label(format!("Elasticity: {:.2}", material.elasticity));
                    });
            }
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut render_pass, &tris, &screen_descriptor);

        randomize_button_clicked
    }
}
