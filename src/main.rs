mod core;
mod render;
mod ui; // ✨ 新しいUIモジュールを宣言！
mod input; // ✨ 新しい入力モジュールを宣言！

use pixels::Error;
use pixels::Pixels;
use pixels::SurfaceTexture;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::core::engine;
use crate::core::generation; // 地形生成モジュールをインポート！
use crate::core::seed_generator; // ✨新しいシードジェネレーターをインポート！
use crate::core::world::{World, HEIGHT, WIDTH}; // Tile をインポート
use rand::rngs::StdRng; // 乱数生成器
use rand::{SeedableRng}; // Rng トレイトと SeedableRng を use

use std::{sync::Arc, time::Instant}; // Instant を使うために追加！
use crate::input::UserAction; // inputモジュールからUserActionをインポート

#[derive(Default)]
struct App {
  seed_value: u64, // 生成されたシード値を保持するフィールド
  fps: f64,        // 計算されたFPSを保持するフィールド
  window: Option<Arc<Window>>,
  pixels: Option<Pixels<'static>>,
  world: Option<Box<World>>, // World はヒープに置くのが安全だよ！
  rng: Option<StdRng>,      // 草の成長などに使う乱数生成器
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
    // --- 入力処理 ---
    // inputモジュールにイベント処理をお願いするよ！
    match input::handle_window_event(&event) {
      UserAction::ExitApp => {
        println!("Exit action received. Closing application.");
        event_loop.exit();
        return; // イベントループを抜けるので、これ以上の処理は不要
      }
      UserAction::RegenerateWorld => {
        init(self); // ワールドを再生成！
      }
      UserAction::None => {
        // 他のウィンドウイベント（RedrawRequestedなど）の処理を続ける
      }
    }

    // UserActionで処理されなかったイベントのみ、ここで処理する
    match event {
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

    // --- 草を成長させる処理 ---
    // world と rng の両方が利用可能な場合のみ実行するよ
    if let (Some(world_box), Some(rng_instance)) = (self.world.as_mut(), self.rng.as_mut()) {
      world_box.grow_grass(rng_instance); // 🌱
    }

    // --- UI更新 ---
    // uiモジュールにウィンドウタイトルの更新をお願い！
    ui::update_window_title(self.window.as_ref(), self.seed_value);

    if let Some(window) = self.window.as_ref() { window.request_redraw(); }
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
  // 初期タイトルはシンプルに。FPSなどは about_to_wait で更新されるよ。
  // window.set_title("terraspiel"); // ui::update_window_title が担当するので不要
  
  // Pixels インスタンスがまだなければ作成する
  if app.pixels.is_none() {
    app.pixels = {
      let size = window.inner_size();
      let surface_texture =
        SurfaceTexture::new(size.width, size.height, window.clone());
      Some(Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap())
    };
  } else {
    // 既存の Pixels インスタンスがあれば、バッファサイズをリセットする
    // (ワールドが再生成されるので、描画バッファもクリアしたい)
    // 必要であれば、ウィンドウサイズ変更に合わせて surface もリサイズする
    // let size = window.inner_size();
    // app.pixels.as_mut().unwrap().resize_surface(size.width, size.height).unwrap();
    app.pixels.as_mut().unwrap().resize_buffer(WIDTH as u32, HEIGHT as u32).unwrap();
  }

  app.coords = generate_coords();

  // World インスタンスを Box で包んでヒープに確保！
  app.world = Some(Box::new(World::new()));

  // --- シード値の生成 ---
  let world_seed = seed_generator::generate_seed(); // 🌟ここで新しい関数を呼び出すよ！
  app.seed_value = world_seed; // 生成したシード値を App に保持
  println!("Generated World Seed: {}", app.seed_value); // 生成されたシードをログに出してみよう！

  // 乱数生成器を初期化
  // ワールド生成とは別のシードを使うことで、草の生え方などに異なるランダム性を与えられるよ！
  app.rng = Some(StdRng::seed_from_u64(world_seed + 1)); // ワールド生成とは別のシード

  // --- 地形生成 ---
  // シード値を指定してワールドを生成するよ！この数字を変えると地形も変わるんだ。
  generation::generate_initial_world(app.world.as_mut().unwrap(), world_seed);
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
