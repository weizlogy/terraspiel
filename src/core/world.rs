// 世界そのものの定義

use crate::core::rng::GameRngMethods; // ✨ 共通の乱数生成器メソッドトレイトをインポート！
use crate::core::material::{Terrain, Overlay};

pub const WIDTH: usize = 1600; // ワールドの幅を2倍に
pub const HEIGHT: usize = 1200; // ワールドの高さも2倍に

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

  /// 指定された座標のオーバーレイへの参照を取得する。
  /// 座標が無効な場合は `None` を返すよ。
  pub fn get_overlay(&self, x: usize, y: usize) -> Option<&Overlay> {
    self.get(x, y).map(|tile| &tile.overlay)
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

  /// ワールド内で草を成長させる処理。
  ///
  /// この関数はゲームループから定期的に呼び出されることを想定しているよ。
  /// 光が届き、かつ土 (Dirt) である地形の上に、一定の確率で草 (Grass) を生成するんだ。
  ///
  /// # Arguments
  /// * `rng` - 乱数生成器への可変参照。草が生えるかどうかの確率判定に使うよ。
  ///
  /// # Note
  /// この関数はワールドの全列をスキャンするから、頻繁に呼びすぎるとパフォーマンスに影響が出るかもしれないよ。
  /// 呼び出し頻度を調整するか、将来的に部分的な更新を検討してみてね！😉
  pub fn grow_grass(&mut self, rng: &mut dyn GameRngMethods) {
    // ワールドの各列をスキャンするよ
    // 草の成長は上から光が当たる場所で起こるので、Y=0から順にスキャンするのが効率的だよ。
    for x in 0..WIDTH {
      // 各列で、Y=0 (一番上) から順に下にスキャンして、最初に光が遮られる場所を探す
      for y_ground in 0..HEIGHT {
        // まず、現在のタイルを取得。取得できなければ次のYへ (ありえないはずだけど安全のため)
        let Some(ground_tile) = self.get(x, y_ground) else { continue; };

        // 固体地形にぶつかったら、それより下は光が届かないのでこの列は終了。
        // ただし、草が生えるのは土の上なので、土以外の場合は単に光が遮られただけ。
        if ground_tile.terrain.is_solid() {
          // もし土だったら、その上に草を生やせるかチェックする。
          if ground_tile.terrain == Terrain::Dirt {
            // 土ブロックの1つ上 (y_ground - 1) が草を生やせる空間かチェック。
            // y_ground が0だと、その上はないのでスキップ。
            if y_ground == 0 { break; } // Y=0の土の上には草は生えない

            // 一つ上のタイルを取得。取得できなければ次のYへ (ありえないはずだけど安全のため)
            let Some(tile_above) = self.get(x, y_ground - 1) else { break; }; // 上のタイルがなければこの列は終了

            // 上のタイルが Empty Terrain かつ Air Overlay (完全に空間) であることを確認。
            if tile_above.terrain == Terrain::Empty && tile_above.overlay == Overlay::Air {
              // 草が生える条件がそろった！🎉
              // ゲームプレイ用のRNGを使用して、確率で草を生成します
              if rng.gen_bool_prob(0.005) { // 0.5% の確率で草を生成！🌱
                self.set_overlay(x, y_ground - 1, Overlay::Grass);
              }
            }
          }
        }
      }
    }
  }

  /// 指定された座標がワールド内で衝突可能な固体ブロックかどうかを判定します。
  /// ワールドの範囲外も固体として扱います。
  ///
  /// # Arguments
  /// * `x` - チェックするX座標 (タイル単位)
  /// * `y` - チェックするY座標 (タイル単位)
  ///
  /// # Returns
  /// * 座標が固体であれば `true`、そうでなければ `false`
  pub fn is_solid_at(&self, x: isize, y: isize) -> bool {
    if x < 0 || x >= WIDTH as isize || y < 0 || y >= HEIGHT as isize {
      return true; // ワールド外は壁とみなす
    }
    self.get(x as usize, y as usize).map_or(true, |tile| tile.is_solid())
  }
}
