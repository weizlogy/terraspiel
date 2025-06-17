mod core;
mod render;

use pixels::Error;
use pixels::Pixels;
use pixels::SurfaceTexture;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window;
use winit::window::{Window, WindowId};

use crate::core::material::Terrain;
use crate::core::world::{World, Tile, HEIGHT, WIDTH}; // Tile をインポート

use std::sync::Arc;

#[derive(Default)]
struct App {
  window: Option<Arc<Window>>,
  pixels: Option<Pixels<'static>>,
  world: Option<Box<World>>, // World はヒープに置くのが安全だよ！
}

impl ApplicationHandler for App {
  fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
  }

  fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
    match cause {
      winit::event::StartCause::Init => {
        self.window = Some(Arc::new(event_loop.create_window(Window::default_attributes()).unwrap()));
        init(self);
      },
      _ => (),
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      },
      WindowEvent::RedrawRequested => {
        // pixels と world がちゃんと準備できてるか確認してから描画しようね！
        if let (Some(pixels), Some(world_box)) = (self.pixels.as_mut(), self.world.as_mut()) {
          let frame = pixels.frame_mut();
          draw_world(world_box.as_mut(), frame); // Box から &mut World を取り出して渡すよ

          // 描画結果を画面に反映！
          if let Err(err) = pixels.render() {
            eprintln!("pixels.render() でエラー発生！: {}", err);
            event_loop.exit(); // エラーが出たら、残念だけど終了…
          }
        } else {
          // まだ準備できてなかったら、もう一回描画リクエストしとこっか
          if let Some(window) = self.window.as_ref() { window.request_redraw(); }
        }
      }
      _ => (),
    }
  }
}

fn main() -> Result<(), Error> {
  // EventLoopはOSからのいろんなイベント(マウス操作、キー入力とか)を受け取る係だよ！
  let event_loop = EventLoop::new().expect("イベントループの作成に失敗しちゃった…");
  // アプリケーションの状態を管理する App インスタンス
  let mut app = {
    App::default()
  };

  // これがゲームの心臓部、イベントループだよ！
  let _ = event_loop.run_app(&mut app);

  // run()は戻ってこないけど、型エラー防止のためにOk(())を返すよ！
  Ok(())
}

fn init(app: &mut App) {
  // Arcで包んだwindowを使うことで、ライフタイム問題を華麗に回避！(๑•̀ㅂ•́)و✧
  let window = app.window.as_ref().unwrap();
  window.set_title("terraspiel");

  app.pixels = {
    let size = window.inner_size();
    // SurfaceTexture::newはArc<Window>も受け取れるから、cloneでOK！
    let surface_texture =
      SurfaceTexture::new(size.width, size.height, window.clone());
    Some(Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap())
  };

  // World インスタンスを Box で包んでヒープに確保！
  app.world = Some(Box::new(World::new()));
}

fn draw_world(world: &mut World, frame: &mut [u8]) {
  // 📦 地形の一部をDirtにする（仮）
  // この処理は毎フレーム実行されるから、ワールドの初期化は init 関数とかでした方がいいかもね！
  for x in 0..WIDTH {
    for y in (HEIGHT - 10)..HEIGHT {
      world.set_terrain(x, y, Terrain::Dirt);
    }
  }

  // ワールドの各タイルを描画バッファに書き込むよ！
  render::render::draw_world(world, frame);
}
