use super::gui::{Gui, UiData};
use super::wgpu_render::WgpuRenderer;
use crate::app::{Dot, HEIGHT, WIDTH};
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    surface: Arc<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    wgpu_renderer: WgpuRenderer,
    pub gui: Gui,
}

impl Renderer {
    pub fn new(
        window: &Arc<Window>,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let binding = window.clone();
        let surface = instance
            .create_surface(binding.as_ref())
            .expect("Failed to create surface");
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
        let surface = Arc::new(surface);

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find an appropriate adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: WIDTH,
            height: HEIGHT,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let wgpu_renderer = WgpuRenderer::new(&device, config.format);
        let gui = Gui::new(event_loop, &device, config.format);

        Self {
            surface,
            device,
            queue,
            config,
            wgpu_renderer,
            gui,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, window: &Window, dots: &[Dot], ui_data: &UiData) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.wgpu_renderer
            .render(&self.device, &mut encoder, &view, dots);

        self.gui.render(
            window,
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            ui_data,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
