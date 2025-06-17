// 世界の構成物

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
      Terrain::Empty => [0, 0, 0, 255],
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
      Overlay::Water => [0, 0, 255, 150],
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
}
