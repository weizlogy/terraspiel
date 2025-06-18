// ゲームロジック

use rand::seq::SliceRandom;
use rand::Rng;

use crate::core::world::{World, WIDTH, HEIGHT}; // Terrain は material.rs から直接使うので不要になるかも
use crate::core::material::{Terrain, Overlay}; // Overlay も使うよ！

const GRAVITY: u8 = 1;
const MAX_FALL_SPEED: u8 = 6;

pub fn update_world(world: &mut World, coords: &mut [(usize, usize)]) {
  coords.shuffle(&mut rand::rng());

  for &(x, y) in coords.iter() {
    let tile_at_xy = match world.get(x, y) {
        Some(tile_ref) => *tile_ref, // Tile は Copy トレイトを実装してるから、ここで値がコピーされるよ！
        None => continue, // タイルが存在しなければスキップ
    };
    let current_terrain = tile_at_xy.terrain;
    let current_overlay = tile_at_xy.overlay;

    // --- 1. 地形自体の動き (Dirt, Sand など。タイル全体が動く) ---
    if current_terrain == Terrain::Dirt || current_terrain == Terrain::Sand {
      if y + 1 >= HEIGHT { // 最下段なら落下できない
        continue; // この座標の処理は終わり
      }

      let below_tile_for_terrain = world.get(x, y + 1).unwrap(); // y+1 < HEIGHT は上でチェック済み

      if below_tile_for_terrain.terrain == Terrain::Empty { // 地形が落下できる場合
        let mut speed = world.get_fall_speed(x, y);
        speed = (speed + GRAVITY).min(MAX_FALL_SPEED);
        let distance = speed as usize;
        let mut fall_dist = 0;

        for dy_terrain in 1..=distance {
          if y + dy_terrain >= HEIGHT { break; }
          let check_tile = world.get(x, y + dy_terrain).unwrap();
          if check_tile.terrain == Terrain::Empty {
            fall_dist = dy_terrain;
          } else {
            break;
          }
        }

        if fall_dist > 0 {
          world.swap_terrain(x, y, x, y + fall_dist); // タイル全体を交換
          world.set_fall_speed(x, y + fall_dist, speed);
          world.set_fall_speed(x, y, 0);
          continue; // 地形が動いたので、この座標の処理は完了
        }
      } else { // 地形が何かに乗っている場合
        world.set_fall_speed(x, y, 0); // 接地したら速度リセット
        if current_terrain.should_attempt_slide(world, x, y) {
          if try_slide_terrain(world, x, y) { // try_slide_terrain は以前の try_slide
            continue; // 地形が横滑りしたので、この座標の処理は完了
          }
        }
      }
      // Dirt/Sand の場合、ここまでで落下も横滑りもしなかったとしても、
      // この (x,y) の処理は完了とする (下のオーバーレイ単独の動きのロジックには進まない)。
      continue;
    }

    // --- 2. オーバーレイの動き (Water など。オーバーレイのみが動く) ---
    // 地形が Dirt/Sand ではなかった場合に、このロジックが実行される。
    if current_overlay.can_flow() {
      if y + 1 < HEIGHT { // 最下段でない場合のみ落下を試みる
        let mut speed = world.get_fall_speed(x, y); // タイルの現在の落下速度を利用
        speed = (speed + GRAVITY).min(MAX_FALL_SPEED);
        let distance_to_check = speed as usize;
        let mut overlay_fall_dist = 0;

        for dy_o in 1..=distance_to_check {
          if y + dy_o >= HEIGHT { break; }
          if let Some(tile_below_for_overlay) = world.get(x, y + dy_o) {
            // オーバーレイが落下できるのは、下のタイルのオーバーレイが置き換え可能で、
            // かつ、下のタイルの地形が流体の通過を許容する場合
            if tile_below_for_overlay.overlay.is_replaceable_by_fluid() && tile_below_for_overlay.terrain.allows_fluid_passthrough() {
              overlay_fall_dist = dy_o;
            } else {
              break; // 置き換え不可能なオーバーレイにぶつかった
            }
          } else { break; } // ワールド範囲外 (get が None)
        }

        if overlay_fall_dist > 0 {
          world.set_overlay(x, y, Overlay::Air); // 元の場所は空気になる
          world.set_overlay(x, y + overlay_fall_dist, current_overlay); // 新しい場所にオーバーレイを移動
          world.set_fall_speed(x, y + overlay_fall_dist, speed); // 移動先のタイルに速度を設定
          world.set_fall_speed(x, y, 0); // 元の場所の速度はリセット
          continue; // オーバーレイが落下したので、この (x,y) は完了
        }
      }

      // 落下しなかった、または最下段だった場合、速度をリセット
      world.set_fall_speed(x, y, 0);
      // 横滑りを試みる
      if current_overlay.should_attempt_slide(world, x, y) { // should_attempt_slide は Overlay のメソッド
        if try_slide_overlay(world, x, y, current_overlay) {
          continue; // オーバーレイが横滑りしたので、この (x,y) は完了
        }
      }
    }
  }
}

