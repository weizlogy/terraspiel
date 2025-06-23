/*! src/core/biomes/mod.rs */

use crate::core::world::World; // World構造体をインポート
use crate::core::rng::GameRngMethods; // GameRngMethodsトレイトをインポート
use crate::core::generation::{BiomeParameters, DetermineTileParameters};

pub mod forest;
pub mod desert;
pub mod snowland;
pub mod plains;

/// ワールド内の異なる環境を示すバイオーム。
/// この enum は public にして、他のモジュールからも参照できるようにします。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
  Forest,
  Desert,
  Plains,
  Snowland,
}

/// 各バイオームが実装すべき振る舞いを定義するトレイト（インターフェース）。
/// これがストラテジーパターンの「戦略」の定義にあたります。
pub trait BiomeGenerator {
  /// バイオーム固有の地形パラメータ（岩盤層の深さなど）を返します。
  fn generate_biome_parameters(&self, surface_y: usize, rng: &mut dyn GameRngMethods) -> BiomeParameters;

  /// 指定された座標における地形とオーバーレイの種類を決定します。
  fn determine_tile(&self, params: &DetermineTileParameters, rng: &mut dyn GameRngMethods) -> (crate::core::material::Terrain, crate::core::material::Overlay);

  /// 洞窟内に液体（水、溶岩など）を生成します。このメソッドは、(x, y) が洞窟として確定した直後に呼び出されます。
  fn place_fluids_in_cave(&self, world: &mut World, params: &DetermineTileParameters, water_noise_val: f64, lava_noise_val: f64);

  /// 地表にバイオーム固有の装飾（草、木、雪など）を配置します。
  fn place_surface_decorations(&self, world: &mut World, x: usize, surface_y: usize, rng: &mut dyn GameRngMethods);
}
