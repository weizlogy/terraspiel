use std::error::Error;
use winit::event_loop::EventLoop;

mod app;
mod render;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let app = app::App::new(&event_loop)?;
    app.run(event_loop)
}