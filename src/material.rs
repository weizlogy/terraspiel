use palette::{FromColor, Hsl, RgbHue, Srgb};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

/// 物質の状態 (固体/液体/気体/粒子)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Solid,  // 固体
    Liquid, // 液体
    Gas,    // 気体
}

impl State {
    /// 状態のエネルギーレベルを返す (plan.md参照)
    pub fn get_energy_level(&self) -> f32 {
        match self {
            State::Solid => 0.2,
            State::Liquid => 0.5,
            State::Gas => 0.8,
        }
    }
}

/// ブレンド反応の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionType {
    /// 相互変化: 両方の物質が変化する
    Reaction,
    /// 触媒（低→高）: エネルギーが高い方は変化せず、低い方のみ変化
    CatalyticLowChanges,
    /// 触媒（高→消滅）: エネルギーが高い方は変化し、低い方は消滅
    CatalyticHighChangesAndLowVanishes,
}

/// 2つの状態間のエネルギー差に基づいて反応タイプを決定する
pub fn decide_reaction_type(state_a: State, state_b: State) -> ReactionType {
    let energy_a = state_a.get_energy_level();
    let energy_b = state_b.get_energy_level();
    let delta_e = (energy_a - energy_b).abs();

    if delta_e < 0.2 {
        ReactionType::Reaction
    } else if delta_e < 0.5 {
        ReactionType::CatalyticLowChanges
    } else {
        ReactionType::CatalyticHighChangesAndLowVanishes
    }
}

/// ベース物質のパラメータ（特性）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMaterialParams {
    // 基本
    pub state: State,

    // 物理特性
    pub density: f32,       // 密度 (0.0 ~ 1.0)
    pub viscosity: f32,     // 粘度 (0.0 ~ 1.0)
    pub hardness: f32,      // 硬度 (0.0 ~ 1.0)
    pub elasticity: f32,    // 弾性 (0.0 ~ 1.0)

    // 熱・エネルギー系
    pub temperature: f32,       // 相対温度 (-1.0 ~ 1.0)
    pub heat_conductivity: f32, // 熱伝導率 (0.0 ~ 1.0)
    pub heat_capacity_high: f32,     // 熱容量(高) (0.0 ~ 1.0)
    pub heat_capacity_low: f32,      // 熱容量(低) (-1.0 ~ 0.0)

    // 電磁特性

    // 光・見た目系
    pub color_hue: f32,        // 色相 (0.0 ~ 1.0)
    pub color_saturation: f32, // 彩度 (0.0 ~ 1.0)
    pub color_luminance: f32,  // 明度 (0.0 ~ 1.0)
    pub luminescence: f32,     // 自発光度 (0.0 ~ 1.0)
    pub entropy_bias: f32,     // エントロピーバイアス (0.0 ~ 1.0)
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
            heat_capacity_high: 0.6,
            heat_capacity_low: -0.1,
            color_hue: 0.5,
            color_saturation: 0.8,
            color_luminance: 0.6,
            luminescence: 0.0,
            entropy_bias: 0.1,
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

pub fn from_seed(seed: u64) -> BaseMaterialParams {
    let mut rng = StdRng::seed_from_u64(seed);

    let state = match rng.gen_range(0..=2) {
        0 => State::Solid,
        1 => State::Liquid,
        _ => State::Gas,
    };

    BaseMaterialParams {
        state,
        density: rng.gen(),
        viscosity: rng.gen(),
        hardness: rng.gen(),
        elasticity: rng.gen(),
        temperature: rng.gen::<f32>() * 2.0 - 1.0, // -1.0 ~ 1.0
        heat_conductivity: rng.gen(),
        heat_capacity_high: rng.gen(),
        heat_capacity_low: rng.gen::<f32>() - 1.0,
        color_hue: rng.gen(),
        color_saturation: rng.gen(),
        color_luminance: rng.gen::<f32>() * 0.8 + 0.2, // 0.2-1.0の範囲にマッピング
        luminescence: rng.gen(),
        entropy_bias: rng.gen(),
    }
}

/// 物質のすべてを決定する数値列 (plan.md参照)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDNA {
    pub seed: u64,
    /// 各特性を0〜1正規化した値。順序はBaseMaterialParamsのフィールドに対応。
    pub genes: [f32; 14],
}

