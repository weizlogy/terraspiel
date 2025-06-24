/*! src/core/biomes/plains.rs */

use super::BiomeGenerator;
use crate::core::generation::{BiomeParameters, DetermineTileParameters};
use crate::core::world::World;
use crate::core::material::{Terrain, Overlay};
use crate::core::rng::GameRngMethods;

/// 平原バイオームの生成ストラテジー。
pub struct PlainsBiome;

impl BiomeGenerator for PlainsBiome {
  /// 平原バイオームのパラメータを生成する。
  fn generate_biome_parameters(&self, surface_y: usize, rng: &mut dyn GameRngMethods) -> BiomeParameters {
    let rock_layer_start_depth_offset_min: i32 = 30;
    let rock_layer_start_depth_offset_max: i32 = 50;
    let rock_layer_y = (surface_y as i32 + rng.gen_i32_range(rock_layer_start_depth_offset_min, rock_layer_start_depth_offset_max)) as usize;
    BiomeParameters { rock_layer_y }
  }

  /// 平原のタイルを決定する。地質は森林に似ている。
  fn determine_tile(&self, params: &DetermineTileParameters, rng: &mut dyn GameRngMethods) -> (Terrain, Overlay) {    // 1. 平原バイオームの基本的な地質を決定
    let mut terrain = if params.y >= params.rock_layer_y { Terrain::Rock } else { Terrain::Dirt };

    // 2. 他のバイオームとのブレンド
    //    今回は、隣接するバイオームが持つ「砂」や「氷」の性質を、
    //    それぞれのバイオームの重みに応じて平原に混ぜ込む
    let sand_from_desert = params.biome_weights.desert * (0.05 - (params.y as f64 / 700.0).max(0.0)); // 砂漠の砂 (少なめ、かなり浅いほど)
    let ice_from_snow = params.biome_weights.snow * (0.1 - (params.y as f64 / 600.0).max(0.0));   // 雪原の氷 (やや少なめ、浅いほど)

    //    これらの影響を合計して、平原の地形に影響を与える
    if terrain == Terrain::Dirt {
        if rng.gen_f64() < sand_from_desert { terrain = Terrain::Sand; }
        if rng.gen_f64() < ice_from_snow { terrain = Terrain::HardIce; }
    }

    // 3. 鉱石の生成
    if terrain == Terrain::Dirt || terrain == Terrain::Rock {
        if params.ore_noise.coal > 0.65 { terrain = Terrain::Coal; } // 平原は石炭がやや出やすい
        if params.ore_noise.iron > 0.7 { terrain = Terrain::Iron; }
    }

    (terrain, Overlay::Air)
  }

  /// 洞窟内の液体を生成する。分布は森林と似ている。
  fn place_fluids_in_cave(&self, world: &mut World, params: &DetermineTileParameters, water_noise_val: f64, lava_noise_val: f64) {
    let water_threshold = 0.65;
    let lava_threshold = 0.75;

    if params.y > params.rock_layer_y + 20 {
      if lava_noise_val > lava_threshold {
        world.set_overlay(params.x, params.y, Overlay::Lava);
      }
    } else if params.y > params.surface_y + 15 {
      if water_noise_val > water_threshold {
        world.set_overlay(params.x, params.y, Overlay::Water);
      }
    }
  }

  /// 地表の装飾を配置する。平原は草が多く、木はまばら。
  fn place_surface_decorations(&self, world: &mut World, x: usize, surface_y: usize, rng: &mut dyn GameRngMethods) {
    if surface_y > 0 {
      let y_on_surface = surface_y - 1;
      if let Some(surface_tile) = world.get(x, surface_y) {
        if surface_tile.terrain == Terrain::Dirt && world.get_overlay(x, y_on_surface) == Some(&Overlay::Air) {
          // 平原は木がまばらで、草が多い
          if rng.gen_bool_prob(0.02) {
            // 非常に低い確率で単一ブロックの木を生成
            world.set_overlay(x, y_on_surface, Overlay::Tree);
          } else if rng.gen_bool_prob(0.8) {
            world.set_overlay(x, y_on_surface, Overlay::Grass);
          }
        }
      }
    }
  }
}