//! プレイヤーに関するすべてのロジックを管理するモジュールだよ！

use crate::core::world::{World, WIDTH, HEIGHT};
use crate::core::material::{Terrain, Overlay};

/// プレイヤーが実行可能なアクションを定義する enum だよ。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlayerAction {
  MoveLeft,
  MoveRight,
  Jump,
  BreakBlock,
  PlaceBlock,
  SelectNextTerrain,
  SelectNextOverlay,
  None,
}

// --- プレイヤーの物理演算に関する定数 ---
const PLAYER_ACCEL_X: f32 = 0.5;
const PLAYER_MAX_SPEED_X: f32 = 4.0;
const PLAYER_JUMP_STRENGTH: f32 = 7.0;
const PLAYER_FRICTION: f32 = 0.8; // 摩擦係数
const GRAVITY: f32 = 0.3;
const MAX_FALL_SPEED: f32 = 8.0;

// プレイヤーの初期スポーン位置
pub const PLAYER_SPAWN_X: f32 = WIDTH as f32 / 2.0;
pub const PLAYER_SPAWN_Y: f32 = HEIGHT as f32 / 4.0; // 地表より少し上

/// プレイヤーの状態を保持する構造体。
#[derive(Debug)]
pub struct Player {
  pub x: f32,
  pub y: f32,
  pub vel_x: f32,
  pub vel_y: f32,
  pub width: f32,
  pub height: f32,
  pub on_ground: bool,
  pub selected_terrain: Terrain,
  pub selected_overlay: Overlay,
}

impl Player {
  /// 新しいプレイヤーインスタンスを作成するよ。
  pub fn new(x: f32, y: f32) -> Self {
    Player {
      x,
      y,
      vel_x: 0.0,
      vel_y: 0.0,
      width: 1.8, // 2ブロック弱の幅
      height: 2.8, // 3ブロック弱の高さ
      on_ground: false,
      selected_terrain: Terrain::Dirt,
      selected_overlay: Overlay::Air,
    }
  }
}

