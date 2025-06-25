// src/render/render.rs に配置してください

use crate::core::camera::{Camera, VIEW_WIDTH, VIEW_HEIGHT};
use crate::core::player::Player;
use crate::core::world::World;

/// 2つのRGBAカラーをアルファブレンディングで合成するヘルパー関数。
fn blend_colors(bottom: [u8; 4], top: [u8; 4]) -> [u8; 4] {
  let top_alpha = top[3] as f32 / 255.0;
  if top_alpha == 0.0 {
    return bottom;
  }
  if top_alpha >= 1.0 {
    return top;
  }

  let bottom_alpha = bottom[3] as f32 / 255.0;
  let out_alpha = top_alpha + bottom_alpha * (1.0 - top_alpha);
  if out_alpha == 0.0 {
    return [0, 0, 0, 0];
  }

  let r = (top[0] as f32 * top_alpha + bottom[0] as f32 * bottom_alpha * (1.0 - top_alpha)) / out_alpha;
  let g = (top[1] as f32 * top_alpha + bottom[1] as f32 * bottom_alpha * (1.0 - top_alpha)) / out_alpha;
  let b = (top[2] as f32 * top_alpha + bottom[2] as f32 * bottom_alpha * (1.0 - top_alpha)) / out_alpha;

  [r as u8, g as u8, b as u8, (out_alpha * 255.0) as u8]
}

/// ゲーム画面を描画する。カメラに映る範囲のみをレンダリングするよ。
pub fn draw_game(world: &World, player: &Player, camera: &Camera, frame: &mut [u8]) {
  let cam_x = camera.x.floor() as isize;
  let cam_y = camera.y.floor() as isize;

  // ビューポート（画面）の各ピクセルをループ処理
  for screen_y in 0..VIEW_HEIGHT {
    for screen_x in 0..VIEW_WIDTH {
      // スクリーン座標からワールド座標を計算
      let world_x = cam_x + screen_x as isize;
      let world_y = cam_y + screen_y as isize;

      // フレームバッファ内の対応するピクセル位置を計算
      let i = (screen_y * VIEW_WIDTH + screen_x) * 4;
      let pixel = &mut frame[i..i + 4];

      // ワールド座標からタイルの色を取得
      let tile_color = if world_x >= 0 && world_y >= 0 {
        // `world.get` は Option を返すので、安全にタイルを取得できる
        if let Some(tile) = world.get(world_x as usize, world_y as usize) {
          let terrain_color = tile.terrain.color();
          let overlay_color = tile.overlay.color();
          // 地形の色とオーバーレイの色をブレンドして最終的な色を決定
          blend_colors(terrain_color, overlay_color)
        } else {
          // ワールドの範囲外（でもVecの範囲内）はデフォルトの背景色
          [20, 20, 30, 255]
        }
      } else {
        // ワールドの範囲外はデフォルトの背景色
        [20, 20, 30, 255]
      };

      pixel.copy_from_slice(&tile_color);
    }
  }

  // --- プレイヤーの描画 ---
  // プレイヤーのワールド座標を、カメラを基準としたスクリーン座標に変換
  let player_screen_x = player.x - camera.x;
  let player_screen_y = player.y - camera.y;

  // プレイヤーのサイズを定義（ここでは仮に 1x2 タイルサイズ）
  let player_width = 1.0;
  let player_height = 2.0;

  // プレイヤーを描画するピクセル範囲を計算
  let start_x = player_screen_x.floor() as isize;
  let end_x = (player_screen_x + player_width).ceil() as isize;
  let start_y = player_screen_y.floor() as isize;
  let end_y = (player_screen_y + player_height).ceil() as isize;

  // プレイヤーの色（目立つようにピンク！💖）
  let player_color = [255, 0, 255, 255];

  // プレイヤーの各ピクセルを描画
  for y in start_y..end_y {
    for x in start_x..end_x {
      // ピクセルが画面内に収まっているかチェック
      if x >= 0 && x < VIEW_WIDTH as isize && y >= 0 && y < VIEW_HEIGHT as isize {
        let i = (y as usize * VIEW_WIDTH + x as usize) * 4;
        frame[i..i + 4].copy_from_slice(&player_color);
      }
    }
  }
}
