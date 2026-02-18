use crate::material::{from_dna, to_dna, BaseMaterialParams, MaterialDNA};
use crate::physics::engine::DOT_RADIUS;
use crate::physics::{engine, Physics};
use crate::renderer::Renderer;
use rand::thread_rng;
use rand::Rng;
use std::sync::{mpsc, Arc};
use winit::window::{Window, WindowBuilder};

// ドットの状態を保持する構造体
#[derive(Clone)]
pub struct Dot {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub vx: f64, // x方向速度
    pub vy: f64, // y方向速度
    pub material: BaseMaterialParams,
    pub material_dna: MaterialDNA, // 物質DNA
    pub name: String,              // 自動生成された名前
    pub reaction_count: u32,
    pub last_reaction_time: std::time::Instant,
    pub last_check_time: std::time::Instant, // 最後の確率判定時刻
    pub is_selected: bool,                   // 選択状態
    pub glowing_since: Option<std::time::Instant>,
    pub last_heat_exchange_time: std::time::Instant, // 最後の熱交換時刻
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
    pub dots: Vec<Dot>,                 // ドットリスト
    pub gravity: f64,                   // 重力加速度
    pub last_time: std::time::Instant,  // 時間管理用
    pub start_time: std::time::Instant, // 経過時間用
    pub physics: Physics,

    pub is_updating: bool,                     // 物理更新中かどうかのフラグ
    pub left_mouse_pressed: bool,              // 左クリックが押されているか
    pub last_dot_add_time: std::time::Instant, // 最後にドットを追加した時刻
    pub dot_add_interval: std::time::Duration, // ドット追加の間隔
    pub frame_times: std::collections::VecDeque<f64>,
    pub last_fps_update: std::time::Instant,
    pub fps: f64,
    pub brush_material: BaseMaterialParams, // 現在選択中の物質
    pub brush_seed: u64,                    // ブラシのシード
    pub selected_dot_id: Option<u64>,       // マウスがクリックしたドットのID
    pub next_dot_id: u64,                   // 次に生成するドットのID

    // 非同期処理用
    pub result_rx: mpsc::Receiver<BlendResult>, // ブレンド結果受信

    // Test features
    pub last_test_dot_add_time: std::time::Instant,
    pub test_dot_add_interval: std::time::Duration,
    pub is_test_mode_enabled: bool,
    pub max_test_dots: u32,
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

impl App {
    pub fn new(
        collision_tx: mpsc::Sender<((usize, MaterialDNA), (usize, MaterialDNA))>,
        result_rx: mpsc::Receiver<BlendResult>,
        is_test_mode_enabled: bool,
        max_test_dots: u32,
    ) -> Self {
        Self {
            window: None,

            renderer: None,

            mouse_position: None,

            dots: Vec::new(),

            gravity: 9.8 * 20.0,

            last_time: std::time::Instant::now(),
            start_time: std::time::Instant::now(),
            physics: Physics::new(collision_tx.clone()),

            is_updating: false,

            left_mouse_pressed: false,

            last_dot_add_time: std::time::Instant::now(),

            dot_add_interval: std::time::Duration::from_millis(100),

            frame_times: std::collections::VecDeque::with_capacity(100),

            last_fps_update: std::time::Instant::now(),

            fps: 0.0,

            brush_material: BaseMaterialParams::default(),

            brush_seed: 0,

            selected_dot_id: None,
            next_dot_id: 0,
            result_rx,

            // Test features
            last_test_dot_add_time: std::time::Instant::now(),
            test_dot_add_interval: std::time::Duration::from_millis(1000), // 1000ms = 1秒
            is_test_mode_enabled,
            max_test_dots,
        }
    }

    // ブラシの物質をランダム化

    fn randomize_brush_material(&mut self) {
        let mut rng = thread_rng();
        self.brush_seed = rng.gen();
        self.brush_material = crate::material::from_seed(self.brush_seed);
    }

    fn add_random_dots(&mut self) {
        let mut rng = thread_rng();
        let num_dots_to_add = rng.gen_range(10..=100);

        for _ in 0..num_dots_to_add {
            let x = rng.gen_range(DOT_RADIUS..WIDTH as f64 - DOT_RADIUS);
            let y = rng.gen_range(DOT_RADIUS..HEIGHT as f64 - DOT_RADIUS);

            let seed: u64 = rng.gen();
            let material = crate::material::from_seed(seed);
            let material_dna = crate::material::to_dna(&material, seed);
            let name = crate::naming::generate_name(&material_dna);

            let dot = Dot {
                id: self.next_dot_id,
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                material,
                material_dna,
                name,
                reaction_count: 0,
                last_reaction_time: std::time::Instant::now(),
                last_check_time: std::time::Instant::now(),
                is_selected: false,
                glowing_since: None,
                last_heat_exchange_time: std::time::Instant::now(),
            };
            self.dots.push(dot);
            self.next_dot_id += 1;
        }
        self.is_updating = true;
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
        let material_dna = to_dna(&self.brush_material, self.brush_seed);
        let name = crate::naming::generate_name(&material_dna);

        let dot = Dot {
            id: self.next_dot_id,
            x: x as f64,
            y: y as f64,
            vx: 0.0,
            vy: 0.0,
            material: self.brush_material.clone(), // ブラシの物質を適用
            material_dna,
            name,
            reaction_count: 0,
            last_reaction_time: std::time::Instant::now(),
            last_check_time: std::time::Instant::now(),
            is_selected: false,
            glowing_since: None,
            last_heat_exchange_time: std::time::Instant::now(),
        };

        self.dots.push(dot);
        self.next_dot_id += 1;

        self.is_updating = true;

        self.last_time = std::time::Instant::now();

        self.last_dot_add_time = std::time::Instant::now();
    }

