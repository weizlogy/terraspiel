use crate::material::{BaseMaterialParams, MaterialDNA};
use egui_wgpu::{wgpu, Renderer, ScreenDescriptor};
use egui_winit::winit;

pub struct UiData {
    pub fps: f64,
    pub dot_count: usize,
    pub selected_material: Option<BaseMaterialParams>,
    pub selected_dot_dna: Option<MaterialDNA>,
    pub selected_dot_name: Option<String>,
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
    ) -> (bool, bool) { // 戻り値を (randomize_clicked, clear_clicked) に変更
        let mut randomize_button_clicked = false;
        let mut clear_button_clicked = false; // 新しいフラグ

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
                    // CLSボタンを追加
                    if ui
                        .button("CLS")
                        .on_hover_text("Clear all dots")
                        .clicked()
                    {
                        clear_button_clicked = true;
                    }
                });

            // ホバーした物質の情報を表示するウィンドウ
            if let Some(material) = &ui_data.selected_material {
                let window_title = ui_data
                    .selected_dot_name
                    .clone()
                    .unwrap_or_else(|| "Selected Material".to_string());

                egui::Window::new(window_title)
                    .default_pos(egui::pos2(10.0, 80.0))
                    .resizable(true)
                    .default_height(300.0)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(dna) = &ui_data.selected_dot_dna {
                                ui.label(format!("Seed: {}", dna.seed));
                            }
                            
                            ui.separator();

                            egui::Grid::new("material_properties_grid")
                                .num_columns(2)
                                .spacing([20.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    // --- Basic ---
                                    ui.heading("Basic");
                                    ui.end_row();
                                    ui.label("State");
                                    ui.label(format!("{:?}", material.state));
                                    ui.end_row();

                                    // --- Physical ---
                                    ui.heading("Physical");
                                    ui.end_row();
                                    ui.label("Density");
                                    ui.label(format!("{:.2}", material.density));
                                    ui.end_row();
                                    ui.label("Viscosity");
                                    ui.label(format!("{:.2}", material.viscosity));
                                    ui.end_row();
                                    ui.label("Hardness");
                                    ui.label(format!("{:.2}", material.hardness));
                                    ui.end_row();
                                    ui.label("Elasticity");
                                    ui.label(format!("{:.2}", material.elasticity));
                                    ui.end_row();
                                    ui.label("Melting Point");
                                    ui.label(format!("{:.2}", material.melting_point));
                                    ui.end_row();
                                    ui.label("Boiling Point");
                                    ui.label(format!("{:.2}", material.boiling_point));
                                    ui.end_row();
                                    ui.label("Flammability");
                                    ui.label(format!("{:.2}", material.flammability));
                                    ui.end_row();

                                    // --- Thermal ---
                                    ui.heading("Thermal");
                                    ui.end_row();
                                    ui.label("Temperature");
                                    ui.label(format!("{:.2}", material.temperature));
                                    ui.end_row();
                                    
                                    ui.label("Heat Capacity");
                                    ui.label(format!("{:.2}", material.heat_capacity));
                                    ui.end_row();

                                    // --- Electromagnetic ---
                                    ui.heading("Electromagnetic");
                                    ui.end_row();
                                    ui.label("Conductivity");
                                    ui.label(format!("{:.2}", material.conductivity));
                                    ui.end_row();
                                    ui.label("Magnetism");
                                    ui.label(format!("{:.2}", material.magnetism));
                                    ui.end_row();

                                    // --- Optical ---
                                    ui.heading("Optical");
                                    ui.end_row();
                                    ui.label("Color Hue");
                                    ui.label(format!("{:.2}", material.color_hue));
                                    ui.end_row();
                                    ui.label("Color Saturation");
                                    ui.label(format!("{:.2}", material.color_saturation));
                                    ui.end_row();
                                    ui.label("Color Luminance");
                                    ui.label(format!("{:.2}", material.color_luminance));
                                    ui.end_row();
                                    ui.label("Luminescence");
                                    ui.label(format!("{:.2}", material.luminescence));
                                    ui.end_row();
                                });
                        });
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

        (randomize_button_clicked, clear_button_clicked) // 戻り値を変更
    }
}
