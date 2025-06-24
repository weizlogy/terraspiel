/*! src/core/biomes/snowland.rs */

use super::BiomeGenerator; // BiomeGenerator trait のみインポート
use crate::core::generation::{BiomeParameters, DetermineTileParameters}; // generation.rs から構造体をインポート
use crate::core::world::World;
use crate::core::material::{Terrain, Overlay}; // TerrainとOverlayをインポート
use crate::core::rng::GameRngMethods; // GameRngMethodsトレイトをインポート

pub struct SnowlandBiome;

impl BiomeGenerator for SnowlandBiome {
  fn generate_biome_parameters(&self, surface_y: usize, rng: &mut dyn GameRngMethods) -> BiomeParameters {
    let rock_layer_start_depth_offset_min: i32 = 30;
    let rock_layer_start_depth_offset_max: i32 = 50;
    let rock_layer_y = (surface_y as i32 + rng.gen_i32_range(rock_layer_start_depth_offset_min, rock_layer_start_depth_offset_max)) as usize;
    BiomeParameters { rock_layer_y }
  }

  fn determine_tile(&self, params: &DetermineTileParameters, rng: &mut dyn GameRngMethods) -> (Terrain, Overlay) {
    // 1. 雪原バイオームの基本的な地質を決定
    let mut terrain = if params.y >= params.rock_layer_y { Terrain::Rock } else { Terrain::Dirt };

    // 2. 他のバイオームとのブレンド
    //    今回は、隣接するバイオームが持つ「土」や「砂」の性質を、
    //    それぞれのバイオームの重みに応じて雪原に混ぜ込む
    let dirt_from_forest = params.biome_weights.forest * (0.3 - (params.y as f64 / 500.0).min(0.2)); // 森林の土 (多め、浅いほど影響)
    let sand_from_desert = params.biome_weights.desert * (0.1 - (params.y as f64 / 800.0).max(0.0)); // 砂漠の砂 (少なめ、かなり浅いほど)
    let dirt_from_plains = params.biome_weights.plains * (0.2 - (params.y as f64 / 600.0).min(0.1)); // 平原の土 (やや多め、浅いほど)

    //    これらの「土っぽさ」「砂っぽさ」を合計して、雪原の地形に影響を与える
    let total_non_ice_influence = dirt_from_forest + sand_from_desert + dirt_from_plains;
    if terrain != Terrain::Rock && rng.gen_f64() < total_non_ice_influence {
        terrain = if rng.gen_f64() < dirt_from_forest / total_non_ice_influence.max(0.01) { // 影響度で比較
            Terrain::Dirt
        } else {
            Terrain::Sand // 砂 or その他 (今回はほぼ土)
        };
    } else if terrain != Terrain::Rock && terrain != Terrain::Dirt && rng.gen_f64() < 0.3 {
        terrain = Terrain::HardIce; // HardIceを混ぜる
    }

    // 3. 鉱石の生成
    if terrain == Terrain::Dirt || terrain == Terrain::Rock {
        if params.ore_noise.coal > 0.6 + (params.biome_weights.forest * 0.05) { terrain = Terrain::Coal; } // 森林の影響で石炭が少し増えるかも
        if params.ore_noise.iron > 0.75 + (params.biome_weights.desert * 0.1) { terrain = Terrain::Iron; } // 砂漠の影響で鉄が増えるかも
    }

    (terrain, Overlay::Air)
  }

  fn place_fluids_in_cave(&self, world: &mut World, params: &DetermineTileParameters, water_noise_val: f64, lava_noise_val: f64) {
    // 雪原: 水は多く、溶岩は出にくい
    let water_threshold = 0.5;
    let lava_threshold = 0.8;

    if params.y > params.rock_layer_y + 20 { // ワールドのかなり深い位置
      if lava_noise_val > lava_threshold {
        world.set_overlay(params.x, params.y, Overlay::Lava);
      }
    } else if params.y > params.surface_y + 15 { // 中深度
      if water_noise_val > water_threshold {
        world.set_overlay(params.x, params.y, Overlay::Water);
      }
    }
  }

  fn place_surface_decorations(&self, world: &mut World, x: usize, surface_y: usize, rng: &mut dyn GameRngMethods) {
    if surface_y > 0 {
      let y_on_surface = surface_y - 1;
      if let Some(surface_tile) = world.get(x, surface_y) {
        if surface_tile.terrain == Terrain::Dirt {
          if world.get_overlay(x, y_on_surface) == Some(&Overlay::Air) {
            if rng.gen_bool_prob(0.4) { world.set_overlay(x, y_on_surface, Overlay::Snow); }
            if rng.gen_bool_prob(0.02) { world.set_overlay(x, y_on_surface, Overlay::Tree); }
          }
        }
      }
    }
  }
}