use crate::material::{BaseMaterialParams, State};
use crate::renderer::Renderer;
use rand::Rng;
use std::sync::Arc;
use winit::window::{Window, WindowBuilder};

// ドットの状態を保持する構造体
#[derive(Clone)]
pub struct Dot {
    pub x: f64,
    pub y: f64,
    pub vx: f64, // x方向速度
    pub vy: f64, // y方向速度
    pub material: BaseMaterialParams,
}

impl Dot {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0, // 初期速度は0
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
    pub dots: Vec<Dot>,                // ドットリスト
    pub gravity: f64,                  // 重力加速度
    pub last_time: std::time::Instant, // 時間管理用

    pub is_updating: bool,                     // 物理更新中かどうかのフラグ
    pub left_mouse_pressed: bool,              // 左クリックが押されているか
    pub last_dot_add_time: std::time::Instant, // 最後にドットを追加した時刻
    pub dot_add_interval: std::time::Duration, // ドット追加の間隔
    pub frame_times: std::collections::VecDeque<f64>,
    pub last_fps_update: std::time::Instant,
    pub fps: f64,
    pub brush_material: BaseMaterialParams, // 現在選択中の物質
    pub hovered_dot_index: Option<usize>,   // マウスがホバーしているドット
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;
pub const DOT_RADIUS: f64 = 2.0;
const GAS_REFERENCE_DENSITY: f32 = 0.5;
const GAS_DIFFUSION_FACTOR: f64 = 5.0;

impl App {
    pub fn new() -> Self {
        Self {
            window: None,

            renderer: None,

            mouse_position: None,

            dots: Vec::new(),

            gravity: 9.8 * 10.0,

            last_time: std::time::Instant::now(),

            is_updating: false,

            left_mouse_pressed: false,

            last_dot_add_time: std::time::Instant::now(),

            dot_add_interval: std::time::Duration::from_millis(100),

            frame_times: std::collections::VecDeque::with_capacity(100),

            last_fps_update: std::time::Instant::now(),

            fps: 0.0,

            brush_material: BaseMaterialParams::default(),

            hovered_dot_index: None,
        }
    }

    // ブラシの物質をランダム化

