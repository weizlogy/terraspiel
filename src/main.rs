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

use crate::core::engine;
use crate::core::material::{Terrain, Overlay}; // Overlay をインポート
use crate::core::world::{World, Tile, HEIGHT, WIDTH}; // Tile をインポート

use std::sync::Arc;

#[derive(Default)]
struct App {
  window: Option<Arc<Window>>,
  pixels: Option<Pixels<'static>>,
  world: Option<Box<World>>, // World はヒープに置くのが安全だよ！
  coords: Vec<(usize, usize)>,
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

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    engine::update_world(&mut self.world.as_mut().unwrap(), &mut self.coords); // 💥重力を適用！
    self.window.as_ref().unwrap().request_redraw();
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

  app.coords = generate_coords();

  // World インスタンスを Box で包んでヒープに確保！
  app.world = Some(Box::new(World::new()));

  // ★テストコード
  // 📦 地形の一部をDirtにする（仮）
  // この処理は毎フレーム実行されるから、ワールドの初期化は init 関数とかでした方がいいかもね！
  for x in WIDTH / 4..WIDTH / 2 {
    for y in 100..200 {
      app.world.as_mut().unwrap().set_terrain(x, y, Terrain::Dirt);
    }
  }

  for x in WIDTH / 4..WIDTH / 2 {
    for y in 0..100 {
      app.world.as_mut().unwrap().set_terrain(x, y, Terrain::Sand);
    }
  }

  // 💧 水 (Overlay) もちょっと置いてみよう！ 岩盤の上に水を配置
  for x in WIDTH / 2..WIDTH * 3 / 4 {
    // 水を置く範囲の底に岩盤を敷いておく
    app.world.as_mut().unwrap().set_terrain(x, 150, Terrain::Rock);
    // その一段上に水を配置
    app.world.as_mut().unwrap().set_overlay(x, 0, Overlay::Water);
    app.world.as_mut().unwrap().set_overlay(x, 0, Overlay::Water); // さらにもう一段水
  }
}

fn generate_coords() -> Vec<(usize, usize)> {
  let mut coords = Vec::with_capacity(WIDTH * HEIGHT);
  for y in (0..HEIGHT).rev() {
    for x in 0..WIDTH {
      coords.push((x, y));
    }
  }
  coords
}

fn draw_world(world: &mut World, frame: &mut [u8]) {
  // ワールドの各タイルを描画バッファに書き込むよ！
  render::render::draw_world(world, frame);
}
