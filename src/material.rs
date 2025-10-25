use palette::{FromColor, Hsl, RgbHue, Srgb};
use serde::{Deserialize, Serialize};

/// 物質の状態 (固体/液体/気体/粒子)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Solid,    // 固体
    Liquid,   // 液体
    Gas,      // 気体
    Particle, // 粒子（例：粉体）
}

/// ベース物質のパラメータ（特性）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMaterialParams {
    // 基本
    pub state: State,

    // 物理特性
    pub density: f32,    // 密度 (0.0 ~ 1.0)
    pub viscosity: f32,  // 粘度 (0.0 ~ 1.0)
    pub hardness: f32,   // 硬度 (0.0 ~ 1.0)
    pub elasticity: f32, // 弾性 (0.0 ~ 1.0)

    // 熱・エネルギー系
    pub temperature: f32,       // 相対温度 (-1.0 ~ 1.0)
    pub heat_conductivity: f32, // 熱伝導率 (0.0 ~ 1.0)
    pub heat_capacity: f32,     // 熱容量 (0.0 ~ 1.0)

    // 電磁特性
    pub conductivity: f32, // 電導率 (0.0 ~ 1.0)
    pub magnetism: f32,    // 磁性 (-1.0 ~ 1.0)

    // 光・見た目系
    pub color_hue: f32,        // 色相 (0.0 ~ 1.0)
    pub color_saturation: f32, // 彩度 (0.0 ~ 1.0)
    pub color_luminance: f32,  // 明度 (0.0 ~ 1.0)
    pub luminescence: f32,     // 自発光度 (0.0 ~ 1.0)
}

impl Default for BaseMaterialParams {
    fn default() -> Self {
        Self {
            state: State::Solid,
            density: 0.5,
            viscosity: 0.3,
            hardness: 0.7,
            elasticity: 0.2,
            temperature: 0.0,
            heat_conductivity: 0.4,
            heat_capacity: 0.6,
            conductivity: 0.1,
            magnetism: 0.0,
            color_hue: 0.5,
            color_saturation: 0.8,
            color_luminance: 0.6,
            luminescence: 0.0,
        }
    }
}

impl BaseMaterialParams {
    /// HSVからRGBを計算して返す
    /// H: 0.0~1.0, S: 0.0~1.0, L: 0.0~1.0
    pub fn get_color_rgb(&self) -> (u8, u8, u8) {
        // HSLからRGBへの変換
        let hsl = Hsl::new(
            RgbHue::from_degrees(self.color_hue * 360.0),
            self.color_saturation,
            self.color_luminance,
        );
        let rgb: Srgb<u8> = Srgb::from_color(hsl).into_format();

        (rgb.red, rgb.green, rgb.blue)
    }
}
