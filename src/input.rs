// src/input.rs

use winit::event::WindowEvent;
use winit::keyboard::{KeyCode, PhysicalKey};

/// 取りうるユーザーのアクションを定義する。
#[derive(Debug, PartialEq, Eq)]
pub enum UserAction {
  None,
  RegenerateWorld,
  ExitApp,
}

/// ウィンドウイベントを解析し、対応するユーザーアクションを返す。
///
/// # Arguments
/// * `event` - winit から渡される `WindowEvent`。
///
/// # Returns
/// 解析された `UserAction`。
pub fn handle_window_event(event: &WindowEvent) -> UserAction {
  match event {
    WindowEvent::CloseRequested => UserAction::ExitApp,
    WindowEvent::KeyboardInput { event, .. } => {
      if event.state == winit::event::ElementState::Pressed && event.physical_key == PhysicalKey::Code(KeyCode::KeyR) {
        println!("'R' key pressed. Requesting world regeneration...");
        return UserAction::RegenerateWorld;
      }
      UserAction::None
    }
    _ => UserAction::None,
  }
}