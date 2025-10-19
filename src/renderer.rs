use crate::app::{App, Dot};

impl App {
    // ドット描画
    pub fn draw_dots(&mut self) {
        println!("draw_dots called, number of dots: {}", self.dots.len()); // デバッグ出力
        if let Some(ref mut pixels) = self.pixels {
            // フレームをクリア（黒）
            let frame = pixels.frame_mut();
            for pixel in frame.chunks_exact_mut(4) {
                pixel[0] = 0;  // R
                pixel[1] = 0;  // G
                pixel[2] = 0;  // B
                pixel[3] = 255; // A
            }

            // すべてのドットを描画
            for dot in &self.dots {
                println!("Drawing dot at ({}, {})", dot.x, dot.y); // デバッグ出力
                // 4x4ドットの範囲を計算
                let x = dot.x as i32;
                let y = dot.y as i32;
                let start_x = (x - 2).max(0).min(crate::app::WIDTH as i32 - 1);
                let end_x = (x + 1).max(0).min(crate::app::WIDTH as i32 - 1);
                let start_y = (y - 2).max(0).min(crate::app::HEIGHT as i32 - 1);
                let end_y = (y + 1).max(0).min(crate::app::HEIGHT as i32 - 1);

                println!("Drawing range: ({}, {}) to ({}, {})", start_x, start_y, end_x, end_y); // デバッグ出力

                for py in start_y..=end_y {
                    for px in start_x..=end_x {
                        let pixel_index = (py as usize * crate::app::WIDTH as usize + px as usize) * 4;
                        // RGBA: 白色 (255, 255, 255, 255)
                        frame[pixel_index] = 255;       // R
                        frame[pixel_index + 1] = 255;   // G
                        frame[pixel_index + 2] = 255;   // B
                        frame[pixel_index + 3] = 255;   // A
                    }
                }
            }
        }
    }

    // 物理更新
    pub fn update_physics(&mut self) {
        // 更新が必要な場合のみ物理計算を行う
        if self.is_updating {
            let now = std::time::Instant::now();
            let dt = now.duration_since(self.last_time).as_secs_f64();
            self.last_time = now;

            for dot in &mut self.dots {
                // 重力を適用
                dot.vy += self.gravity * dt;

                // 位置を更新
                dot.x += dot.vx * dt;
                dot.y += dot.vy * dt;

                // 境界衝突処理（画面下端）
                if dot.y >= (crate::app::HEIGHT as f64 - 2.0) {  // ドットの半径分余裕を持たせる
                    dot.y = crate::app::HEIGHT as f64 - 2.0;
                    dot.vy = -dot.vy * self.bounce_factor;  // 反発
                }

                // 境界衝突処理（画面上端）
                if dot.y <= 1.0 {
                    dot.y = 1.0;
                    dot.vy = -dot.vy * self.bounce_factor;
                }

                // 境界衝突処理（左右端）
                if dot.x >= crate::app::WIDTH as f64 - 2.0 {
                    dot.x = crate::app::WIDTH as f64 - 2.0;
                    dot.vx = -dot.vx * self.bounce_factor;
                }
                if dot.x <= 1.0 {
                    dot.x = 1.0;
                    dot.vx = -dot.vx * self.bounce_factor;
                }
            }

            // 更新を停止する条件を確認
            let all_stopped = self.dots.iter().all(|dot| {
                // 速度が非常に小さいか、画面下端にあり跳ね返りが非常に小さいかを確認
                let velocity_small = dot.vy.abs() < 0.1 && dot.vx.abs() < 0.1;
                let at_bottom = dot.y >= crate::app::HEIGHT as f64 - 3.0; // 少し余裕を持たせる
                let slow_bounce = velocity_small && at_bottom;
                
                slow_bounce
            });

            // すべてのドットが停止状態に達した場合、更新を停止
            if all_stopped && !self.dots.is_empty() {
                self.is_updating = false;
                println!("Physics update stopped - all dots have stopped"); // デバッグ出力
            }
        }
    }
}