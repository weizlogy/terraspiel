use palette::{FromColor, Hsl, RgbHue, Srgb};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

/// 物質の状態 (固体/液体/気体/粒子)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Solid,    // 固体
    Liquid,   // 液体
    Gas,      // 気体
    Particle, // 粒子（例：粉体）
}

impl State {
    /// 状態のエネルギーレベルを返す (plan.md参照)
    pub fn get_energy_level(&self) -> f32 {
        match self {
            State::Solid => 0.2,
            State::Particle => 0.4,
            State::Liquid => 0.6,
            State::Gas => 0.9,
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
    pub density: f32,    // 密度 (0.0 ~ 1.0)
    pub viscosity: f32,  // 粘度 (0.0 ~ 1.0)
    pub hardness: f32,   // 硬度 (0.0 ~ 1.0)
    pub elasticity: f32, // 弾性 (0.0 ~ 1.0)
    pub melting_point: f32, // 融点 (0.0 ~ 1.0)
    pub boiling_point: f32, // 沸点 (0.0 ~ 1.0)
    pub flammability: f32, // 可燃性 (0.0 ~ 1.0)

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
            melting_point: 0.3,
            boiling_point: 0.7,
            flammability: 0.1,
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

pub fn from_seed(seed: u64) -> BaseMaterialParams {
    let mut rng = StdRng::seed_from_u64(seed);

    let state = match rng.gen_range(0..=3) {
        0 => State::Solid,
        1 => State::Liquid,
        2 => State::Gas,
        _ => State::Particle,
    };

    BaseMaterialParams {
        state,
        density: rng.gen(),
        viscosity: rng.gen(),
        hardness: rng.gen(),
        elasticity: rng.gen(),
        melting_point: rng.gen(),
        boiling_point: rng.gen(),
        flammability: rng.gen(),
        temperature: rng.gen::<f32>() * 2.0 - 1.0, // -1.0 ~ 1.0
        heat_conductivity: rng.gen(),
        heat_capacity: rng.gen(),
        conductivity: rng.gen(),
        magnetism: rng.gen::<f32>() * 2.0 - 1.0, // -1.0 ~ 1.0
        color_hue: rng.gen(),
        color_saturation: rng.gen(),
        color_luminance: rng.gen(),
        luminescence: rng.gen(),
    }
}

/// 物質のすべてを決定する数値列 (plan.md参照)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDNA {
    pub seed: u64,
    /// 各特性を0〜1正規化した値。順序はBaseMaterialParamsのフィールドに対応。
    pub genes: [f32; 17],
}

impl MaterialDNA {
    /// 2つのDNAを線形補間でブレンドする
    /// ratio: 0.0でa、1.0でbになる
    pub fn blend(&self, other: &Self, ratio: f32) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut new_genes = [0.0; 17];
        for i in 0..17 {
            new_genes[i] = self.genes[i] * (1.0 - ratio) + other.genes[i] * ratio;
        }

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
        x if (0.0..0.25).contains(&x) => State::Solid,
        x if (0.25..0.5).contains(&x) => State::Liquid,
        x if (0.5..0.75).contains(&x) => State::Gas,
        _ => State::Particle,
    };

    BaseMaterialParams {
        state,
        density: dna.genes[1],
        viscosity: dna.genes[2],
        hardness: dna.genes[3],
        elasticity: dna.genes[4],
        melting_point: dna.genes[5],
        boiling_point: dna.genes[6],
        flammability: dna.genes[7],
        temperature: dna.genes[8] * 2.0 - 1.0, // 0..1 to -1..1
        heat_conductivity: dna.genes[9],
        heat_capacity: dna.genes[10],
        conductivity: dna.genes[11],
        magnetism: dna.genes[12] * 2.0 - 1.0, // 0..1 to -1..1
        color_hue: dna.genes[13],
        color_saturation: dna.genes[14],
        color_luminance: dna.genes[15],
        luminescence: dna.genes[16],
    }
}

/// BaseMaterialParamsからDNAへの変換
pub fn to_dna(params: &BaseMaterialParams, seed: u64) -> MaterialDNA {
    let state_gene = match params.state {
        State::Solid => 0.125,
        State::Liquid => 0.375,
        State::Gas => 0.625,
        State::Particle => 0.875,
    };

    MaterialDNA {
        seed,
        genes: [
            state_gene,
            params.density,
            params.viscosity,
            params.hardness,
            params.elasticity,
            params.melting_point,
            params.boiling_point,
            params.flammability,
            (params.temperature + 1.0) / 2.0, // -1..1 to 0..1
            params.heat_conductivity,
            params.heat_capacity,
            params.conductivity,
            (params.magnetism + 1.0) / 2.0, // -1..1 to 0..1
            params.color_hue,
            params.color_saturation,
            params.color_luminance,
            params.luminescence,
        ],
    }
}

/// DNAシードに基づいて手続き的な名前を生成する
pub fn generate_material_name(seed: u64) -> String {
    use rand::Rng;
    use rand_seeder::Seeder;

    let mut rng: rand::rngs::StdRng = Seeder::from(seed).make_rng();

    let vowels = ["a", "i", "u", "e", "o", "ya", "yu", "yo"];
    let consonants = [
        "k", "s", "t", "n", "h", "m", "r", "w", "g", "z", "d", "b", "p",
    ];
    let special_syllables = ["n", "xtu", "ltsu"];

    let mut name = String::new();
    let length = rng.gen_range(2..=5); // 2〜5音節

    for i in 0..length {
        // たまに特殊な音節を入れる
        if rng.gen_bool(0.1) {
            name.push_str(special_syllables[rng.gen_range(0..special_syllables.len())]);
        } else {
            let consonant = consonants[rng.gen_range(0..consonants.len())];
            let vowel = vowels[rng.gen_range(0..vowels.len())];
            name.push_str(consonant);
            name.push_str(vowel);
        }

        // 最初の文字を大文字にする
        if i == 0 {
            if let Some(first) = name.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
        }
    }

    name
}

impl MaterialDNA {
    /// このDNAに基づいた物質名を生成する
    pub fn get_name(&self) -> String {
        generate_material_name(self.seed)
    }
}
