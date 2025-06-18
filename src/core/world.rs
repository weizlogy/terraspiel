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
  pub tiles: Vec<Vec<Option<Tile>>>,
  fall_speeds: Box<[u8]>,
}

impl World {
  pub fn new() -> Self {
    World {
      // tiles を Vec<Vec<Option<Tile>>> として正しく初期化するよ！
      // 各マスにはデフォルトで Some(Tile::empty()) が入るようにしてみようか！
      tiles: vec![vec![Some(Tile::empty()); WIDTH]; HEIGHT],
      fall_speeds: vec![0; WIDTH * HEIGHT].into_boxed_slice(),
    }
  }

  pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
    // 2次元配列としてアクセスし、中の Option<&Tile> から &Tile を取り出すよ
    self.tiles.get(y)
      .and_then(|row| row.get(x))
      .and_then(|option_tile| option_tile.as_ref())
  }

  pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
    // 2次元配列としてミュータブルにアクセスし、中の Option<&mut Tile> から &mut Tile を取り出すよ
    self.tiles.get_mut(y)
      .and_then(|row| row.get_mut(x))
      .and_then(|option_tile| option_tile.as_mut())
  }

  pub fn get_fall_speed(&self, x: usize, y: usize) -> u8 {
    // fall_speeds は1次元配列のままなので、インデックス計算が必要だね
    self.fall_speeds[y * WIDTH + x]
  }

  pub fn set_fall_speed(&mut self, x: usize, y: usize, speed: u8) {
    // fall_speeds は1次元配列のままなので、インデックス計算が必要だね
    self.fall_speeds[y * WIDTH + x] = speed;
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

  pub fn swap_terrain(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
    // 2つの座標が範囲内であることを確認してからスワップするよ
    if x1 < WIDTH && y1 < HEIGHT && x2 < WIDTH && y2 < HEIGHT {
      // 同じ行なら Vec::swap が使えるね！
      if y1 == y2 {
        if x1 != x2 { // 同じ要素をスワップしても意味ないからね
          self.tiles[y1].swap(x1, x2);
        }
      } else {
        // 異なる行の場合は、一度取り出して入れ替えるのが安全だよ
        // (Rustの借用規則で、同時に2つの異なる要素へのミュータブルな参照を取るのがちょっと難しいからね)
        let tile1_opt = self.tiles[y1][x1].take();
        let tile2_opt = self.tiles[y2][x2].take();
        self.tiles[y1][x1] = tile2_opt;
        self.tiles[y2][x2] = tile1_opt;
      }
    }
  }

  pub fn count_stack_height(&self, x: usize, y: usize) -> usize {
    let mut count = 0;
    for dy in 1..=4 {
      if y >= dy {
        // get は Option<&Tile> を返すので、ちゃんと中身を確認しようね！
        if let Some(tile_above) = self.get(x, y - dy) {
          if tile_above.terrain == Terrain::Dirt {
            count += 1;
          } else {
            break;
          }
        } else { // タイルが存在しない (None) か、範囲外 (getがNoneを返す) ならループを抜ける
          break;
        }
      }
    }
    count
  }
}
