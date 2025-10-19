use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(&mut App::new())?;
    Ok(())
}

struct App {
    window: Option<Window>,
    pixels: Option<Pixels>,
    mouse_position: Option<(f64, f64)>, // マウス位置を保存
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            pixels: None,
            mouse_position: None,
        }
    }

    fn draw_dot(&mut self, x: i32, y: i32) {
        println!("Drawing dot at: ({}, {})", x, y); // デバッグ出力
        if let Some(ref mut pixels) = self.pixels {
            let frame = pixels.frame_mut();

            // 4x4ドットの範囲を計算（中心をクリック位置）
            let start_x = (x - 2).max(0).min(WIDTH as i32 - 1);
            let end_x = (x + 1).max(0).min(WIDTH as i32 - 1);
            let start_y = (y - 2).max(0).min(HEIGHT as i32 - 1);
            let end_y = (y + 1).max(0).min(HEIGHT as i32 - 1);

            println!(
                "Drawing range: ({}, {}) to ({}, {})",
                start_x, start_y, end_x, end_y
            ); // デバッグ出力

            for py in start_y..=end_y {
                for px in start_x..=end_x {
                    let pixel_index = (py as usize * WIDTH as usize + px as usize) * 4;
                    // RGBA: 白色 (255, 255, 255, 255)
                    frame[pixel_index] = 255; // R
                    frame[pixel_index + 1] = 255; // G
                    frame[pixel_index + 2] = 255; // B
                    frame[pixel_index + 3] = 255; // A
                }
            }

            // 描画後に即座に画面に反映
            if pixels.render().is_err() {
                println!("Error rendering pixels");
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 初回実行時やアプリ再開時にウィンドウとピクセルを初期化
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Dot Drawer")
                .with_inner_size(LogicalSize::new(WIDTH, HEIGHT));

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            let pixels =
                Pixels::new(WIDTH, HEIGHT, surface_texture).expect("Failed to create pixels");

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref window) = self.window {
            if window.id() != window_id {
                return;
            }
        } else {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                // マウス位置を保存
                self.mouse_position = Some((position.x, position.y));
                println!("Mouse moved to: ({}, {})", position.x, position.y); // デバッグ出力
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                // 左クリックで保存されたマウス位置を使ってドットを描画
                if let Some((x, y)) = self.mouse_position {
                    println!("Left click detected at: ({}, {})", x, y); // デバッグ出力
                    self.draw_dot(x as i32, y as i32);
                } else {
                    println!("Left click detected but no mouse position saved");
                    // デバッグ出力
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut pixels) = self.pixels {
                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
}
