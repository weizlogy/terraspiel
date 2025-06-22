// ゲームロジック

use crate::core::world::{World, WIDTH, HEIGHT}; // Terrain は material.rs から直接使うので不要になるかも
use crate::core::material::{Terrain, Overlay}; // Overlay も使うよ！
use crate::core::player::{self, Player, PlayerAction}; // ✨ player モジュールと関連するものをインポート！
use crate::core::rng::GameRng; // ✨ 共通の乱数生成器をインポート！

const GRAVITY: u8 = 1;
const MAX_FALL_SPEED: u8 = 6;

pub fn update_game_state(world: &mut World, player: &mut Player, coords: &mut [(usize, usize)], player_actions: &[PlayerAction], rng: &mut GameRng) {
  // プレイヤーの更新を物理演算の前に実行！
  player::update(player, world, player_actions);

  // 座標をシャッフルして、更新順序をランダムにする
  rng.shuffle(coords);

  for &(x, y) in coords.iter() {
    // 現在のタイルを取得。取得できなければ次の座標へ。
    let Some(tile_at_xy) = world.get(x, y).copied() else { continue; }; // copied() で Tile の値をコピーするよ

    let current_terrain = tile_at_xy.terrain;
    let current_overlay = tile_at_xy.overlay;

    // --- A. 草専用の落下処理 ---
    // もし現在のオーバーレイが草で、かつ真下が完全に空いていたら、草だけ1マス落下させる。
    // これは、例えば草の下の土ブロックが他の要因で消滅し、草だけが空中に取り残された場合などに適用されるよ。
    if current_overlay == Overlay::Grass {
      if y + 1 < HEIGHT { // ワールドの底でなければ落下を試みる
        // 真下のタイル情報を取得
        if let Some(tile_below) = world.get(x, y + 1) {
          // 真下が「地形が空(Empty)」かつ「オーバーレイも空気(Air)」の場合のみ草は落下する
          if tile_below.terrain == Terrain::Empty && tile_below.overlay == Overlay::Air {
            world.set_overlay(x, y, Overlay::Air);       // 元の場所の草を消去
            world.set_overlay(x, y + 1, Overlay::Grass); // 1マス下に草を再設置
            // 草が動いたので、この座標 (x,y) に関する今フレームの物理演算はここまで。
            // 次のフレーム、またはこのフレームの後の処理で、移動後の草や元の場所の地形が再度評価されることになるよ。
            continue;
          }
        }
      }
    }

    // --- 1. 地形自体の動き (Dirt, Sand など。タイル全体が動く) ---
    if current_terrain == Terrain::Dirt || current_terrain == Terrain::Sand {
      if y + 1 >= HEIGHT { // 最下段なら落下できない
        continue; // この座標の処理は終わり
      }

      // 1つ下のタイルを取得。y+1 < HEIGHT は上でチェック済みなので unwrap() でも安全だけど、
      // 安全のため Option を使うか、let Some(...) else { ... } を使うのがよりRustらしいかも。
      let Some(below_tile_for_terrain) = world.get(x, y + 1) else { continue; };
      if below_tile_for_terrain.terrain == Terrain::Empty { // 地形が落下できる場合
        let mut speed = world.get_fall_speed(x, y);
        speed = (speed + GRAVITY).min(MAX_FALL_SPEED);
        let distance = speed as usize;
        let mut fall_dist = 0;

        for dy_terrain in 1..=distance {
          // 落下先のY座標が範囲外ならループ終了
          if y + dy_terrain >= HEIGHT { break; } // ガード節
          let check_tile = world.get(x, y + dy_terrain).unwrap();
          if check_tile.terrain == Terrain::Empty {
            fall_dist = dy_terrain;
          } else {
            break;
          }
        }

        if fall_dist > 0 {
          world.swap_terrain(x, y, x, y + fall_dist); // タイル全体を交換
          // `swap_terrain` は Tile 構造体全体 (terrain と overlay の両方) を交換する。
          // そのため、もし Terrain::Dirt の上に Overlay::Grass があった場合、Grass も Dirt と一緒に移動するよ！
          world.set_fall_speed(x, y + fall_dist, speed);
          world.set_fall_speed(x, y, 0);
          continue; // 地形が動いたので、この座標の処理は完了
        }
      } else { // 地形が何かに乗っている場合
        world.set_fall_speed(x, y, 0); // 接地したら速度リセット // ガード節ではないけど、ネスト解消のため移動
        if current_terrain.should_attempt_slide(world, x, y) {
          if try_slide_terrain(world, x, y, rng) { // try_slide_terrain は以前の try_slide
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
        let mut speed = world.get_fall_speed(x, y); // タイルの現在の落下速度を利用 // ガード節ではないけど、ネスト解消のため移動
        speed = (speed + GRAVITY).min(MAX_FALL_SPEED);
        let distance_to_check = speed as usize;
        let mut overlay_fall_dist = 0;

        for dy_o in 1..=distance_to_check {
          if y + dy_o >= HEIGHT { break; } // ガード節
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
        if try_slide_overlay(world, x, y, current_overlay, rng) {
          continue; // オーバーレイが横滑りしたので、この (x,y) は完了
        }
      }
    }
  }
}

// 地形全体をスライドさせる (以前の try_slide 関数)
fn try_slide_terrain(world: &mut World, x: usize, y: usize, rng: &mut GameRng) -> bool {
  let mut did_slide = false;
  let offsets = if rng.random_bool(0.5) {
    [(-1, 1), (1, 1)] // 左下、右下
  } else {
    [(1, 1), (-1, 1)] // 右下、左下
  };

  for (dx, dy) in offsets {
    let nx = x as isize + dx;
    let ny = y as isize + dy;

    if nx >= 0 && nx < WIDTH as isize && ny >= 0 && ny < HEIGHT as isize {
      let (nx, ny) = (nx as usize, ny as usize); // usize に変換

      // 移動先のタイルを取得。取得できなければ次のオフセットへ。
      let Some(target_tile) = world.get(nx, ny) else { continue; }; // ガード節

      if target_tile.terrain == Terrain::Empty { // 移動先が空ならスライド可能
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
fn try_slide_overlay(world: &mut World, x: usize, y: usize, overlay_to_move: Overlay, rng: &mut GameRng) -> bool {
  let mut did_slide = false;
  // オーバーレイは左右均等に広がりたいので、(-1,0)と(1,0)も候補に入れるとより自然かも。
  // 今回は斜め下への広がりを維持。
  let offsets = if rng.random_bool(0.5) { [(-1, 1), (1, 1)] } else { [(1, 1), (-1, 1)] };

  // dyは常に1 (斜め下) と仮定してループ
  for (dx, dy) in offsets { // dyは常に1 (斜め下)
    let nx = x as isize + dx;
    let ny = y as isize + dy; // 水平方向の広がりも考慮するなら dy は 0 もあり得る

    if nx >= 0 && nx < WIDTH as isize && ny >= 0 && ny < HEIGHT as isize {
      let (nx, ny) = (nx as usize, ny as usize); // usize に変換

      // 移動先のタイルを取得。取得できなければ次のオフセットへ。
      let Some(target_tile) = world.get(nx, ny) else { continue; }; // ガード節

      if target_tile.overlay.is_replaceable_by_fluid() { // 移動先のオーバーレイが空気など置き換え可能なものか // ガード節
          world.set_overlay(x, y, Overlay::Air); // 元の場所は空気になる
          world.set_overlay(nx, ny, overlay_to_move); // 新しい場所にオーバーレイを移動
          world.set_fall_speed(nx, ny, 0); // スライド先は速度0
          world.set_fall_speed(x, y, 0); // 元の場所も速度0
          did_slide = true;
          break;
        }
    }
  }
  did_slide
}
