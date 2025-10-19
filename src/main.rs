mod app;
mod event_handler;
mod material;
mod renderer;

use app::App;

use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(&mut App::new())?;
    Ok(())
}
