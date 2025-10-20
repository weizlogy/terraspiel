use crate::app::App;

impl App {
    // ドット描画
    pub fn draw_dots(&mut self) {
        println!("draw_dots called, number of dots: {}", self.dots.len()); // デバッグ出力
        if let Some(ref mut pixels) = self.pixels {
            // フレームをクリア（黒）
            let frame = pixels.frame_mut();
            for pixel in frame.chunks_exact_mut(4) {
                pixel[0] = 0; // R
                pixel[1] = 0; // G
                pixel[2] = 0; // B
                pixel[3] = 255; // A
            }

            // すべてのドットを描画
            for dot in &self.dots {
                println!("Drawing dot at ({}, {})", dot.x, dot.y); // デバッグ出力
                dot.material.draw_dot(
                    frame,
                    dot.x as i32,
                    dot.y as i32,
                    crate::app::WIDTH,
                    crate::app::HEIGHT,
                );
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
                if dot.y >= (crate::app::HEIGHT as f64 - 2.0) {
                    // ドットの半径分余裕を持たせる
                    dot.y = crate::app::HEIGHT as f64 - 2.0;
                    dot.vy = -dot.vy * self.bounce_factor; // 反発
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