    fn randomize_brush_material(&mut self) {
        let mut rng = rand::thread_rng();

        let state_choice = rng.gen_range(0..3);

        let state = match state_choice {
            0 => State::Solid,

            1 => State::Liquid,

            _ => State::Gas,
        };

        self.brush_material.state = state;

        self.brush_material.density = rng.random(); // 0.0 ~ 1.0

        self.brush_material.color_hue = rng.random(); // 0.0 ~ 1.0

        self.brush_material.viscosity = rng.random();

        self.brush_material.hardness = rng.random();

        self.brush_material.elasticity = rng.random();
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
        let mut dot = Dot::new(x as f64, y as f64);

        dot.material = self.brush_material.clone(); // ブラシの物質を適用

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

        let mut rng = rand::thread_rng();

        // 1. 状態に基づいて力を適用

        for dot in &mut self.dots {
            match dot.material.state {
                State::Solid | State::Liquid | State::Particle => {
                    dot.vy += self.gravity * dt;
                }

                State::Gas => {
                    let buoyancy =
                        (GAS_REFERENCE_DENSITY - dot.material.density) as f64 * self.gravity;

                    dot.vy -= buoyancy * dt;

                    let diffusion_strength =
                        (1.0 - dot.material.viscosity) as f64 * GAS_DIFFUSION_FACTOR;

                    dot.vx += (rng.random::<f64>() - 0.5) * diffusion_strength * dt;

                    dot.vy += (rng.random::<f64>() - 0.5) * diffusion_strength * dt;
                }
            }
        }

        // 2. 衝突判定と応答

        let mut all_stopped = true;

        let num_dots = self.dots.len();

        for i in 0..num_dots {
            for j in (i + 1)..num_dots {
                let (dot1_slice, dot2_slice) = self.dots.split_at_mut(j);

                let dot1 = &mut dot1_slice[i];

                let dot2 = &mut dot2_slice[0];

                let dx = dot2.x - dot1.x;

                let dy = dot2.y - dot1.y;

                let distance_sq = dx * dx + dy * dy;

                let min_dist = DOT_RADIUS * 2.0;

                if distance_sq < min_dist * min_dist && distance_sq > 1e-6 {
                    let distance = distance_sq.sqrt();

                    let overlap = 0.5 * (min_dist - distance);

                    let nx = dx / distance;

                    let ny = dy / distance;

                    dot1.x -= overlap * nx;

                    dot1.y -= overlap * ny;

                    dot2.x += overlap * nx;

                    dot2.y += overlap * ny;

                    match (dot1.material.state, dot2.material.state) {
                        (State::Solid, State::Solid) | (State::Liquid, State::Liquid) => {
                            handle_detailed_collision(dot1, dot2, nx, ny, dt);
                        }

                        (State::Solid, State::Liquid) => {
                            if dot1.material.density > dot2.material.density
                                && dot1.material.viscosity > dot2.material.viscosity
                            {
                                // Solid is immovable, Liquid bounces off

                                let e = (dot1.material.elasticity + dot2.material.elasticity)
                                    as f64
                                    / 2.0;

                                let v_liquid_n = dot2.vx * nx + dot2.vy * ny;

                                if v_liquid_n < 0.0 {
                                    // Liquid moving towards Solid

                                    dot2.vx -= (1.0 + e) * v_liquid_n * nx;

                                    dot2.vy -= (1.0 + e) * v_liquid_n * ny;
                                }
                            } else {
                                handle_detailed_collision(dot1, dot2, nx, ny, dt);
                            }
                        }

                        (State::Liquid, State::Solid) => {
                            if dot2.material.density > dot1.material.density
                                && dot2.material.viscosity > dot1.material.viscosity
                            {
                                // Solid is immovable, Liquid bounces off

                                let e = (dot1.material.elasticity + dot2.material.elasticity)
                                    as f64
                                    / 2.0;

                                // Normal is from dot1->dot2. Liquid is dot1. Check against inverse normal.

                                let v_liquid_n = dot1.vx * (-nx) + dot1.vy * (-ny);

                                if v_liquid_n < 0.0 {
                                    // Liquid moving towards Solid

                                    dot1.vx -= (1.0 + e) * v_liquid_n * (-nx);

                                    dot1.vy -= (1.0 + e) * v_liquid_n * (-ny);
                                }
                            } else {
                                handle_detailed_collision(dot1, dot2, nx, ny, dt);
                            }
                        }

                        (State::Gas, State::Gas) => {
                            handle_gas_collision(dot1, dot2, nx, ny);
                        }

                        (State::Solid, State::Gas) | (State::Liquid, State::Gas) => {
                            handle_gas_displacement(dot2, dot1, nx, ny);
                        }

                        (State::Gas, State::Solid) | (State::Gas, State::Liquid) => {
                            handle_gas_displacement(dot1, dot2, nx, ny);
                        }

                        _ => {
                            handle_simple_collision(dot1, dot2, nx, ny);
                        }
                    }
                }
            }
        }

        // 3. 位置更新と壁との衝突

        for dot in &mut self.dots {
            dot.x += dot.vx * dt;

            dot.y += dot.vy * dt;

            let elasticity = dot.material.elasticity as f64;

            if dot.y >= (HEIGHT as f64 - DOT_RADIUS) {
                dot.y = HEIGHT as f64 - DOT_RADIUS;

                dot.vy *= -elasticity;
            }

            if dot.y <= DOT_RADIUS {
                dot.y = DOT_RADIUS;

                dot.vy *= -elasticity;
            }

            if dot.x >= (WIDTH as f64 - DOT_RADIUS) {
                dot.x = WIDTH as f64 - DOT_RADIUS;

                dot.vx *= -elasticity;
            }

            if dot.x <= DOT_RADIUS {
                dot.x = DOT_RADIUS;

                dot.vx *= -elasticity;
            }

            if dot.material.state != State::Gas {
                let velocity_small = dot.vy.abs() < 0.1 && dot.vx.abs() < 0.1;

                let at_bottom = dot.y >= (HEIGHT as f64 - DOT_RADIUS - 1.0);

                if !(velocity_small && at_bottom) {
                    all_stopped = false;
                }
            } else {
                all_stopped = false;
            }
        }

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
            if renderer.render(window, &self.dots, &mut ui_data) {
                self.randomize_brush_material();
            }
        }
    }
}

