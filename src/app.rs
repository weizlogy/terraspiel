use pixels::Pixels;
use winit::window::Window;
use winit::event_loop::ActiveEventLoop;  // 追加

// ドットの状態を保持する構造体
struct Dot {
    x: f64,
    y: f64,
    vx: f64,  // x方向速度
    vy: f64,  // y方向速度
}

impl Dot {
    fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,  // 初期速度は0
            vy: 0.0,
        }
    }
}

// App構造体
pub struct App {
    pub window: Option<Window>,
    pub pixels: Option<Pixels>,
    pub mouse_position: Option<(f64, f64)>,
    dots: Vec<Dot>,        // ドットリスト
    gravity: f64,          // 重力加速度
    last_time: std::time::Instant,  // 時間管理用
    bounce_factor: f64,    // 反発係数
    is_updating: bool,     // 物理更新中かどうかのフラグ
    update_until: Option<std::time::Instant>, // 物理更新を続ける期限
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            pixels: None,
            mouse_position: None,
            dots: Vec::new(),
            gravity: 9.8 * 10.0,  // 重力加速度（画面ピクセル基準にスケーリング）
            last_time: std::time::Instant::now(),
            bounce_factor: 0.7,   // 反発係数
            is_updating: false,   // 更新中フラグの初期値
            update_until: None,   // 更新期限の初期値
        }
    }

    // ドット追加
    pub fn add_dot(&mut self, x: i32, y: i32) {
        self.dots.push(Dot::new(x as f64, y as f64));
        // ドット追加時に更新を開始
        self.is_updating = true;
        // 例えば、5秒間更新を続ける
        self.update_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
    }

    // 物理更新
    pub fn update_physics(&mut self) {
        // 更新が必要な場合のみ物理計算を行う
        if self.is_updating {
            let now = std::time::Instant::now();
            let dt = now.duration_since(self.last_time).as_secs_f64();
            self.last_time = now;

            for dot in &mut self.dots {
                // 重力を適用
                dot.vy += self.gravity * dt;

                // 位置を更新
                dot.x += dot.vx * dt;
                dot.y += dot.vy * dt;

                // 境界衝突処理（画面下端）
                if dot.y >= (super::HEIGHT as f64 - 2.0) {  // ドットの半径分余裕を持たせる
                    dot.y = super::HEIGHT as f64 - 2.0;
                    dot.vy = -dot.vy * self.bounce_factor;  // 反発
                }

                // 境界衝突処理（画面上端）
                if dot.y <= 1.0 {
                    dot.y = 1.0;
                    dot.vy = -dot.vy * self.bounce_factor;
                }

                // 境界衝突処理（左右端）
                if dot.x >= super::WIDTH as f64 - 2.0 {
                    dot.x = super::WIDTH as f64 - 2.0;
                    dot.vx = -dot.vx * self.bounce_factor;
                }
                if dot.x <= 1.0 {
                    dot.x = 1.0;
                    dot.vx = -dot.vx * self.bounce_factor;
                }
            }

            // 更新を停止する条件を確認
            let all_stopped = self.dots.iter().all(|dot| {
                // 速度が非常に小さいか、画面下端にあり跳ね返りが非常に小さいかを確認
                let velocity_small = dot.vy.abs() < 0.1 && dot.vx.abs() < 0.1;
                let at_bottom = dot.y >= super::HEIGHT as f64 - 3.0; // 少し余裕を持たせる
                let slow_bounce = velocity_small && at_bottom;
                
                slow_bounce
            });

            // すべてのドットが停止状態に達した場合、更新を停止
            if all_stopped && !self.dots.is_empty() {
                self.is_updating = false;
            }
        }
    }

    // ドット描画
    pub fn draw_dots(&mut self) {
        println!("draw_dots called, number of dots: {}", self.dots.len()); // デバッグ出力
        if let Some(ref mut pixels) = self.pixels {
            // フレームをクリア（黒）
            let frame = pixels.frame_mut();
            for pixel in frame.chunks_exact_mut(4) {
                pixel[0] = 0;  // R
                pixel[1] = 0;  // G
                pixel[2] = 0;  // B
                pixel[3] = 255; // A
            }

            // すべてのドットを描画
            for dot in &self.dots {
                println!("Drawing dot at ({}, {})", dot.x, dot.y); // デバッグ出力
                // 4x4ドットの範囲を計算
                let x = dot.x as i32;
                let y = dot.y as i32;
                let start_x = (x - 2).max(0).min(super::WIDTH as i32 - 1);
                let end_x = (x + 1).max(0).min(super::WIDTH as i32 - 1);
                let start_y = (y - 2).max(0).min(super::HEIGHT as i32 - 1);
                let end_y = (y + 1).max(0).min(super::HEIGHT as i32 - 1);

                println!("Drawing range: ({}, {}) to ({}, {})", start_x, start_y, end_x, end_y); // デバッグ出力

                for py in start_y..=end_y {
                    for px in start_x..=end_x {
                        let pixel_index = (py as usize * super::WIDTH as usize + px as usize) * 4;
                        // RGBA: 白色 (255, 255, 255, 255)
                        frame[pixel_index] = 255;       // R
                        frame[pixel_index + 1] = 255;   // G
                        frame[pixel_index + 2] = 255;   // B
                        frame[pixel_index + 3] = 255;   // A
                    }
                }
            }
        }
    }

    // 単一ドット描画（4x4ピクセル） - 不要になったので削除
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 初回実行時やアプリ再開時にウィンドウとピクセルを初期化
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Dot Drawer with Gravity")
                .with_inner_size(winit::dpi::LogicalSize::new(super::WIDTH, super::HEIGHT));

            let window = event_loop.create_window(window_attributes)
                .expect("Failed to create window");

            let window_size = window.inner_size();
            let surface_texture = pixels::SurfaceTexture::new(window_size.width, window_size.height, &window);
            let pixels = pixels::Pixels::new(super::WIDTH, super::HEIGHT, surface_texture)
                .expect("Failed to create pixels");

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
        
        // アプリが再開されたときに時間差分が大きくなるのを防ぐため、last_timeを現在時刻にリセット
        self.last_time = std::time::Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(ref window) = self.window {
            if window.id() != window_id {
                return;
            }
        } else {
            return;
        }

        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                // マウス位置を保存
                self.mouse_position = Some((position.x, position.y));
            }
            winit::event::WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                // 左クリックでドットを追加
                if let Some((x, y)) = self.mouse_position {
                    self.add_dot(x as i32, y as i32);
                    // 再描画をリクエスト
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                // 物理更新
                self.update_physics();
                
                // ドット描画
                self.draw_dots();
                
                // 画面に反映
                if let Some(ref mut pixels) = self.pixels {
                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }
                
                // 更新中であれば、すぐに次の再描画をリクエスト
                if self.is_updating {
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}