/// プレイヤーの状態を更新する関数。入力、物理、ワールドとのインタラクションを処理するよ。
pub fn update(player: &mut Player, world: &mut World, player_actions: &[PlayerAction], cursor_pos: (f32, f32)) {
  // 1. 水平方向の移動入力
  let mut desired_vel_x = 0.0;
  if player_actions.contains(&PlayerAction::MoveLeft) {
    desired_vel_x -= PLAYER_ACCEL_X;
  }
  if player_actions.contains(&PlayerAction::MoveRight) {
    desired_vel_x += PLAYER_ACCEL_X;
  }

  // 速度を更新
  player.vel_x += desired_vel_x;
  player.vel_x *= PLAYER_FRICTION; // 摩擦で減速

  // 最大速度でクランプ
  player.vel_x = player.vel_x.clamp(-PLAYER_MAX_SPEED_X, PLAYER_MAX_SPEED_X);

  // 2. 垂直方向の移動 (重力とジャンプ)
  player.vel_y += GRAVITY;
  player.vel_y = player.vel_y.min(MAX_FALL_SPEED); // 落下速度を制限

  if player_actions.contains(&PlayerAction::Jump) && player.on_ground {
    player.vel_y = -PLAYER_JUMP_STRENGTH; // 上向きに速度を与える
    player.on_ground = false; // 地面から離れた
  }

  // 3. 衝突判定と位置更新
  // --- X軸方向 ---
  let mut new_x = player.x + player.vel_x;
  let target_x_min = new_x.floor() as isize;
  let target_x_max = (new_x + player.width - 0.001).floor() as isize;
  let current_y_min = player.y.floor() as isize;
  let current_y_max = (player.y + player.height - 0.001).floor() as isize;

  for y_check in current_y_min..=current_y_max {
    if player.vel_x > 0.0 { // 右に移動中
      if world.is_solid_at(target_x_max, y_check) {
        new_x = target_x_max as f32 - player.width;
        player.vel_x = 0.0;
        break;
      }
    } else if player.vel_x < 0.0 { // 左に移動中
      if world.is_solid_at(target_x_min, y_check) {
        new_x = (target_x_min + 1) as f32;
        player.vel_x = 0.0;
        break;
      }
    }
  }
  player.x = new_x;

  // --- Y軸方向 ---
  let mut new_y = player.y + player.vel_y;
  let target_y_min = new_y.floor() as isize;
  let target_y_max = (new_y + player.height - 0.001).floor() as isize;
  let current_x_min = player.x.floor() as isize;
  let current_x_max = (player.x + player.width - 0.001).floor() as isize;

  player.on_ground = false; // まずは地面にいないと仮定

  for x_check in current_x_min..=current_x_max {
    if player.vel_y > 0.0 { // 下に移動中 (落下中)
      if world.is_solid_at(x_check, target_y_max) {
        new_y = target_y_max as f32 - player.height;
        player.vel_y = 0.0;
        player.on_ground = true; // 地面に衝突した！
        break;
      }
    } else if player.vel_y < 0.0 { // 上に移動中 (ジャンプ中)
      if world.is_solid_at(x_check, target_y_min) {
        new_y = (target_y_min + 1) as f32;
        player.vel_y = 0.0;
        break;
      }
    }
  }
  player.y = new_y;

  // 4. ブロックの破壊と設置
  // マウスカーソルの位置をターゲット座標とする
  let target_x_break_place = cursor_pos.0.floor() as usize;
  let target_y_break_place = cursor_pos.1.floor() as usize;

  // プレイヤーの中心からターゲットブロックの中心までの距離を計算する
  let player_center_x = player.x + player.width / 2.0;
  let player_center_y = player.y + player.height / 2.0;
  let target_center_x = target_x_break_place as f32 + 0.5;
  let target_center_y = target_y_break_place as f32 + 0.5;

  let dist_sq = (target_center_x - player_center_x).powi(2) + (target_center_y - player_center_y).powi(2);
  let max_reach_sq = 5.0f32.powi(2); // 5ブロックの範囲まで届くようにする

  // 届く範囲内であれば、アクションを実行する
  if dist_sq <= max_reach_sq {
    if player_actions.contains(&PlayerAction::BreakBlock) {
      if let Some(tile_to_break) = world.get(target_x_break_place, target_y_break_place) {
        if tile_to_break.is_solid() {
          world.set_terrain(target_x_break_place, target_y_break_place, Terrain::Empty);
          world.set_overlay(target_x_break_place, target_y_break_place, Overlay::Air);
        }
      }
    } else if player_actions.contains(&PlayerAction::PlaceBlock) {
      if let Some(tile_to_place) = world.get(target_x_break_place, target_y_break_place) {
        if !tile_to_place.is_solid() {
          world.set_terrain(target_x_break_place, target_y_break_place, player.selected_terrain);
          world.set_overlay(target_x_break_place, target_y_break_place, player.selected_overlay);
        }
      }
    }
  }

  // 5. 選択中のマテリアル変更
  if player_actions.contains(&PlayerAction::SelectNextTerrain) {
    player.selected_terrain = match player.selected_terrain {
      Terrain::Dirt => Terrain::Sand,
      Terrain::Sand => Terrain::Rock,
      Terrain::Rock => Terrain::Coal,
      Terrain::Coal => Terrain::Copper,
      Terrain::Copper => Terrain::Iron,
      Terrain::Iron => Terrain::Gold,
      Terrain::Gold => Terrain::Obsidian,
      Terrain::Obsidian => Terrain::HardIce,
      Terrain::HardIce => Terrain::Empty,
      Terrain::Empty => Terrain::Dirt,
    };
  }
  if player_actions.contains(&PlayerAction::SelectNextOverlay) {
    player.selected_overlay = match player.selected_overlay {
      Overlay::Air => Overlay::Water,
      Overlay::Water => Overlay::Lava,
      Overlay::Lava => Overlay::Snow,
      Overlay::Snow => Overlay::Grass,
      Overlay::Grass => Overlay::Tree,
      Overlay::Tree => Overlay::Stone,
      Overlay::Stone => Overlay::Ice,
      Overlay::Ice => Overlay::FlammableGas,
      Overlay::FlammableGas => Overlay::Air,
      Overlay::None => Overlay::Air,
    };
  }
}