    pub fn handle_resume(&mut self, event_loop: &winit::event_loop::EventLoopWindowTarget<()>) {
        if self.window.is_none() {
            let window = Arc::new(
                WindowBuilder::new()
                    .with_title("terraspiel")
                    .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
                    .build(event_loop)
                    .expect("Failed to create window"),
            );

            self.window = Some(window.clone());

            self.renderer = Some(Renderer::new(&window, event_loop));
        }

        self.last_time = std::time::Instant::now();
        self.start_time = std::time::Instant::now();
        self.last_dot_add_time = std::time::Instant::now();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(new_size);
        }
    }

    pub fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = Some((position.x, position.y));
    }

    pub fn handle_mouse_input(
        &mut self,

        state: winit::event::ElementState,

        button: winit::event::MouseButton,
    ) {
        match button {
            winit::event::MouseButton::Left => {
                self.left_mouse_pressed = state == winit::event::ElementState::Pressed;
                if self.left_mouse_pressed {
                    if let Some((x, y)) = self.mouse_position {
                        self.add_dot_if_not_exists(x as i32, y as i32);
                    }
                }
            }
            winit::event::MouseButton::Right => {
                if state == winit::event::ElementState::Pressed {
                    if let Some((x, y)) = self.mouse_position {
                        let mut clicked_dot_id = None;
                        // クリック位置のドットを探す
                        for dot in self.dots.iter().rev() {
                            let dx = dot.x - x;
                            let dy = dot.y - y;
                            if (dx * dx + dy * dy) < (DOT_RADIUS * DOT_RADIUS) {
                                clicked_dot_id = Some(dot.id);
                                break;
                            }
                        }

                        // selected_dot_id を更新
                        self.selected_dot_id = clicked_dot_id;

                        // is_selected フラグを更新
                        for dot in self.dots.iter_mut() {
                            dot.is_selected = Some(dot.id) == clicked_dot_id;
                        }

                        if let Some(ref window) = self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn update_physics(&mut self) {
        if !self.is_updating {
            return;
        }

        let now = std::time::Instant::now();

        let dt = now.duration_since(self.last_time).as_secs_f64();

        self.last_time = now;

        if let Some(ref renderer) = self.renderer {
            let device = renderer.get_device();
            let queue = renderer.get_queue();

            // GPUリソースを初期化
            if self.physics.compute_pipeline.is_none() {
                self.physics.initialize_gpu_resources(device);
            }

            // GPUリソースを更新
            self.physics.update_gpu_resources(device, queue, &self.dots, dt);

            // GPUで物理演算を実行
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Physics Encoder"),
            });
            self.physics.update_gpu_physics(device, queue, &mut encoder);
            queue.submit(std::iter::once(encoder.finish()));

            // GPUからCPUへのデータ同期
            self.physics.sync_gpu_to_cpu(device, queue, &mut self.dots);
        }

        // GPUが利用可能でも、CPUでの衝突判定と位置更新を行う
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

        // --- Test Dot Generation ---
        if self.is_test_mode_enabled
            && now.duration_since(self.last_test_dot_add_time) >= self.test_dot_add_interval
            && self.dots.len() < self.max_test_dots as usize
        {
            self.add_random_dots();
            self.last_test_dot_add_time = now;
        }

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
                dot.name = crate::naming::generate_name(&dot.material_dna);
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

        let (hovered_material, hovered_dot_dna, hovered_dot_name, _hovered_dot_velocity) =
            if let Some(selected_id) = self.selected_dot_id {
                self.dots
                    .iter()
                    .find(|d| d.id == selected_id)
                    .map_or((None, None, None, None), |dot| {
                        (
                            Some(dot.material.clone()),
                            Some(dot.material_dna.clone()),
                            Some(dot.name.clone()),
                            Some((dot.vx, dot.vy)),
                        )
                    })
            } else {
                (None, None, None, None)
            };

        let ui_data = crate::renderer::gui::UiData {
            fps: self.fps,
            dot_count: self.dots.len(),
            selected_material: hovered_material,
            selected_dot_dna: hovered_dot_dna,
            selected_dot_name: hovered_dot_name,
        };

        if let Some(renderer) = &mut self.renderer {
            let time = self.start_time.elapsed().as_secs_f32();
            let (randomize_clicked, clear_clicked) =
                renderer.render(window, &self.dots, &ui_data, time); // 戻り値を受け取る

            if randomize_clicked {
                self.randomize_brush_material();
            }
            if clear_clicked {
                // CLSボタンがクリックされたら
                self.clear_dots(); // ドットをクリア
            }
        }
    }
}
