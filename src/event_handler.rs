use crate::app::App;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::event::WindowEvent;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 初回実行時やアプリ再開時にウィンドウとピクセルを初期化
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("Dot Drawer with Gravity")
                .with_inner_size(winit::dpi::LogicalSize::new(crate::app::WIDTH, crate::app::HEIGHT));

            let window = event_loop.create_window(window_attributes)
                .expect("Failed to create window");

            let window_size = window.inner_size();
            let surface_texture = pixels::SurfaceTexture::new(window_size.width, window_size.height, &window);
            let pixels = pixels::Pixels::new(crate::app::WIDTH, crate::app::HEIGHT, surface_texture)
                .expect("Failed to create pixels");

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
        
        // アプリが再開されたときに時間差分が大きくなるのを防ぐため、last_timeを現在時刻にリセット
        self.last_time = std::time::Instant::now();
        self.last_dot_add_time = std::time::Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref window) = self.window {
            if window.id() != window_id {
                return;
            }
        } else {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                // マウス位置を保存
                self.mouse_position = Some((position.x, position.y));
            }
            WindowEvent::MouseInput {
                state,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        // 左クリック押下
                        println!("Left mouse pressed"); // デバッグ出力
                        self.left_mouse_pressed = true;
                        if let Some((x, y)) = self.mouse_position {
                            println!("Adding dot at ({}, {})", x as i32, y as i32); // デバッグ出力
                            self.add_dot_if_not_exists(x as i32, y as i32);
                            println!("Number of dots after add: {}", self.dots.len()); // デバッグ出力
                            // 再描画をリクエスト
                            if let Some(ref window) = self.window {
                                window.request_redraw();
                            }
                        } else {
                            println!("Mouse position is unknown - move the mouse first"); // デバッグ出力
                        }
                    }
                    winit::event::ElementState::Released => {
                        // 左クリック解放
                        println!("Left mouse released"); // デバッグ出力
                        self.left_mouse_pressed = false;
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // 左クリック押しっぱなしで、かつ指定時間経過している場合にドット追加
                if self.left_mouse_pressed {
                    if let Some((x, y)) = self.mouse_position {
                        if std::time::Instant::now().duration_since(self.last_dot_add_time) >= self.dot_add_interval {
                            println!("Adding dot due to hold at ({}, {})", x as i32, y as i32); // デバッグ出力
                            self.add_dot_if_not_exists(x as i32, y as i32);
                        }
                    }
                }
                
                // 物理更新
                self.update_physics();
                
                // ドット描画
                self.draw_dots();
                
                // 画面に反映
                if let Some(ref mut pixels) = self.pixels {
                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }
                
                // 更新中であれば、すぐに次の再描画をリクエスト
                if self.is_updating {
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}