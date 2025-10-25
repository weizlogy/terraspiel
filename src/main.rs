mod app;
mod material;

use app::App;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();

    event_loop.run(move |event, event_loop| {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        match event {
            Event::Resumed => {
                app.handle_resume(event_loop);
            }
            Event::WindowEvent { event, window_id } => {
                if let Some(ref window) = app.window {
                    if window.id() == window_id {
                        match event {
                            WindowEvent::CloseRequested => {
                                event_loop.exit();
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                app.handle_cursor_moved(position);
                            }
                            WindowEvent::MouseInput { state, button, .. } => {
                                app.handle_mouse_input(state, button);
                            }
                            WindowEvent::RedrawRequested => {
                                app.handle_redraw_requested();
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        // アニメーションや更新が必要な場合は、再描画をリクエスト
        if app.is_updating {
            if let Some(ref window) = app.window {
                window.request_redraw();
            }
        }
    }).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