impl MaterialDNA {
    /// 2つのDNAを線形補間でブレンドする
    /// ratio: 0.0でa、1.0でbになる
    pub fn blend(&self, other: &Self, ratio: f32) -> Self {
        use rand::rngs::StdRng;
        use rand::Rng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut new_genes = [0.0; 14];

        // --- 他の特性は線形補間 ---
        for i in 0..14 {
            if i < 9 || i > 11 {
                new_genes[i] = self.genes[i] * (1.0 - ratio) + other.genes[i] * ratio;
            }
        }

        // --- 色の計算 ---
        let mut hasher_for_noise = DefaultHasher::new();
        self.seed.hash(&mut hasher_for_noise);
        other.seed.hash(&mut hasher_for_noise);
        ratio.to_bits().hash(&mut hasher_for_noise);
        let noise_seed = hasher_for_noise.finish();
        let mut rng = StdRng::seed_from_u64(noise_seed);

        // 9: color_hue
        let hue_a = self.genes[9];
        let hue_b = other.genes[9];
        let mut diff = hue_b - hue_a;
        // エントロピーバイアスが高いほど、より積極的な補間になる
        let non_linear_ratio = ratio.powf(2.0 - new_genes[13].clamp(0.0, 1.9)); // powfは負にならないように

        if diff.abs() > 0.5 {
            if diff > 0.0 {
                diff -= 1.0;
            } else {
                diff += 1.0;
            }
        }
        let blended_hue = hue_a + diff * non_linear_ratio;
        // エントロピーバイアスが高いほど、ノイズが大きくなる
        let noise_range = 0.005 + new_genes[13] * 0.1; // 0.005 ~ 0.105
        let noise = rng.gen_range(-noise_range..noise_range);
        new_genes[9] = (blended_hue + noise + 1.0) % 1.0; // 0-1の範囲に正規化

        // 10: color_saturation
        let density_a = self.genes[1];
        let density_b = other.genes[1];
        let total_density = density_a + density_b;
        let sat_a = self.genes[10];
        let sat_b = other.genes[10];

        let blended_saturation = if total_density > 0.0 {
            (sat_a * density_a + sat_b * density_b) / total_density
        } else {
            (sat_a + sat_b) / 2.0 // 密度が両方0の場合のフォールバック
        };

        let blended_temp_gene = new_genes[5]; // 0..1
        let temperature = blended_temp_gene * 2.0 - 1.0; // -1..1
        let temp_correction = temperature * 0.2; // 温度に応じて彩度が上下する (±20%)
        new_genes[10] = (blended_saturation * (1.0 + temp_correction)).clamp(0.0, 1.0);

        // 11: color_luminance
        new_genes[11] = rng.gen(); // 高止まりを防ぐため、完全にランダム化

        // ブレンド後のgenesからハッシュを計算して新しいseedとする
        let mut hasher = DefaultHasher::new();
        for &gene in &new_genes {
            gene.to_bits().hash(&mut hasher);
        }
        let mut new_seed = hasher.finish();

        // seedが0になるのを防ぐ
        if new_seed == 0 {
            new_seed = 1;
        }

        Self {
            seed: new_seed,
            genes: new_genes,
        }
    }
}

/// DNAからBaseMaterialParamsへの変換
pub fn from_dna(dna: &MaterialDNA) -> BaseMaterialParams {
    // genes[0] を State にマッピング
    let state = match dna.genes[0] {
        x if (0.0..0.33).contains(&x) => State::Solid,
        x if (0.33..0.66).contains(&x) => State::Liquid,
        _ => State::Gas,
    };

    BaseMaterialParams {
        state,
        density: dna.genes[1],
        viscosity: dna.genes[2],
        hardness: dna.genes[3],
        elasticity: dna.genes[4],
        temperature: dna.genes[5] * 2.0 - 1.0, // 0..1 to -1..1
        heat_conductivity: dna.genes[6],
        heat_capacity_high: dna.genes[7],
        heat_capacity_low: dna.genes[8] - 1.0,
        color_hue: dna.genes[9],
        color_saturation: dna.genes[10],
        color_luminance: dna.genes[11],
        luminescence: dna.genes[12],
        entropy_bias: dna.genes[13],
    }
}

/// BaseMaterialParamsからDNAへの変換
pub fn to_dna(params: &BaseMaterialParams, seed: u64) -> MaterialDNA {
    let state_gene = match params.state {
        State::Solid => 0.165,
        State::Liquid => 0.495,
        State::Gas => 0.825,
    };

    MaterialDNA {
        seed,
        genes: [
            state_gene,
            params.density,
            params.viscosity,
            params.hardness,
            params.elasticity,
            (params.temperature + 1.0) / 2.0, // -1..1 to 0..1
            params.heat_conductivity,
            params.heat_capacity_high,
            params.heat_capacity_low + 1.0,
            params.color_hue,
            params.color_saturation,
            params.color_luminance,
            params.luminescence,
            params.entropy_bias,
        ],
    }
}
