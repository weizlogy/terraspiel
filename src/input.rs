//! src/input.rs

use winit::event::{ElementState, WindowEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::collections::HashSet;
use crate::core::player::PlayerAction;

/// ユーザーの操作によって発生する、アプリケーション全体に関わるアクション。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
  None,
  RegenerateWorld,
  ExitApp,
}

/// ウィンドウイベントを解釈し、対応する UserAction を返す。
/// また、押下されたキーの状態を更新する。
///
/// # Arguments
/// * `event` - winit から渡される `WindowEvent`。
/// * `pressed_keys` - 現在押されているキーのセットを保持する `HashSet` への可変参照。
/// * `pressed_mouse_buttons` - 現在押されているマウスボタンのセットを保持する `HashSet` への可変参照。
///
/// # Returns
/// 解析された `UserAction`。
pub fn handle_window_event(event: &WindowEvent, pressed_keys: &mut HashSet<KeyCode>, pressed_mouse_buttons: &mut HashSet<MouseButton>) -> UserAction {
  match event {
    WindowEvent::CloseRequested => UserAction::ExitApp,

    // マウスのクリックイベントを処理するよ
    WindowEvent::MouseInput { state, button, .. } => {
      match state {
        ElementState::Pressed => {
          pressed_mouse_buttons.insert(*button);
        }
        ElementState::Released => {
          pressed_mouse_buttons.remove(button);
        }
      }
      return UserAction::None;
    }
    // キーリピート（押しっぱなし）は無視して、押した瞬間と離した瞬間だけを捉えるよ
    WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
      if let PhysicalKey::Code(key_code) = event.physical_key {
        match event.state {
          ElementState::Pressed => {
            // Rキーはワールド再生成っていう特別なアクションだから、ここで直接返すね
            if key_code == KeyCode::KeyR {
              println!("'R' key pressed. Requesting world regeneration...");
              return UserAction::RegenerateWorld;
            }
            // 他のキーは「押されてるキーリスト」に追加するだけ
            pressed_keys.insert(key_code);
          }
          ElementState::Released => {
            // キーが離されたら「押されてるキーリスト」から削除
            pressed_keys.remove(&key_code);
          }
        }
      }
      UserAction::None
    }
    _ => UserAction::None,
  }
}

/// 現在押されているキーのセットから、プレイヤーのアクションリストを生成する。
pub fn get_player_actions(pressed_keys: &HashSet<KeyCode>, pressed_mouse_buttons: &HashSet<MouseButton>) -> Vec<PlayerAction> {
  let mut actions = Vec::new();
  if pressed_keys.contains(&KeyCode::KeyA) { actions.push(PlayerAction::MoveLeft); }
  if pressed_keys.contains(&KeyCode::KeyD) { actions.push(PlayerAction::MoveRight); }
  if pressed_keys.contains(&KeyCode::Space) { actions.push(PlayerAction::Jump); }
  if pressed_keys.contains(&KeyCode::KeyZ) { actions.push(PlayerAction::SelectNextTerrain); }
  if pressed_keys.contains(&KeyCode::KeyX) { actions.push(PlayerAction::SelectNextOverlay); }
  if pressed_mouse_buttons.contains(&MouseButton::Left) { actions.push(PlayerAction::BreakBlock); }
  if pressed_mouse_buttons.contains(&MouseButton::Right) { actions.push(PlayerAction::PlaceBlock); }
  actions
}