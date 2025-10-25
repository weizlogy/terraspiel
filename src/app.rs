use crate::renderer::Renderer;
use crate::material::BaseMaterialParams;
use std::sync::Arc;
use winit::window::{Window, WindowBuilder};

// ドットの状態を保持する構造体
pub struct Dot {
    pub x: f64,
    pub y: f64,
    pub vx: f64,  // x方向速度
    pub vy: f64,  // y方向速度
    pub material: BaseMaterialParams,
}

impl Dot {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,  // 初期速度は0
            vy: 0.0,
            material: BaseMaterialParams::default(),
        }
    }
}

// App構造体
pub struct App {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,
    pub mouse_position: Option<(f64, f64)>,
    pub dots: Vec<Dot>,        // ドットリスト
    pub gravity: f64,          // 重力加速度
    pub last_time: std::time::Instant,  // 時間管理用
    pub bounce_factor: f64,    // 反発係数
    pub is_updating: bool,     // 物理更新中かどうかのフラグ
    pub left_mouse_pressed: bool, // 左クリックが押されているか
    pub last_dot_add_time: std::time::Instant, // 最後にドットを追加した時刻
    pub dot_add_interval: std::time::Duration, // ドット追加の間隔
    pub frame_times: std::collections::VecDeque<f64>,
    pub last_fps_update: std::time::Instant,
    pub fps: f64,
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

impl App {
    pub fn handle_window_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        if let Some(renderer) = &mut self.renderer {
            renderer.gui.handle_window_event(window, event)
        } else {
            false
        }
    }

    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            mouse_position: None,
            dots: Vec::new(),
            gravity: 9.8 * 10.0,  // 重力加速度（画面ピクセル基準にスケーリング）
            last_time: std::time::Instant::now(),
            bounce_factor: 0.7,   // 反発係数
            is_updating: false,   // 更新中フラグの初期値
            left_mouse_pressed: false, // 左クリック押下状態の初期値
            last_dot_add_time: std::time::Instant::now(), // ドット追加時刻の初期値
            dot_add_interval: std::time::Duration::from_millis(100), // 100msごとにドット追加
            frame_times: std::collections::VecDeque::with_capacity(100),
            last_fps_update: std::time::Instant::now(),
            fps: 0.0,
        }
    }

    // ドット追加（簡略化版 - 常にドットを追加）
    pub fn add_dot_if_not_exists(&mut self, x: i32, y: i32) {
        // 位置の重複チェックを一旦外す
        self.dots.push(Dot::new(x as f64, y as f64));
        self.is_updating = true; // 物理更新を開始
        self.last_time = std::time::Instant::now(); // 物理更新の基準時刻をリセット
        self.last_dot_add_time = std::time::Instant::now(); // 最後に追加した時刻を更新
    }

    // ウィンドウの再開時に呼び出される
        pub fn handle_resume(&mut self, event_loop: &winit::event_loop::EventLoopWindowTarget<()>) {
            if self.window.is_none() {
                let window = Arc::new(
                    WindowBuilder::new()
                        .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
                        .build(event_loop)
                        .expect("Failed to create window")
                );
                self.window = Some(window.clone());
                self.renderer = Some(Renderer::new(&window, event_loop));
            }
            
            // アプリが再開されたときに時間差分が大きくなるのを防ぐため、last_timeを現在時刻にリセット
            self.last_time = std::time::Instant::now();
            self.last_dot_add_time = std::time::Instant::now();
        }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(new_size);
        }
    }

    // カーソル移動時に呼び出される
    pub fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = Some((position.x, position.y));
    }

    // マウス入力時に呼び出される
    pub fn handle_mouse_input(&mut self, state: winit::event::ElementState, button: winit::event::MouseButton) {
        match button {
            winit::event::MouseButton::Left => {
                match state {
                    winit::event::ElementState::Pressed => {
                        // 左クリック押下
                        println!("Left mouse pressed"); // デバッグ出力
                        self.left_mouse_pressed = true;
                        if let Some((x, y)) = self.mouse_position {
                            println!("Adding dot at ({}, {})", x as i32, y as i32); // デバッグ出力
                            self.add_dot_if_not_exists(x as i32, y as i32);
                            println!("Number of dots after add: {}", self.dots.len()); // デバッグ出力
                            // 再描画をリクエスト
                            if let Some(ref window) = self.window {
                                window.request_redraw();
                            }
                        } else {
                            println!("Mouse position is unknown - move the mouse first"); // デバッグ出力
                        }
                    }
                    winit::event::ElementState::Released => {
                        // 左クリック解放
                        println!("Left mouse released"); // デバッグ出力
                        self.left_mouse_pressed = false;
                    }
                }
            }
            _ => {}
        }
    }

    // ドットのインスタンスデータ（中心座標、色）を生成する
    pub fn create_dot_instance_data(&self) -> Vec<f32> {
        let mut instance_data: Vec<f32> = Vec::new();
        for dot in &self.dots {
            let (r, g, b) = dot.material.get_color_rgb();
            let color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
            
            // 各ドットの中心座標と色をインスタンスデータとして追加
            instance_data.push(dot.x as f32);
            instance_data.push(dot.y as f32);
            instance_data.extend_from_slice(&color);
        }

        instance_data
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
                if dot.y >= (crate::app::HEIGHT as f64 - 2.0) {
                    // ドットの半径分余裕を持たせる
                    dot.y = crate::app::HEIGHT as f64 - 2.0;
                    dot.vy = -dot.vy * self.bounce_factor; // 反発
                }

                // 境界衝突処理（画面上端）
                if dot.y <= 1.0 {
                    dot.y = 1.0;
                    dot.vy = -dot.vy * self.bounce_factor;
                }

                // 境界衝突処理（左右端）
                if dot.x >= crate::app::WIDTH as f64 - 2.0 {
                    dot.x = crate::app::WIDTH as f64 - 2.0;
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
                let at_bottom = dot.y >= crate::app::HEIGHT as f64 - 3.0; // 少し余裕を持たせる
                let slow_bounce = velocity_small && at_bottom;

                slow_bounce
            });

            // すべてのドットが停止状態に達した場合、更新を停止
            if all_stopped && !self.dots.is_empty() {
                self.is_updating = false;
                println!("Physics update stopped - all dots have stopped"); // デバッグ出力
            }
        }
    }

    // 再描画要求時に呼び出される
    pub fn handle_redraw_requested(&mut self) {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_time).as_secs_f64();
        self.frame_times.push_back(delta_time);
        if self.frame_times.len() > 100 {
            self.frame_times.pop_front();
        }

        // 左クリック押しっぱなしで、かつ指定時間経過している場合にドット追加
        if self.left_mouse_pressed {
            if let Some((x, y)) = self.mouse_position {
                if std::time::Instant::now().duration_since(self.last_dot_add_time) >= self.dot_add_interval {
                    println!("Adding dot due to hold at ({}, {})", x as i32, y as i32); // デバッグ出力
                    self.add_dot_if_not_exists(x as i32, y as i32);
                }
            }
        }
        
        // 物理更新
        self.update_physics();
        
        // FPS計算とUI描画
        if now.duration_since(self.last_fps_update).as_secs_f32() > 0.5 {
            let sum: f64 = self.frame_times.iter().sum();
            if !self.frame_times.is_empty() {
                self.fps = self.frame_times.len() as f64 / sum;
            }
            self.last_fps_update = now;
        }

        let instance_data = self.create_dot_instance_data();
        let window = self.window.as_ref().unwrap();
        let fps = self.fps;
        let num_dots = self.dots.len();

        if let Some(renderer) = &mut self.renderer {
            renderer.render(window, &instance_data, |ctx| {
                egui::Window::new("Info")
                    .movable(false)
                    .default_pos(egui::pos2(10.0, 10.0))
                    .show(ctx, |ui| {
                        ui.label(format!("FPS: {:.2}", fps));
                        ui.label(format!("Dots: {}", num_dots));
                });
            });
        }
    }
}