// 地形全体をスライドさせる (以前の try_slide 関数)
fn try_slide_terrain(world: &mut World, x: usize, y: usize) -> bool {
  let mut did_slide = false;
  let mut rng = rand::rng();
  let offsets = if rng.random_bool(0.5) {
    [(-1, 1), (1, 1)] // 左下、右下
  } else {
    [(1, 1), (-1, 1)] // 右下、左下
  };

  for (dx, dy) in offsets {
    let nx = x as isize + dx;
    let ny = y as isize + dy;

    if nx >= 0 && nx < WIDTH as isize && ny >= 0 && ny < HEIGHT as isize {
      let nx = nx as usize;
      let ny = ny as usize;

      if world.get(nx, ny).unwrap().terrain == Terrain::Empty {
        world.swap_terrain(x, y, nx, ny); // タイル全体を交換
        // スライドした場合、移動先の速度は元の速度を引き継ぐか、0にするか。
        // ここでは元の速度を引き継がず、0にリセットしておく (スライド先で安定する想定)。
        // 必要であれば、元の速度を world.get_fall_speed(x,y) で取得して設定も可能。
        world.set_fall_speed(nx, ny, 0); // スライド先は速度0
        world.set_fall_speed(x, y, 0); // 元の場所も速度0
        did_slide = true;
        break;
      }
    }
  }
  did_slide
}

// オーバーレイのみをスライドさせる
fn try_slide_overlay(world: &mut World, x: usize, y: usize, overlay_to_move: Overlay) -> bool {
  let mut did_slide = false;
  let mut rng = rand::rng();
  // オーバーレイは左右均等に広がりたいので、(-1,0)と(1,0)も候補に入れるとより自然かも。
  // 今回は斜め下への広がりを維持。
  let offsets = if rng.random_bool(0.5) { [(-1, 1), (1, 1)] } else { [(1, 1), (-1, 1)] };

  for (dx, dy) in offsets { // dyは常に1 (斜め下)
    let nx = x as isize + dx;
    let ny = y as isize + dy; // 水平方向の広がりも考慮するなら dy は 0 もあり得る

    if nx >= 0 && nx < WIDTH as isize && ny >= 0 && ny < HEIGHT as isize {
      let nx = nx as usize;
      let ny = ny as usize;
      if let Some(target_tile) = world.get(nx, ny) {
        if target_tile.overlay.is_replaceable_by_fluid() { // 移動先のオーバーレイが空気など置き換え可能なものか
          world.set_overlay(x, y, Overlay::Air); // 元の場所は空気になる
          world.set_overlay(nx, ny, overlay_to_move); // 新しい場所にオーバーレイを移動
          world.set_fall_speed(nx, ny, 0); // スライド先は速度0
          world.set_fall_speed(x, y, 0); // 元の場所も速度0
          did_slide = true;
          break;
        }
      }
    }
  }
  did_slide
}
