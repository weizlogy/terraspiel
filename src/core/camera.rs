//! src/core/camera.rs

use crate::core::player::Player;
use crate::core::world::{WIDTH as WORLD_WIDTH, HEIGHT as WORLD_HEIGHT};

// 描画するビューポート（カメラに映る範囲）のサイズを定義するよ。
// このサイズがウィンドウの内部解像度になるんだ。
pub const VIEW_WIDTH: usize = 300;
pub const VIEW_HEIGHT: usize = 200;

/// ゲームワールドを覗くカメラを表す構造体。
///
/// カメラの位置は、描画するワールド領域の左上の角に対応するよ。
#[derive(Debug)]
pub struct Camera {
  // カメラの左上のx, y座標 (ワールド座標系)。
  // f32にしておくと、ピクセル単位より滑らかな動きを表現できるからおすすめ！
  pub x: f32,
  pub y: f32,
}

impl Camera {
  /// 新しいカメラを作成する。
  pub fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }

  /// プレイヤーの位置に基づいてカメラの位置を更新する。
  /// カメラビューがプレイヤーを中央に捉えようとするけど、
  /// ワールドの境界を越えないように賢く調整してくれるよ。
  pub fn update(&mut self, player: &Player) {
    let target_x = player.x - (VIEW_WIDTH as f32 / 2.0);
    let target_y = player.y - (VIEW_HEIGHT as f32 / 2.0);

    // `clamp` メソッドで、カメラがワールドの端からはみ出さないようにするよ！
    self.x = target_x.clamp(0.0, (WORLD_WIDTH - VIEW_WIDTH) as f32);
    self.y = target_y.clamp(0.0, (WORLD_HEIGHT - VIEW_HEIGHT) as f32);
  }
}