// src/ui.rs

use std::sync::Arc;
use winit::window::Window;

/// ウィンドウのタイトルバーに現在のシード値とFPSを表示する。
///
/// # Arguments
/// * `window` - タイトルを設定する対象のウィンドウ (`Option<&Arc<Window>>`)。
/// * `seed` - 表示するシード値 (`u64`)。
pub fn update_window_title(window_opt: Option<&Arc<Window>>, seed: u64) {
  if let Some(window) = window_opt {
    window.set_title(&format!("terraspiel | Seed: {}", seed));
  }
}