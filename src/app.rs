use crate::material::BaseMaterialParams;
use winit::window::Window;
use wgpu::Surface;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use winit::window::WindowBuilder;

// ドットの状態を保持する構造体
pub struct Dot {
    pub x: f64,
    pub y: f64,
    pub vx: f64,  // x方向速度
    pub vy: f64,  // y方向速度
    pub material: BaseMaterialParams,
}

impl Dot {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,  // 初期速度は0
            vy: 0.0,
            material: BaseMaterialParams::default(),
        }
    }
}

// App構造体
pub struct App {
    pub window: Option<Arc<Window>>,
    pub surface: Option<Arc<Surface<'static>>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub mouse_position: Option<(f64, f64)>,
    pub dots: Vec<Dot>,        // ドットリスト
    pub gravity: f64,          // 重力加速度
    pub last_time: std::time::Instant,  // 時間管理用
    pub bounce_factor: f64,    // 反発係数
    pub is_updating: bool,     // 物理更新中かどうかのフラグ
    pub left_mouse_pressed: bool, // 左クリックが押されているか
    pub last_dot_add_time: std::time::Instant, // 最後にドットを追加した時刻
    pub dot_add_interval: std::time::Duration, // ドット追加の間隔
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            mouse_position: None,
            dots: Vec::new(),
            gravity: 9.8 * 10.0,  // 重力加速度（画面ピクセル基準にスケーリング）
            last_time: std::time::Instant::now(),
            bounce_factor: 0.7,   // 反発係数
            is_updating: false,   // 更新中フラグの初期値
            left_mouse_pressed: false, // 左クリック押下状態の初期値
            last_dot_add_time: std::time::Instant::now(), // ドット追加時刻の初期値
            dot_add_interval: std::time::Duration::from_millis(100), // 100msごとにドット追加
        }
    }

    // ドット追加（簡略化版 - 常にドットを追加）
    pub fn add_dot_if_not_exists(&mut self, x: i32, y: i32) {
        // 位置の重複チェックを一旦外す
        self.dots.push(Dot::new(x as f64, y as f64));
        self.is_updating = true; // 物理更新を開始
        self.last_time = std::time::Instant::now(); // 物理更新の基準時刻をリセット
        self.last_dot_add_time = std::time::Instant::now(); // 最後に追加した時刻を更新
    }

    // ウィンドウの再開時に呼び出される
    pub fn handle_resume(&mut self, event_loop: &winit::event_loop::EventLoopWindowTarget<()>) {
        if self.window.is_none() {
            let window = Arc::new(
                WindowBuilder::new()
                    .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT))
                    .build(event_loop)
                    .expect("Failed to create window")
            );
            self.window = Some(window.clone());

            // wgpuの初期化
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: wgpu::InstanceFlags::empty(),
                dx12_shader_compiler: Default::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            });

            let surface = instance.create_surface(window.as_ref()).expect("Failed to create surface");
            let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })).expect("Failed to find an appropriate adapter");

            let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            }, None)).expect("Failed to create device");

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps.formats.iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);
            
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: WIDTH,
                height: HEIGHT,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &config);


            let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
            self.surface = Some(Arc::new(surface));
            self.device = Some(device);
            self.queue = Some(queue);
            self.config = Some(config);
        }
        
        // アプリが再開されたときに時間差分が大きくなるのを防ぐため、last_timeを現在時刻にリセット
        self.last_time = std::time::Instant::now();
        self.last_dot_add_time = std::time::Instant::now();
    }

    // カーソル移動時に呼び出される
    pub fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = Some((position.x, position.y));
    }

    // マウス入力時に呼び出される
    pub fn handle_mouse_input(&mut self, state: winit::event::ElementState, button: winit::event::MouseButton) {
        match button {
            winit::event::MouseButton::Left => {
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
            _ => {}
        }
    }

    // ドットのインスタンスデータ（中心座標、色）を生成する
    pub fn create_dot_instance_data(&self) -> (Vec<f32>, wgpu::VertexBufferLayout<'static>) {
        let mut instance_data: Vec<f32> = Vec::new();
        for dot in &self.dots {
            let (r, g, b) = dot.material.get_color_rgb();
            let color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
            
            // 各ドットの中心座標と色をインスタンスデータとして追加
            instance_data.push(dot.x as f32);
            instance_data.push(dot.y as f32);
            instance_data.extend_from_slice(&color);
        }

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: (2 + 3) as wgpu::BufferAddress * std::mem::size_of::<f32>() as wgpu::BufferAddress, // position(x,y) + color(r,g,b)
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1, // position (was 0)
                    format: wgpu::VertexFormat::Float32x2, // position
                },
                wgpu::VertexAttribute {
                    offset: (2 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    shader_location: 2, // color (was 1)
                    format: wgpu::VertexFormat::Float32x3, // color
                },
            ],
        };

        (instance_data, instance_layout)
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

    // 再描画要求時に呼び出される
    pub fn handle_redraw_requested(&mut self) {
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
        if let (Some(ref surface), Some(ref device), Some(ref queue), Some(ref config)) = (&self.surface, &self.device, &self.queue, &self.config) {
            let frame: wgpu::SurfaceTexture = surface.get_current_texture().expect("Failed to acquire next swap chain texture");
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
            
            // ドットの描画 (シェーダーを使用)
            // インスタンスデータを準備
            let (instance_data, instance_layout) = self.create_dot_instance_data();
            if !instance_data.is_empty() {
                // シェーダーモジュールを読み込む
                let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Dot shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/dot.wgsl").into()),
                });

                // レンダーパイプラインを作成 (インスタンシング用)
                let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Dot render pipeline"),
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress, // 頂点の基本形状 (x, y) - 4ピクセルの正方形
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 0,
                                    format: wgpu::VertexFormat::Float32x2, // vertex offset
                                },
                            ],
                        }, instance_layout],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleStrip,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

                // 基本形状の頂点バッファ (4ピクセルの正方形)
                let square_vertex_data: [f32; 8] = [
                    -2.0, -2.0,  // 左下
                    2.0, -2.0,   // 右下
                    -2.0, 2.0,   // 左上
                    2.0, 2.0,    // 右上
                ];
                let square_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Square Vertex Buffer"),
                    contents: bytemuck::cast_slice(&square_vertex_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                // インスタンスデータのバッファ
                let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // 描画コマンドを発行
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_vertex_buffer(0, square_vertex_buffer.slice(..)); // 基本形状の頂点
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..)); // インスタンスデータ（位置と色）
                    render_pass.draw(0..4, 0..instance_data.len() as u32 / 5); // 4頂点で1つの正方形、インスタンス数は (instance_data.len() / 5)
                }
            } else {
                // ドットがない場合は、空のレンダーパスを作成
                {
                    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }
            }
            
            queue.submit(std::iter::once(encoder.finish()));
            frame.present();
        }
    }
}