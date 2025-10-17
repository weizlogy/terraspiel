use egui::ViewportId;
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;
use pixels::{PixelsBuilder, SurfaceTexture};
use std::error::Error;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Terraspiel - Minimal Example")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop)?,
    );

    let mut input = WinitInputHelper::new();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(true)
            .build()?
    };

    let mut egui_state = EguiState::new(
        egui::Context::default(),
        ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut egui_rpass =
        EguiRenderer::new(pixels.device(), pixels.render_texture_format(), None, 1);

    event_loop.run(move |event, elwt| {
        // egui にイベントを渡す
        if let Event::WindowEvent { event, .. } = &event {
            let _response = egui_state.on_window_event(&window, event);
        }

        // メインの入力とウィンドウイベント処理
        if input.update(&event) {
            // Escキーで終了
            if input.key_pressed(KeyCode::Escape) {
                elwt.exit();
                return;
            }

            // ウィンドウリサイズ
            if let Some(size) = input.window_resized() {
                if pixels.resize_surface(size.width, size.height).is_err() {
                    elwt.exit();
                    return;
                }
            }

            // 描画リクエスト
            window.request_redraw();
        }

        match event {
            // 閉じるボタン
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // egui UI の構築
                let raw_input = egui_state.take_egui_input(&window);
                let full_output = egui_state.egui_ctx().run(raw_input, |ctx| {
                    egui::Window::new("Hello").show(ctx, |ui| {
                        ui.label("This is a minimal sample for Terraspiel.");
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            elwt.exit();
                        }
                    });
                });

                let clipped_primitives = egui_state
                    .egui_ctx()
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                // egui の描画コマンドを pixels に渡す
                let result = pixels.render_with(|encoder, render_target, context| {
                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [
                            pixels.texture().size().width,
                            pixels.texture().size().height,
                        ],
                        pixels_per_point: window.scale_factor() as f32,
                    };

                    // テクスチャ更新
                    for (id, image_delta) in &full_output.textures_delta.set {
                        egui_rpass.update_texture(
                            &context.device,
                            &context.queue,
                            *id,
                            image_delta,
                        );
                    }
                    for id in &full_output.textures_delta.free {
                        egui_rpass.free_texture(id);
                    }

                    // バッファ更新と描画
                    egui_rpass.update_buffers(
                        &context.device,
                        &context.queue,
                        encoder,
                        &clipped_primitives,
                        &screen_descriptor,
                    );
                    let mut rpass =
                        encoder.begin_render_pass(&pixels::wgpu::RenderPassDescriptor {
                            label: Some("egui render pass"),
                            color_attachments: &[Some(pixels::wgpu::RenderPassColorAttachment {
                                view: render_target,
                                resolve_target: None,
                                ops: pixels::wgpu::Operations {
                                    load: pixels::wgpu::LoadOp::Clear(pixels::wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0,
                                    }),
                                    store: pixels::wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                    egui_rpass.render(&mut rpass, &clipped_primitives, &screen_descriptor);

                    Ok(())
                });

                if result.is_err() {
                    elwt.exit();
                }
            }
            _ => (),
        }
    })?;

    Ok(())
}