// --- ヘルパー関数 ---

// Solid/Liquid間の詳細な衝突処理

fn handle_detailed_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;

    let m1 = dot1.material.density as f64;

    let m2 = dot2.material.density as f64;

    let v1n = dot1.vx * nx + dot1.vy * ny;

    let v2n = dot2.vx * nx + dot2.vy * ny;

    let v1n_new = (m1 * v1n + m2 * v2n - m2 * e * (v1n - v2n)) / (m1 + m2);

    let v2n_new = (m1 * v1n + m2 * v2n + m1 * e * (v1n - v2n)) / (m1 + m2);

    dot1.vx += (v1n_new - v1n) * nx;

    dot1.vy += (v1n_new - v1n) * ny;

    dot2.vx += (v2n_new - v2n) * nx;

    dot2.vy += (v2n_new - v2n) * ny;

    let density_diff = dot1.material.density - dot2.material.density;

    if density_diff.abs() > 0.1 {
        if dot1.y < dot2.y && density_diff > 0.0 {
            dot1.vy += density_diff as f64 * 5.0 * dt;

            dot2.vy -= density_diff as f64 * 5.0 * dt;
        } else if dot2.y < dot1.y && density_diff < 0.0 {
            dot2.vy += density_diff.abs() as f64 * 5.0 * dt;

            dot1.vy -= density_diff.abs() as f64 * 5.0 * dt;
        }
    }

    let avg_viscosity = (dot1.material.viscosity + dot2.material.viscosity) / 2.0;

    if avg_viscosity < 0.5 && ny.abs() > 0.8 {
        let spread_force = (1.0 - avg_viscosity) as f64 * 10.0 * dt;

        if dot1.x < dot2.x {
            dot1.vx -= spread_force;

            dot2.vx += spread_force;
        } else {
            dot1.vx += spread_force;

            dot2.vx -= spread_force;
        }
    }
}

// Gas間の衝突処理

fn handle_gas_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
    let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;

    let m1 = dot1.material.density as f64;

    let m2 = dot2.material.density as f64;

    let v1n = dot1.vx * nx + dot1.vy * ny;

    let v2n = dot2.vx * nx + dot2.vy * ny;

    let v1n_new = (m1 * v1n + m2 * v2n - m2 * e * (v1n - v2n)) / (m1 + m2);

    let v2n_new = (m1 * v1n + m2 * v2n + m1 * e * (v1n - v2n)) / (m1 + m2);

    dot1.vx += (v1n_new - v1n) * nx;

    dot1.vy += (v1n_new - v1n) * ny;

    dot2.vx += (v2n_new - v2n) * nx;

    dot2.vy += (v2n_new - v2n) * ny;
}

// Gasが他の物体に押し出される処理

fn handle_gas_displacement(gas: &mut Dot, other: &Dot, nx: f64, ny: f64) {
    let e = (gas.material.elasticity + other.material.elasticity) as f64 / 2.0;

    let v_gas_n = gas.vx * nx + gas.vy * ny;

    // nx,nyは常に gas->other を指すため、v_gas_nが正なら向かっている

    if v_gas_n > 0.0 {
        gas.vx -= (1.0 + e) * v_gas_n * nx;

        gas.vy -= (1.0 + e) * v_gas_n * ny;
    }
}

// シンプルな速度交換

fn handle_simple_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
    let k = (dot2.vx - dot1.vx) * nx + (dot2.vy - dot1.vy) * ny;

    dot1.vx += k * nx;

    dot1.vy += k * ny;

    dot2.vx -= k * nx;

    dot2.vy -= k * ny;
}
