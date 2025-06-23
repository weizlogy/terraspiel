/*! src/core/biomes/forest.rs */

use super::BiomeGenerator; // BiomeGenerator trait のみインポート
use crate::core::generation::{BiomeParameters, DetermineTileParameters, BiomeWeights}; // generation.rs から構造体をインポート
use crate::core::world::World;
use crate::core::material::{Terrain, Overlay}; // TerrainとOverlayをインポート
use crate::core::rng::GameRngMethods; // GameRngMethodsトレイトをインポート

pub struct ForestBiome;

impl BiomeGenerator for ForestBiome {
  fn generate_biome_parameters(&self, surface_y: usize, rng: &mut dyn GameRngMethods) -> BiomeParameters {
    let rock_layer_start_depth_offset_min: i32 = 25;
    let rock_layer_start_depth_offset_max: i32 = 45;
    let rock_layer_y = (surface_y as i32 + rng.gen_i32_range(rock_layer_start_depth_offset_min, rock_layer_start_depth_offset_max)) as usize;
    BiomeParameters { rock_layer_y }
  }

  fn determine_tile(&self, params: &DetermineTileParameters, rng: &mut dyn GameRngMethods) -> (Terrain, Overlay) {
    // 1. 森林バイオームの基本的な地質を決定
    let mut terrain = if params.y >= params.rock_layer_y { Terrain::Rock } else { Terrain::Dirt };

    // 2. 他のバイオームとのブレンド
    //    今回は、隣接するバイオームが持つ「砂」や「氷」の性質を、
    //    それぞれのバイオームの重みに応じて森林に混ぜ込む
    let sand_from_desert = params.biome_weights.desert * (0.1 - (params.y as f64 / 800.0).max(0.0)); // 砂漠の砂 (浅いほど影響)
    let ice_from_snow = params.biome_weights.snow * (0.2 - (params.y as f64 / 600.0).max(0.0));   // 雪原の氷 (やや浅いほど影響)

    //    これらの「砂っぽさ」「氷っぽさ」を合計して、森林の地形に影響を与える
    if terrain == Terrain::Dirt {
        if rng.gen_f64() < sand_from_desert { terrain = Terrain::Sand; }
        if rng.gen_f64() < ice_from_snow { terrain = Terrain::HardIce; }
    }

    // 3. 鉱石の生成
    if terrain == Terrain::Dirt || terrain == Terrain::Rock {
        if params.ore_noise.coal > 0.55 { terrain = Terrain::Coal; }
        if params.ore_noise.iron > 0.7 + (params.biome_weights.desert * 0.1) { terrain = Terrain::Iron; } // 砂漠の影響で鉄が増えるかも
    }

    (terrain, Overlay::Air)
  }

  fn place_fluids_in_cave(&self, world: &mut World, params: &DetermineTileParameters, water_noise_val: f64, lava_noise_val: f64) {
    // 森林: 標準的な確率
    let water_threshold = 0.6;
    let lava_threshold = 0.7;

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
        // 地表が土ブロックの場合のみ装飾を配置する
        if surface_tile.terrain == Terrain::Dirt {
          // 装飾を配置する場所が空いているか確認
          if world.get_overlay(x, y_on_surface) == Some(&Overlay::Air) {
            // まず木を生成しようと試みる
            if rng.gen_bool_prob(0.15) {
              let trunk_height = rng.gen_usize_range(3, 6); // 3-5ブロックの高さ
              // 木を置くための十分なスペースがあるか確認
              let can_place_tree = (0..trunk_height).all(|i| {
                y_on_surface >= i && world.get_overlay(x, y_on_surface - i) == Some(&Overlay::Air)
              });

              if can_place_tree {
                // 幹を設置
                for i in 0..trunk_height {
                  world.set_overlay(x, y_on_surface - i, Overlay::Tree);
                }
              }
            } else if rng.gen_bool_prob(0.7) {
              // 木が生成されなかった場合、草を生成
              world.set_overlay(x, y_on_surface, Overlay::Grass);
            }
          }
        }
      }
    }
  }
}