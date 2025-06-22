// 描画

use crate::core::world::{World, WIDTH, HEIGHT};
use crate::core::player::Player;
use crate::core::material::{Overlay};

pub fn draw_game(world: &World, player: &Player, frame: &mut [u8]) {
  for y in 0..HEIGHT {
    for x in 0..WIDTH {
      let i = (y * WIDTH + x) * 4;
      let tile = world.get(x, y).unwrap();

      // まずはテレインの色でピクセルを初期化
      let mut final_color = tile.terrain.color();

      // オーバーレイが存在し、かつ透明でなければテレインの色を上書き
      if tile.overlay != Overlay::None && tile.overlay != Overlay::Air {
        let overlay_color = tile.overlay.color();
        // アルファ値が0より大きい場合のみ上書き（完全に透明なオーバーレイは無視）
        if overlay_color[3] > 0 {
          final_color = overlay_color;
        }
      }

      frame[i..i + 4].copy_from_slice(&final_color);
    }
  }

  // --- プレイヤーの描画 ---
  // ワールドを描画した後に、プレイヤーを四角形で上書き描画するよ！
  let player_color = [255, 0, 255, 255]; // とりあえず目立つピンク色！
  // プレイヤーの座標をピクセル座標に変換
  let px_min = player.x.floor() as usize;
  let py_min = player.y.floor() as usize;
  let px_max = (player.x + player.width).ceil() as usize;
  let py_max = (player.y + player.height).ceil() as usize;

  for y in py_min..py_max.min(HEIGHT) {
    for x in px_min..px_max.min(WIDTH) {
      let i = (y * WIDTH + x) * 4;
      frame[i..i + 4].copy_from_slice(&player_color);
    }
  }
}
