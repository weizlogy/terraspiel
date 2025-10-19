mod app;

use winit::event_loop::EventLoop;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(&mut App::new())?;
    Ok(())
}
