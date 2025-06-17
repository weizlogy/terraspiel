// 描画

use crate::core::world::{World, WIDTH, HEIGHT};
use crate::core::material::{Overlay, Terrain};

pub fn draw_world(world: &World, frame: &mut [u8]) {
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
}
