// 世界の構成物

use crate::core::world::World; // World への参照が必要になるよ

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
  Empty,
  Dirt,
  Rock,
  Sand,
  Snow,
  Coal,
  Copper,
  Iron,
  Gold,
}

impl Terrain {
  pub fn color(&self) -> [u8; 4] {
    match self {
      Terrain::Empty => [20, 20, 30, 255], // 真っ黒から少し明るい色に
      Terrain::Dirt => [101, 67, 33, 255],
      Terrain::Rock => [100, 100, 100, 255],
      Terrain::Sand => [194, 178, 128, 255],
      Terrain::Snow => [240, 240, 255, 255],
      Terrain::Coal => [30, 30, 30, 255],
      Terrain::Copper => [184, 115, 51, 255],
      Terrain::Iron => [180, 180, 200, 255],
      Terrain::Gold => [255, 215, 0, 255],
    }
  }

  pub fn is_solid(&self) -> bool {
    !matches!(self, Terrain::Empty)
  }

  /// このテレインが指定された位置で横滑りを試みるべきかどうかを判断する。
  ///
  /// # Arguments
  /// * `world` - 現在のワールドの状態。テレインの周囲の状況を確認するために使用する。
  /// * `x` - テレインのX座標。
  /// * `y` - テレインのY座標。
  ///
  /// # Returns
  /// 横滑りを試みるべきなら `true`、そうでないなら `false`。
  pub fn should_attempt_slide(&self, world: &World, x: usize, y: usize) -> bool {
    match self {
      Terrain::Sand => true, // Sand は下に支えがあれば常に滑ろうとする
      Terrain::Dirt => world.count_stack_height(x, y) >= 1, // Dirt は上に圧力がかかると滑ろうとする
      _ => false, // 他のテレインはデフォルトでは滑らない
    }
  }

  /// この地形が流体 (水など) の通過を許容するかどうか。
  /// `true` の場合、この地形の上に流体があり、その流体の下にこの地形がある場合、
  /// 流体はこの地形のマスに流れ込む (染み込む) ことができる。
  pub fn allows_fluid_passthrough(&self) -> bool {
    match self {
      Terrain::Rock => false, // 岩は流体を通さない
      Terrain::Dirt | Terrain::Sand | Terrain::Empty => true, // 土、砂、空は流体を通す
      _ => false, // デフォルトでは通さない
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overlay {
  None,
  Grass,
  Tree,
  Stone,
  Ice,
  Water,
  Lava,
  Snow,
  Air,
  FlammableGas,
}

impl Overlay {
  pub fn color(&self) -> [u8; 4] {
    match self {
      Overlay::None => [0, 0, 0, 0],
      Overlay::Grass => [34, 139, 34, 255],
      Overlay::Tree => [50, 30, 10, 255],
      Overlay::Stone => [120, 120, 120, 255],
      Overlay::Ice => [180, 220, 255, 255],
      Overlay::Water => [64, 164, 223, 180], // 少し明るく、透明度も調整
      Overlay::Lava => [255, 69, 0, 200],
      Overlay::Snow => [255, 255, 255, 200],
      Overlay::Air => [0, 0, 0, 0],
      Overlay::FlammableGas => [255, 255, 100, 100],
    }
  }

  pub fn is_liquid(&self) -> bool {
    matches!(self, Overlay::Water | Overlay::Lava | Overlay::Snow)
  }

  pub fn is_gas(&self) -> bool {
    matches!(self, Overlay::Air | Overlay::FlammableGas)
  }

  pub fn is_solid(&self) -> bool {
    matches!(self, Overlay::Grass | Overlay::Tree | Overlay::Stone | Overlay::Ice)
  }

  /// このオーバーレイが流体のように振る舞うか (落下や横滑りの対象になるか)
  pub fn can_flow(&self) -> bool {
    matches!(self, Overlay::Water | Overlay::Lava) // とりあえず水と溶岩を流体にしてみよう
  }

  /// このオーバーレイが他の流体によって置き換え可能か (例: 空気は水に置き換わる)
  pub fn is_replaceable_by_fluid(&self) -> bool {
    matches!(self, Overlay::Air | Overlay::None) // 空気や何もない空間は置き換え可能
  }

  /// このオーバーレイが指定された位置で横滑りを試みるべきかどうかを判断する。
  ///
  /// # Arguments
  /// * `world` - 現在のワールドの状態。
  /// * `x` - オーバーレイのX座標。
  /// * `y` - オーバーレイのY座標。
  ///
  /// # Returns
  /// 横滑りを試みるべきなら `true`、そうでないなら `false`。
  pub fn should_attempt_slide(&self, world: &World, x: usize, y: usize) -> bool {
    match self {
      Overlay::Water | Overlay::Lava => true, // 水や溶岩は常に滑ろうとする
      _ => false,
    }
  }
}
