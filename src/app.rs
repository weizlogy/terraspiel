
use egui::ViewportId;
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use std::error::Error;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub struct App<'a> {
    pub window: Arc<Window>,
    pub input: WinitInputHelper,
    pub pixels: Pixels<'a>,
    pub egui_state: EguiState,
    pub egui_rpass: EguiRenderer,
}

impl<'a> App<'a> {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Terraspiel - Minimal Example")
                .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
                .build(event_loop)?,
        );

        let input = WinitInputHelper::new();

        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
            PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
                .enable_vsync(true)
                .build()?
        };

        let egui_state = EguiState::new(
            egui::Context::default(),
            ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let egui_rpass =
            EguiRenderer::new(pixels.device(), pixels.render_texture_format(), None, 1);

        Ok(Self {
            window,
            input,
            pixels,
            egui_state,
            egui_rpass,
        })
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<(), Box<dyn Error>> {
        event_loop.run(move |event, elwt| {
            if let Event::WindowEvent { event, .. } = &event {
                let _response = self.egui_state.on_window_event(&self.window, event);
            }

            if self.input.update(&event) {
                if self.input.key_pressed(KeyCode::Escape) {
                    elwt.exit();
                    return;
                }

                if let Some(size) = self.input.window_resized() {
                    if self.pixels.resize_surface(size.width, size.height).is_err() {
                        elwt.exit();
                        return;
                    }
                }

                self.window.request_redraw();
            }

            match event {
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
                    let App {
                        window,
                        pixels,
                        egui_state,
                        egui_rpass,
                        ..
                    } = &mut self;

                    let mut should_quit = false;
                    let result = pixels.render_with(|encoder, render_target, context| {
                        let render_result = crate::render::render(
                            egui_state,
                            egui_rpass,
                            pixels,
                            window,
                            encoder,
                            render_target,
                            context,
                        );
                        if let Ok(do_quit) = render_result {
                            should_quit = do_quit;
                        }
                        Ok(())
                    });

                    if result.is_err() || should_quit {
                        elwt.exit();
                    }
                }
                _ => (),
            }
        })?;

        Ok(())
    }
}
