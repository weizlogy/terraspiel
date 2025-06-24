/*! src/core/biomes/desert.rs */

use super::BiomeGenerator; // BiomeGenerator trait のみインポート
use crate::core::generation::{BiomeParameters, DetermineTileParameters}; // generation.rs から構造体をインポート
use crate::core::world::World;
use crate::core::material::{Terrain, Overlay}; // TerrainとOverlayをインポート
use crate::core::rng::GameRngMethods; // GameRngMethodsトレイトをインポート

pub struct DesertBiome;

impl BiomeGenerator for DesertBiome {
  fn generate_biome_parameters(&self, surface_y: usize, rng: &mut dyn GameRngMethods) -> BiomeParameters {
    let rock_layer_start_depth_offset_min: i32 = 20;
    let rock_layer_start_depth_offset_max: i32 = 40;
    let rock_layer_y = (surface_y as i32 + rng.gen_i32_range(rock_layer_start_depth_offset_min, rock_layer_start_depth_offset_max).max(10)) as usize;
    BiomeParameters { rock_layer_y }
  }

  fn determine_tile(&self, params: &DetermineTileParameters, rng: &mut dyn GameRngMethods) -> (Terrain, Overlay) {
    // 1. 基本的な地質を決定する
    let base_terrain = if params.y >= params.rock_layer_y {
      Terrain::Rock
    } else {
      // 砂漠の地層。基本は砂だが、たまに岩が混じるようにして、のっぺり感をなくす
      let depth_in_sand_layer = params.y.saturating_sub(params.surface_y);
      // 浅いところでは岩はほとんどなく、深くなるにつれて少し増える
      let rock_probability = (depth_in_sand_layer as f64 / 100.0).min(0.2); // 最大20%の確率で岩が混じる
      if rng.gen_f64() < rock_probability {
        Terrain::Rock
      } else {
        Terrain::Sand
      }
    };

    // 2. もし地質が岩なら、鉱石に置き換わるか判定する
    if base_terrain == Terrain::Rock {
      // 砂漠は鉄が豊富という設定で、閾値を少し低めにする
      if params.ore_noise.iron > 0.6 {
        return (Terrain::Iron, Overlay::Air);
      }
    }

    // 鉱石に置き換わらなかった場合は、基本的な地質を返す
    (base_terrain, Overlay::Air)
  }

  fn place_fluids_in_cave(&self, world: &mut World, params: &DetermineTileParameters, water_noise_val: f64, lava_noise_val: f64) {
    // 砂漠: 水は少なく、溶岩は少し出やすい
    let water_threshold = 0.8;
    let lava_threshold = 0.65;

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
        if surface_tile.terrain == Terrain::Sand {
          if world.get_overlay(x, y_on_surface) == Some(&Overlay::Air) && rng.gen_bool_prob(0.03) {
            world.set_overlay(x, y_on_surface, Overlay::Tree); // サボテンのつもり
          }
        }
      }
    }
  }
}