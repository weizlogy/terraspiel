use crate::material::BaseMaterialParams;
use pixels::Pixels;
use winit::window::Window;

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
    pub window: Option<Window>,
    pub pixels: Option<Pixels>,
    pub mouse_position: Option<(f64, f64)>,
    pub dots: Vec<Dot>,        // ドットリスト
    pub gravity: f64,          // 重力加速度
    pub last_time: std::time::Instant,  // 時間管理用
    pub bounce_factor: f64,    // 反発係数
    pub is_updating: bool,     // 物理更新中かどうかのフラグ
    pub left_mouse_pressed: bool, // 左クリックが押されているか
    pub last_dot_add_time: std::time::Instant, // 最後にドットを追加した時刻
    pub dot_add_interval: std::time::Duration, // ドット追加の間隔
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

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
            left_mouse_pressed: false, // 左クリック押下状態の初期値
            last_dot_add_time: std::time::Instant::now(), // ドット追加時刻の初期値
            dot_add_interval: std::time::Duration::from_millis(100), // 100msごとにドット追加
        }
    }

    // ドット追加（簡略化版 - 常にドットを追加）
    pub fn add_dot_if_not_exists(&mut self, x: i32, y: i32) {
        // 位置の重複チェックを一旦外す
        self.dots.push(Dot::new(x as f64, y as f64));
        self.is_updating = true; // 物理更新を開始
        self.last_dot_add_time = std::time::Instant::now(); // 最後に追加した時刻を更新
    }
}