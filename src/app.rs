use rand::Rng;
use rand::thread_rng;
use crate::material::{from_dna, BaseMaterialParams, MaterialDNA, State, to_dna};
use crate::physics::engine::DOT_RADIUS;
use crate::physics::{engine, Physics};
use crate::renderer::Renderer;
use std::sync::{mpsc, Arc};
use winit::window::{Window, WindowBuilder};

// ドットの状態を保持する構造体
#[derive(Clone)]
pub struct Dot {
    pub x: f64,
    pub y: f64,
    pub vx: f64, // x方向速度
    pub vy: f64, // y方向速度
    pub material: BaseMaterialParams,
    pub material_dna: MaterialDNA, // 物質DNA
}

/// 非同期ブレンド処理の結果
#[derive(Debug)]
pub enum BlendResult {
    Change { index: usize, new_dna: MaterialDNA },
    Vanish { index: usize },
}

impl Dot {
    // new関数は使われなくなるので削除、もしくは更新が必要
}

// App構造体
pub struct App {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,
    pub mouse_position: Option<(f64, f64)>,
    pub dots: Vec<Dot>,                // ドットリスト
    pub gravity: f64,                  // 重力加速度
    pub last_time: std::time::Instant, // 時間管理用
    pub physics: Physics,

    pub is_updating: bool,                     // 物理更新中かどうかのフラグ
    pub left_mouse_pressed: bool,              // 左クリックが押されているか
    pub last_dot_add_time: std::time::Instant, // 最後にドットを追加した時刻
    pub dot_add_interval: std::time::Duration, // ドット追加の間隔
    pub frame_times: std::collections::VecDeque<f64>,
    pub last_fps_update: std::time::Instant,
    pub fps: f64,
    pub brush_material: BaseMaterialParams, // 現在選択中の物質
    pub hovered_dot_index: Option<usize>,   // マウスがホバーしているドット

    // 非同期処理用
    pub collision_tx: mpsc::Sender<((usize, MaterialDNA), (usize, MaterialDNA))>, // 衝突イベント送信
    pub result_rx: mpsc::Receiver<BlendResult>, // ブレンド結果受信
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

impl App {
    pub fn new(
        collision_tx: mpsc::Sender<((usize, MaterialDNA), (usize, MaterialDNA))>,
        result_rx: mpsc::Receiver<BlendResult>,
    ) -> Self {
        Self {
            window: None,

            renderer: None,

            mouse_position: None,

            dots: Vec::new(),

            gravity: 9.8 * 10.0,

            last_time: std::time::Instant::now(),
            physics: Physics::new(collision_tx.clone()),

            is_updating: false,

            left_mouse_pressed: false,

            last_dot_add_time: std::time::Instant::now(),

            dot_add_interval: std::time::Duration::from_millis(100),

            frame_times: std::collections::VecDeque::with_capacity(100),

            last_fps_update: std::time::Instant::now(),

            fps: 0.0,

            brush_material: BaseMaterialParams::default(),

            hovered_dot_index: None,
            collision_tx,
            result_rx,
        }
    }

    // ブラシの物質をランダム化

    fn randomize_brush_material(&mut self) {
        let mut rng = thread_rng();

        let state_choice = rng.gen_range(0..=2);

        let state = match state_choice {
            0 => State::Solid,

            1 => State::Liquid,

            _ => State::Gas,
        };

        self.brush_material.state = state;

        self.brush_material.density = rng.gen(); // 0.0 ~ 1.0

        self.brush_material.color_hue = rng.gen(); // 0.0 ~ 1.0

        self.brush_material.viscosity = rng.gen();

        self.brush_material.hardness = rng.gen();

        self.brush_material.elasticity = rng.gen();
    }

    pub fn clear_dots(&mut self) {
        self.dots.clear();
        self.is_updating = false;
    }

    pub fn handle_window_event(
        &mut self,

        window: &Window,

        event: &winit::event::WindowEvent,
    ) -> bool {
        if let Some(renderer) = &mut self.renderer {
            renderer.gui.handle_window_event(window, event)
        } else {
            false
        }
    }

    pub fn add_dot_if_not_exists(&mut self, x: i32, y: i32) {
        let mut rng = thread_rng();
        let seed = rng.gen();
        let material_dna = to_dna(&self.brush_material, seed);

        let dot = Dot {
            x: x as f64,
            y: y as f64,
            vx: 0.0,
            vy: 0.0,
            material: self.brush_material.clone(), // ブラシの物質を適用
            material_dna,
        };

        self.dots.push(dot);

        self.is_updating = true;

        self.last_time = std::time::Instant::now();

        self.last_dot_add_time = std::time::Instant::now();
    }

