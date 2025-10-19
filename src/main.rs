mod app;
use app::App;

use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(&mut App::new())?;
    Ok(())
}