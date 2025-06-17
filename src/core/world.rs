// 世界そのものの定義

use crate::core::material::{Terrain, Overlay};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;

#[derive(Clone, Copy, Debug)]
pub struct Tile {
  pub terrain: Terrain,
  pub overlay: Overlay,
}

// Defaultトレイトを実装して、Tile::default()で空タイルを作れるようにするよ！
impl Default for Tile {
  fn default() -> Self {
    Tile::empty()
  }
}

impl Tile {
  pub fn empty() -> Self {
    Tile {
      terrain: Terrain::Empty,
      overlay: Overlay::Air,
    }
  }

  pub fn is_solid(&self) -> bool {
    self.terrain.is_solid() || self.overlay.is_solid()
  }
}

pub struct World {
  pub tiles: Box<[Tile]>,
}

impl World {
  pub fn new() -> Self {
    World {
      tiles: vec![Tile::default(); WIDTH * HEIGHT].into_boxed_slice(),
    }
  }

  pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
    if x < WIDTH && y < HEIGHT {
      // 1次元配列を2次元座標でアクセスするためのインデックス計算だよ！
      Some(&self.tiles[y * WIDTH + x])
    } else {
      None
    }
  }
  pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
    if x < WIDTH && y < HEIGHT {
      // こっちも同じくインデックス計算でアクセスしよう！
      Some(&mut self.tiles[y * WIDTH + x])
    } else {
      None
    }
  }

  pub fn set_overlay(&mut self, x: usize, y: usize, overlay: Overlay) {
    if let Some(tile) = self.get_mut(x, y) {
      tile.overlay = overlay;
    }
  }

  pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
    if let Some(tile) = self.get_mut(x, y) {
      tile.terrain = terrain;
    }
  }
}