    pub fn handle_resume(&mut self, event_loop: &winit::event_loop::EventLoopWindowTarget<()>) {
        if self.window.is_none() {
            let window = Arc::new(
                WindowBuilder::new()
                    .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
                    .build(event_loop)
                    .expect("Failed to create window"),
            );

            self.window = Some(window.clone());

            self.renderer = Some(Renderer::new(&window, event_loop));
        }

        self.last_time = std::time::Instant::now();

        self.last_dot_add_time = std::time::Instant::now();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(new_size);
        }
    }

    pub fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = Some((position.x, position.y));

        // ホバー判定

        self.hovered_dot_index = None;

        for (i, dot) in self.dots.iter().enumerate().rev() {
            let dx = dot.x - position.x;

            let dy = dot.y - position.y;

            if (dx * dx + dy * dy) < (DOT_RADIUS * DOT_RADIUS) {
                self.hovered_dot_index = Some(i);

                break;
            }
        }
    }

    pub fn handle_mouse_input(
        &mut self,

        state: winit::event::ElementState,

        button: winit::event::MouseButton,
    ) {
        if button == winit::event::MouseButton::Left {
            match state {
                winit::event::ElementState::Pressed => {
                    self.left_mouse_pressed = true;

                    if let Some((x, y)) = self.mouse_position {
                        self.add_dot_if_not_exists(x as i32, y as i32);

                        if let Some(ref window) = self.window {
                            window.request_redraw();
                        }
                    }
                }

                winit::event::ElementState::Released => {
                    self.left_mouse_pressed = false;
                }
            }
        }
    }

    pub fn update_physics(&mut self) {
        if !self.is_updating {
            return;
        }

        let now = std::time::Instant::now();

        let dt = now.duration_since(self.last_time).as_secs_f64();

        self.last_time = now;

        // 1. 状態に基づいて力を適用
        engine::update_state(&mut self.dots, self.gravity, dt);

        // 2. 衝突判定と応答
        self.physics.update_collision(&mut self.dots, dt);

        // 3. 位置更新と壁との衝突
        let all_stopped = engine::update_position(&mut self.dots, dt);

        if all_stopped && !self.dots.is_empty() {
            self.is_updating = false;
        }
    }

    pub fn handle_redraw_requested(&mut self) {
        let now = std::time::Instant::now();

        let delta_time = now.duration_since(self.last_time).as_secs_f64();

        self.frame_times.push_back(delta_time);

        if self.frame_times.len() > 100 {
            self.frame_times.pop_front();
        }

        if self.left_mouse_pressed {
            if let Some((x, y)) = self.mouse_position {
                if now.duration_since(self.last_dot_add_time) >= self.dot_add_interval {
                    self.add_dot_if_not_exists(x as i32, y as i32);
                }
            }
        }

        self.update_physics();

        // ブレンド結果を適用
        let mut to_be_removed: Vec<usize> = Vec::new();
        let mut changes: Vec<(usize, MaterialDNA)> = Vec::new();

        for result in self.result_rx.try_iter() {
            match result {
                BlendResult::Change { index, new_dna } => {
                    changes.push((index, new_dna));
                }
                BlendResult::Vanish { index } => {
                    to_be_removed.push(index);
                }
            }
        }

        // 変更を適用
        for (index, new_dna) in changes {
            if let Some(dot) = self.dots.get_mut(index) {
                dot.material_dna = new_dna;
                dot.material = from_dna(&dot.material_dna);
            }
        }

        // 重複を削除し、降順にソートしてインデックスのズレを防ぐ
        to_be_removed.sort_unstable();
        to_be_removed.dedup();
        to_be_removed.reverse();

        for index in to_be_removed {
            if index < self.dots.len() {
                self.dots.remove(index);
            }
        }

        if now.duration_since(self.last_fps_update).as_secs_f32() > 0.5 {
            let sum: f64 = self.frame_times.iter().sum();

            if !self.frame_times.is_empty() {
                self.fps = self.frame_times.len() as f64 / sum;
            }

            self.last_fps_update = now;
        }

        let window = self.window.as_ref().unwrap();

        let hovered_material = self
            .hovered_dot_index
            .map(|i| self.dots[i].material.clone());

        let mut ui_data = crate::renderer::gui::UiData {
            fps: self.fps,

            dot_count: self.dots.len(),

            hovered_material,
        };

        if let Some(renderer) = &mut self.renderer {
            let (randomize_clicked, clear_clicked) = renderer.render(window, &self.dots, &mut ui_data); // 戻り値を受け取る

            if randomize_clicked {
                self.randomize_brush_material();
            }
            if clear_clicked { // CLSボタンがクリックされたら
                self.clear_dots(); // ドットをクリア
            }
        }
    }
}
