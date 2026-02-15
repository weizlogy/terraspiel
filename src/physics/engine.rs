use crate::{
    app::{Dot, HEIGHT, WIDTH},
    material::{MaterialDNA, State},
};
use bytemuck::{Pod, Zeroable};
use rand::thread_rng;
use rand::Rng;
use std::sync::mpsc;
use std::time::Instant;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PhysicsParams {
    delta_time: f32,
    gravity: f32,
    width: f32,
    height: f32,
    dot_radius: f32,
    dots_count: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GpuDot {
    position: [f32; 2],
    velocity: [f32; 2],
    mass: f32,
    state: u32,
    temperature: f32,
    density: f32,
    viscosity: f32,
    elasticity: f32,
    cohesion: f32,
    entropy_bias: f32,
    luminescence: f32,
    heat_capacity_high: f32,
    heat_capacity_low: f32,
    heat_conductivity: f32,
    hardness: f32,
    volatility: f32,
    id: u32,
    reaction_count: u32,
    is_selected: u32,
    _padding: u32, // WGSLのアラインメントに合わせて4バイトのパディングを追加
}

impl GpuDot {
    fn from_cpu_dot(dot: &Dot) -> Self {
        let state_u32 = match dot.material.state {
            State::Solid => 0,
            State::Liquid => 1,
            State::Gas => 2,
        };
        
        GpuDot {
            position: [dot.x as f32, dot.y as f32],
            velocity: [dot.vx as f32, dot.vy as f32],
            mass: dot.material.density,
            state: state_u32,
            temperature: dot.material.temperature,
            density: dot.material.density,
            viscosity: dot.material.viscosity,
            elasticity: dot.material.elasticity,
            cohesion: dot.material.cohesion,
            entropy_bias: dot.material.entropy_bias,
            luminescence: dot.material.luminescence,
            heat_capacity_high: dot.material.heat_capacity_high,
            heat_capacity_low: dot.material.heat_capacity_low,
            heat_conductivity: dot.material.heat_conductivity,
            hardness: dot.material.hardness,
            volatility: dot.material.volatility,
            id: dot.id as u32,
            reaction_count: dot.reaction_count,
            is_selected: if dot.is_selected { 1 } else { 0 },
            _padding: 0, // パディングフィールドをゼロで初期化
        }
    }
}

const COOL_DOWN_SECONDS: f64 = 1.0; // 1秒のクールダウン

pub const DOT_RADIUS: f64 = 2.0;
const GAS_REFERENCE_DENSITY: f32 = 0.5;
const GAS_DIFFUSION_FACTOR: f64 = 5.0;
const INITIAL_WAIT_TIME: f64 = 0.1; // seconds
const DECAY_FACTOR: f64 = 0.5;

pub struct Physics {
    pub grid: Vec<Vec<usize>>,
    pub cols: usize,
    pub rows: usize,
    pub cell_size: f64,
    pub collision_tx: mpsc::Sender<((usize, MaterialDNA), (usize, MaterialDNA))>,
    pub compute_pipeline: Option<wgpu::ComputePipeline>,
    pub physics_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub physics_bind_group: Option<wgpu::BindGroup>,
    pub physics_params_buffer: Option<wgpu::Buffer>,
    pub dots_buffer: Option<wgpu::Buffer>,
}

impl Physics {
    pub fn new(collision_tx: mpsc::Sender<((usize, MaterialDNA), (usize, MaterialDNA))>) -> Self {
        let cell_size = DOT_RADIUS * 2.0;
        let cols = (WIDTH as f64 / cell_size).ceil() as usize;
        let rows = (HEIGHT as f64 / cell_size).ceil() as usize;
        let grid = vec![Vec::new(); cols * rows];

        Physics {
            grid,
            cols,
            rows,
            cell_size,
            collision_tx,
            compute_pipeline: None,
            physics_bind_group_layout: None,
            physics_bind_group: None,
            physics_params_buffer: None,
            dots_buffer: None,
        }
    }

    pub fn initialize_gpu_resources(&mut self, device: &wgpu::Device) {
        // Compute Pipelineの作成
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Physics Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/physics.wgsl").into()),
        });

        // バインドグループレイアウトの作成
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Physics Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Physics Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.compute_pipeline = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Physics Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader_module,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        }));

        // バインドグループレイアウトを保存
        self.physics_bind_group_layout = Some(bind_group_layout);
    }

    pub fn update_gpu_resources(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, dots: &[Dot], dt: f64) {
        // パラメータバッファの作成・更新
        let params = PhysicsParams {
            delta_time: dt as f32, // 実際の経過時間を使用
            gravity: 9.8 * 20.0,
            width: WIDTH as f32,
            height: HEIGHT as f32,
            dot_radius: DOT_RADIUS as f32,
            dots_count: dots.len() as u32,
        };

        if let Some(buffer) = &self.physics_params_buffer {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&params));
        } else {
            self.physics_params_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Physics Params Buffer"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }));
        }

        // ドットデータバッファの作成・更新
        let dots_data: Vec<GpuDot> = dots.iter().map(|dot| GpuDot::from_cpu_dot(dot)).collect();
        let dots_bytes = bytemuck::cast_slice(&dots_data);

        if let Some(buffer) = &self.dots_buffer {
            // 既存バッファーのサイズと新しいデータのサイズを比較
            if buffer.size() != dots_bytes.len() as u64 {
                // サイズが異なる場合は新しいバッファーを作成
                self.dots_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Dots Buffer"),
                    contents: dots_bytes,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                }));
            } else {
                // サイズが同じ場合は既存バッファーに書き込み
                queue.write_buffer(buffer, 0, dots_bytes);
            }
        } else {
            self.dots_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Dots Buffer"),
                contents: dots_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }));
        }

        // Bind Groupの作成
        if let (Some(params_buffer), Some(dots_buffer), Some(bind_group_layout)) = (&self.physics_params_buffer, &self.dots_buffer, &self.physics_bind_group_layout) {
            self.physics_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Physics Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: dots_buffer.as_entire_binding(),
                    },
                ],
            }));
        }
    }

    pub fn update_gpu_physics(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) {
        if let (Some(pipeline), Some(bind_group)) = (&self.compute_pipeline, &self.physics_bind_group) {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Physics Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups((self.dots_buffer.as_ref().unwrap().size() as u32 / std::mem::size_of::<GpuDot>() as u32 + 63) / 64, 1, 1);
        }
    }

    pub fn sync_gpu_to_cpu(&self, device: &wgpu::Device, queue: &wgpu::Queue, dots: &mut Vec<Dot>) {
        if let Some(buffer) = &self.dots_buffer {
            // GPUバッファからCPUにデータをコピー
            let size = buffer.size();
            let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Read Buffer"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Buffer Encoder"),
            });

            encoder.copy_buffer_to_buffer(buffer, 0, &read_buffer, 0, size);
            queue.submit(std::iter::once(encoder.finish()));

            // データを読み取る
            let buffer_slice = read_buffer.slice(..);
            
            // 非同期マッピングを同期的に待機
            let _ = buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::Maintain::Wait);

            let data = buffer_slice.get_mapped_range();
            let gpu_dots: &[GpuDot] = bytemuck::cast_slice(&data);
            
            // GPUデータをCPUデータに変換
            for (i, gpu_dot) in gpu_dots.iter().enumerate() {
                if i < dots.len() {
                    dots[i].x = gpu_dot.position[0] as f64;
                    dots[i].y = gpu_dot.position[1] as f64;
                    dots[i].vx = gpu_dot.velocity[0] as f64;
                    dots[i].vy = gpu_dot.velocity[1] as f64;
                    // 他のパラメータも必要に応じて更新
                }
            }
            
            drop(data);
            read_buffer.unmap();
        }
    }

    pub fn update_collision(&mut self, dots: &mut Vec<Dot>, dt: f64) -> bool {
        // 1. グリッドをクリア
        for cell in self.grid.iter_mut() {
            cell.clear();
        }

        // 2. ドットをグリッドに登録
        for (i, dot) in dots.iter().enumerate() {
            let cell_x = (dot.x / self.cell_size).floor() as usize;
            let cell_y = (dot.y / self.cell_size).floor() as usize;
            let cell_idx = cell_y * self.cols + cell_x;
            if cell_idx < self.grid.len() {
                self.grid[cell_idx].push(i);
            }
        }

        let mut potentially_colliding_pairs = Vec::new();

        // 3. 衝突候補ペアを収集
        for i in 0..dots.len() {
            let cell_x = (dots[i].x / self.cell_size).floor() as i32;
            let cell_y = (dots[i].y / self.cell_size).floor() as i32;

            for y_offset in -1..=1 {
                for x_offset in -1..=1 {
                    let check_x = cell_x + x_offset;
                    let check_y = cell_y + y_offset;

                    if check_x >= 0
                        && check_x < self.cols as i32
                        && check_y >= 0
                        && check_y < self.rows as i32
                    {
                        let cell_idx = (check_y as usize) * self.cols + (check_x as usize);
                        for &j in &self.grid[cell_idx] {
                            if i < j {
                                // ペアを一度だけ登録
                                potentially_colliding_pairs.push((i, j));
                            }
                        }
                    }
                }
            }
        }

        // 4. 衝突判定と処理
        for (i, j) in potentially_colliding_pairs {
            let (dot1_x, dot1_y, dot2_x, dot2_y) = {
                let dot1 = &dots[i];
                let dot2 = &dots[j];
                (dot1.x, dot1.y, dot2.x, dot2.y)
            };

            let dx = dot2_x - dot1_x;
            let dy = dot2_y - dot1_y;
            let distance_sq = dx * dx + dy * dy;
            let min_dist = DOT_RADIUS * 2.0;

            if distance_sq < min_dist * min_dist && distance_sq > 1e-6 {
                let now = Instant::now();

                // --- Reaction Logic ---
                let dot1 = &dots[i];
                let dot2 = &dots[j];
                let wait_time1 =
                    INITIAL_WAIT_TIME * (DECAY_FACTOR * dot1.reaction_count as f64).exp();
                let wait_time2 =
                    INITIAL_WAIT_TIME * (DECAY_FACTOR * dot2.reaction_count as f64).exp();
                let elapsed1 = now.duration_since(dot1.last_reaction_time).as_secs_f64();
                let elapsed2 = now.duration_since(dot2.last_reaction_time).as_secs_f64();

                if elapsed1 >= wait_time1 && elapsed2 >= wait_time2 {
                    // Send collision event for material blending
                    let _ = self.collision_tx.send((
                        (i, dots[i].material_dna.clone()),
                        (j, dots[j].material_dna.clone()),
                    ));

                    // Update reaction counters and timestamps
                    let (dot1_slice, dot2_slice) = dots.split_at_mut(j);
                    let dot1 = &mut dot1_slice[i];
                    let dot2 = &mut dot2_slice[0];
                    dot1.reaction_count += 1;
                    dot2.reaction_count += 1;
                    dot1.last_reaction_time = now;
                    dot2.last_reaction_time = now;
                }

                // --- Physical Collision Response (always happens) ---
                let (dot1_slice, dot2_slice) = dots.split_at_mut(j);
                let dot1 = &mut dot1_slice[i];
                let dot2 = &mut dot2_slice[0];

                let distance = distance_sq.sqrt();
                let overlap = 0.5 * (min_dist - distance);
                let nx = dx / distance;
                let ny = dy / distance;

                dot1.x -= overlap * nx;
                dot1.y -= overlap * ny;
                dot2.x += overlap * nx;
                dot2.y += overlap * ny;

                match (dot1.material.state, dot2.material.state) {
                    (State::Solid, State::Solid) | (State::Liquid, State::Liquid) => {
                        handle_detailed_collision(dot1, dot2, nx, ny, dt);

                        if dot1.material.state == State::Liquid
                            && dot2.material.state == State::Liquid
                        {
                            handle_liquid_accumulation(dot1, dot2, nx, ny, dt);
                        } else if dot1.material.state == State::Solid
                            && dot2.material.state == State::Solid
                        {
                            handle_solid_spreading(dot1, dot2, nx, ny, dt);
                        }
                    }
                    (State::Solid, State::Liquid) => {
                        if dot1.material.density > dot2.material.density
                            && dot1.material.viscosity > dot2.material.viscosity
                        {
                            let e =
                                (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;
                            let v_liquid_n = dot2.vx * nx + dot2.vy * ny;
                            if v_liquid_n < 0.0 {
                                dot2.vx -= (1.0 + e) * v_liquid_n * nx;
                                dot2.vy -= (1.0 + e) * v_liquid_n * ny;
                            }
                        } else {
                            handle_detailed_collision(dot1, dot2, nx, ny, dt);
                        }
                    }
                    (State::Liquid, State::Solid) => {
                        if dot2.material.density > dot1.material.density
                            && dot2.material.viscosity > dot1.material.viscosity
                        {
                            let e =
                                (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;
                            let v_liquid_n = dot1.vx * (-nx) + dot1.vy * (-ny);
                            if v_liquid_n < 0.0 {
                                dot1.vx -= (1.0 + e) * v_liquid_n * (-nx);
                                dot1.vy -= (1.0 + e) * v_liquid_n * (-ny);
                            }
                        } else {
                            handle_detailed_collision(dot1, dot2, nx, ny, dt);
                        }
                    }
                    (State::Gas, State::Gas) => {
                        handle_gas_collision(dot1, dot2, nx, ny);
                    }
                    (State::Solid, State::Gas) | (State::Liquid, State::Gas) => {
                        handle_gas_displacement(dot2, dot1, nx, ny);
                    }
                    (State::Gas, State::Solid) | (State::Gas, State::Liquid) => {
                        handle_gas_displacement(dot1, dot2, nx, ny);
                    }
                }
            }
        }
        true
    }
}

#[allow(dead_code)]
pub struct Explosion {
    x: f64,
    y: f64,
    radius: f64,
    force: f64,
    heat: f32,
}

pub fn update_state(dots: &mut Vec<Dot>, gravity: f64, dt: f64) {
    let mut rng = thread_rng();
    let mut explosions: Vec<Explosion> = Vec::new();
    let mut dots_to_remove: Vec<usize> = Vec::new();

    // 1. 状態変化と爆発の検出
    for (i, dot) in dots.iter_mut().enumerate() {
        if dots_to_remove.contains(&i) {
            continue;
        }

        // 発光中のGasの状態をチェック
        if let Some(since) = dot.glowing_since {
            if since.elapsed().as_secs_f64() > 5.0 {
                dot.material.state = State::Solid;
                dot.material.temperature = 0.0;
                dot.material.luminescence = 0.0;
                dot.glowing_since = None;
                // 固体化したら、このフレームでの他の状態変化はスキップ
                continue;
            }
        }

        // 高温時の状態変化
        if dot.material.temperature > dot.material.heat_capacity_high {
            dot.material.heat_conductivity += 0.1 * dt as f32;
            if dot.material.heat_conductivity > 1.0 {
                // volatilityが0.5以上の場合のみ状態変化
                if dot.material.volatility >= 0.5 {
                    match dot.material.state {
                        State::Solid => {
                            dot.material.state = State::Liquid;
                            // 連続した状態変化を防ぐためにパラメータをランダム化
                            dot.material.heat_capacity_high = rng.gen(); // 0.0 ~ 1.0
                            dot.material.temperature =
                                dot.material.heat_capacity_high * rng.gen::<f32>(); // 新しい上限より低い値に
                            dot.material.heat_conductivity = rng.gen(); // 0.0 ~ 1.0
                        }
                        State::Liquid => {
                            dot.material.state = State::Gas;
                            // 連続した状態変化を防ぐためにパラメータをランダム化
                            dot.material.heat_capacity_high = rng.gen(); // 0.0 ~ 1.0
                            dot.material.temperature =
                                dot.material.heat_capacity_high * rng.gen::<f32>(); // 新しい上限より低い値に
                            dot.material.heat_conductivity = rng.gen(); // 0.0 ~ 1.0
                        }
                        State::Gas => {
                            // 発光状態に移行
                            dot.material.luminescence = 1.0;
                            dot.glowing_since = Some(Instant::now());
                            // パラメータをリセットして、すぐに再発火しないようにする
                            dot.material.heat_capacity_high = rng.gen();
                            dot.material.temperature =
                                dot.material.heat_capacity_high * rng.gen::<f32>();
                            dot.material.heat_conductivity = rng.gen();
                        }
                    }
                }
            }
        }

        // 低温時の状態変化 (plan.md L105-L111)
        if dot.material.temperature < -dot.material.heat_capacity_low {
            match dot.material.state {
                State::Gas => {
                    dot.material.state = State::Liquid;
                    // 連続した状態変化を防ぐためにパラメータをランダム化
                    dot.material.heat_capacity_low = rng.gen::<f32>() - 1.0; // 0.0 ~ 1.0
                    dot.material.temperature =
                        -dot.material.heat_capacity_low * rng.gen::<f32>(); // 新しい下限より高い値に
                    dot.material.heat_conductivity = rng.gen(); // 0.0 ~ 1.0
                }
                State::Liquid => {
                    dot.material.state = State::Solid;
                    // 連続した状態変化を防ぐためにパラメータをランダム化
                    dot.material.heat_capacity_low = rng.gen::<f32>() - 1.0; // 0.0 ~ 1.0
                    dot.material.temperature =
                        -dot.material.heat_capacity_low * rng.gen::<f32>(); // 新しい下限より高い値に
                    dot.material.heat_conductivity = rng.gen(); // 0.0 ~ 1.0
                }
                State::Solid => {
                    // クールダウンチェック
                    if dot.last_check_time.elapsed().as_secs_f64() > COOL_DOWN_SECONDS {
                        if rng.gen::<f32>() < 0.001 {
                            // 0.1%の確率で崩壊
                            dots_to_remove.push(i);
                        }
                        // 確率判定を行ったら時刻を更新
                        dot.last_check_time = Instant::now();
                    }
                }
            }
        }

        // 爆発条件のチェック (plan.md L65-66, 爆発処理)
        let is_stationary = (dot.vx * dot.vx + dot.vy * dot.vy) < 0.1;
        if dot.material.entropy_bias >= 0.8 && dot.material.volatility >= 0.5 && is_stationary {
            // heat_conductivityが高いほど爆発しやすくなる
            let explosion_probability = dot.material.heat_conductivity * 0.01; // 係数は要調整
            if rng.gen::<f32>() < explosion_probability {
                let explosion_power = dot.material.heat_conductivity;
                explosions.push(Explosion {
                    x: dot.x,
                    y: dot.y,
                    radius: 20.0 + (explosion_power * 80.0) as f64, // 20 ~ 100
                    force: 100.0 + (explosion_power * 400.0) as f64, // 100 ~ 500
                    heat: explosion_power * 1.5, // 0.0 ~ 1.5
                });
                // 爆発したドットを削除リストに追加
                dots_to_remove.push(i);
                continue; // この後の処理はスキップ
            }
        }
    }

    // 2. 爆発の影響を適用
    if !explosions.is_empty() {
        for explosion in &explosions {
            for (i, dot) in dots.iter_mut().enumerate() {
                if dots_to_remove.contains(&i) {
                    continue;
                }

                let dx = dot.x - explosion.x;
                let dy = dot.y - explosion.y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq < explosion.radius * explosion.radius {
                    let distance = distance_sq.sqrt();
                    if distance > 1e-6 {
                        let falloff = 1.0 - (distance / explosion.radius);
                        let force = explosion.force * falloff;
                        let nx = dx / distance;
                        let ny = dy / distance;

                        dot.vx += nx * force * dt;
                        dot.vy += ny * force * dt;

                        // 爆発による熱影響
                        dot.material.temperature += explosion.heat * falloff as f32 * 0.5;
                        dot.material.temperature = dot.material.temperature.clamp(-1.0, 2.0);
                        // 温度の有効範囲を少し超えることを許容する
                    }
                }
            }
        }
    }

    // 3. 通常の力を適用
    for (i, dot) in dots.iter_mut().enumerate() {
        if dots_to_remove.contains(&i) {
            continue;
        }

        match dot.material.state {
            State::Solid | State::Liquid => {
                dot.vy += gravity * dt;
            }
            State::Gas => {
                let buoyancy = (GAS_REFERENCE_DENSITY - dot.material.density) as f64 * gravity;
                dot.vy -= buoyancy * dt;
                let diffusion_strength =
                    (1.0 - dot.material.viscosity) as f64 * GAS_DIFFUSION_FACTOR;
                dot.vx += (rng.gen::<f64>() - 0.5) * diffusion_strength * dt;
                dot.vy += (rng.gen::<f64>() - 0.5) * diffusion_strength * dt;
            }
        }
    }

    // 4. 爆発したドットを削除
    dots_to_remove.sort_unstable();
    dots_to_remove.reverse();
    for i in dots_to_remove {
        dots.remove(i);
    }
}

pub fn update_position(dots: &mut Vec<Dot>, dt: f64) -> bool {
    let mut all_stopped = true;
    let mut rng = thread_rng();

    for dot in dots {
        dot.x += dot.vx * dt;

        dot.y += dot.vy * dt;

        let elasticity = dot.material.elasticity as f64;

        // Handle bottom boundary collision with different behavior based on state
        if dot.y >= (HEIGHT as f64 - DOT_RADIUS) {
            dot.y = HEIGHT as f64 - DOT_RADIUS;

            // Different behavior based on material state
            match dot.material.state {
                State::Solid => {
                    // Solids bounce with elasticity and stop when velocity is low
                    dot.vy *= -elasticity;
                    // 床との摩擦を適用
                    let friction_factor = dot.material.viscosity as f64 * 0.7; // 係数を調整
                    dot.vx *= (1.0 - friction_factor).max(0.0);
                    // Particles behave similarly to solids but with more spreading
                    if dot.material.viscosity < 0.5 {
                        let spread_factor = (1.0 - dot.material.viscosity as f64) * 1.5;
                        dot.vx += (rng.gen::<f64>() - 0.5) * spread_factor;
                    }
                }
                State::Liquid => {
                    // Liquids spread based on viscosity
                    dot.vy *= -elasticity * (1.0 - dot.material.viscosity as f64); // More viscous liquids lose more vertical energy

                    // Apply horizontal spreading based on viscosity
                    if dot.material.viscosity < 0.7 {
                        // Only spread if not highly viscous
                        let spread_factor = (1.0 - dot.material.viscosity as f64) * 2.0;
                        dot.vx += (rng.gen::<f64>() - 0.5) * spread_factor;
                    }
                }
                State::Gas => {
                    // Gases should have minimal interaction with bottom, just bounce slightly
                    dot.vy *= -elasticity * 0.1; // Very little bounce to keep gases moving
                }
            }
        }

        if dot.y <= DOT_RADIUS {
            dot.y = DOT_RADIUS;

            // For gases, minimal bounce to keep them moving
            if dot.material.state == State::Gas {
                dot.vy *= -elasticity * 0.1;
            } else {
                // Apply viscosity effect when hitting top boundary too
                if dot.material.state == State::Solid && dot.material.viscosity < 0.6 {
                    let spread_factor = (1.0 - dot.material.viscosity as f64) * 0.3;
                    dot.vx += (rng.gen::<f64>() - 0.5) * spread_factor * 0.3; // Very limited horizontal variability
                }
                dot.vy *= -elasticity;
            }
        }
        if dot.x >= (WIDTH as f64 - DOT_RADIUS) {
            dot.x = WIDTH as f64 - DOT_RADIUS;

            // Handle gas differently - allow more energy to be preserved
            if dot.material.state == State::Gas {
                dot.vx *= -elasticity * 0.3; // Gases preserve more horizontal momentum
            } else {
                // Apply viscosity effect when hitting side walls too
                if dot.material.state == State::Solid && dot.material.viscosity < 0.6 {
                    let spread_factor = (1.0 - dot.material.viscosity as f64) * 0.3;
                    dot.vy += (rng.gen::<f64>() - 0.5) * spread_factor * 0.3; // Very limited vertical variability
                }
                dot.vx *= -elasticity;
            }
        }

        if dot.x <= DOT_RADIUS {
            dot.x = DOT_RADIUS;

            // Handle gas differently - allow more energy to be preserved
            if dot.material.state == State::Gas {
                dot.vx *= -elasticity * 0.3; // Gases preserve more horizontal momentum
            } else {
                // Apply viscosity effect when hitting side walls too
                if dot.material.state == State::Solid && dot.material.viscosity < 0.6 {
                    let spread_factor = (1.0 - dot.material.viscosity as f64) * 0.3;
                    dot.vy += (rng.gen::<f64>() - 0.5) * spread_factor * 0.3; // Very limited vertical variability
                }
                dot.vx *= -elasticity;
            }
        }

        // ここに減衰処理を追加
        let damping_factor = 0.998; // 速度を99.8%に減衰
        dot.vx *= damping_factor;
        dot.vy *= damping_factor;

        if dot.material.state != State::Gas {
            let velocity_small = dot.vy.abs() < 0.1 && dot.vx.abs() < 0.1;

            let at_bottom = dot.y >= (HEIGHT as f64 - DOT_RADIUS - 1.0);

            if !(velocity_small && at_bottom) {
                all_stopped = false;
            }
        } else {
            all_stopped = false;
        }
    }
    all_stopped
}

// --- ヘルパー関数 ---

// Solid/Liquid間の詳細な衝突処理
fn handle_detailed_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;

    let m1 = dot1.material.density as f64 * (1.0 + dot1.material.hardness as f64);
    let m2 = dot2.material.density as f64 * (1.0 + dot2.material.hardness as f64);

    let v1n = dot1.vx * nx + dot1.vy * ny;

    let v2n = dot2.vx * nx + dot2.vy * ny;

    let v1n_new = (m1 * v1n + m2 * v2n - m2 * e * (v1n - v2n)) / (m1 + m2);

    let v2n_new = (m1 * v1n + m2 * v2n + m1 * e * (v1n - v2n)) / (m1 + m2);

    dot1.vx += (v1n_new - v1n) * nx;

    dot1.vy += (v1n_new - v1n) * ny;

    dot2.vx += (v2n_new - v2n) * nx;

    dot2.vy += (v2n_new - v2n) * ny;

    let density_diff = dot1.material.density - dot2.material.density;

    if density_diff.abs() > 0.1 {
        if dot1.y < dot2.y && density_diff > 0.0 {
            dot1.vy += density_diff as f64 * 5.0 * dt;

            dot2.vy -= density_diff as f64 * 5.0 * dt;
        } else if dot2.y < dot1.y && density_diff < 0.0 {
            dot2.vy += density_diff.abs() as f64 * 5.0 * dt;

            dot1.vy -= density_diff.abs() as f64 * 5.0 * dt;
        }
    }

    let avg_viscosity = (dot1.material.viscosity + dot2.material.viscosity) / 2.0;

    // --- 熱交換 ---
    let temp_diff = dot1.material.temperature - dot2.material.temperature;
    let avg_heat_conductivity =
        (dot1.material.heat_conductivity + dot2.material.heat_conductivity) / 2.0;
    let heat_transfer = (temp_diff * avg_heat_conductivity * 0.1).clamp(-1.0, 1.0); // NaNガード
    
    dot1.material.temperature = (dot1.material.temperature - heat_transfer).clamp(-1.0, 1.0);
    dot2.material.temperature = (dot2.material.temperature + heat_transfer).clamp(-1.0, 1.0);

    // --- 凝集力 ---
    if dot1.material.state == dot2.material.state {
        let avg_cohesion = (dot1.material.cohesion + dot2.material.cohesion) / 2.0;
        if avg_cohesion > 0.01 { // 計算負荷を減らすために閾値を設ける
            let ideal_dist = DOT_RADIUS * 1.5; // この距離に近づけようとする
            let dist_sq = (dot1.x - dot2.x).powi(2) + (dot1.y - dot2.y).powi(2);
            let dist = dist_sq.sqrt();
            
            // 凝集力が働く範囲 (e.g., DOT_RADIUS * 4)
            let effective_range = DOT_RADIUS * 4.0;

            if dist < effective_range && dist > 1e-6 {
                // 理想的な距離との差に基づいて力を計算
                let force_magnitude = (ideal_dist - dist) * (avg_cohesion as f64) * 0.01; // 係数は要調整
                
                let nx = (dot2.x - dot1.x) / dist;
                let ny = (dot2.y - dot1.y) / dist;

                let force_x = nx * force_magnitude;
                let force_y = ny * force_magnitude;

                // 質量に応じて力を適用
                let m1 = dot1.material.density as f64;
                let m2 = dot2.material.density as f64;
                let total_mass = m1 + m2;
                if total_mass > 1e-6 {
                    dot1.vx += force_x * (m2 / total_mass);
                    dot1.vy += force_y * (m2 / total_mass);
                    dot2.vx -= force_x * (m1 / total_mass);
                    dot2.vy -= force_y * (m1 / total_mass);
                }
            }
        }
    }

    // Liquids spread based on viscosity
    if dot1.material.state == State::Liquid && dot2.material.state == State::Liquid {
        let viscosity_threshold = 0.5; // Liquids require lower viscosity to spread

        if avg_viscosity < viscosity_threshold && ny.abs() > 0.8 {
            let spread_factor = (1.0 - avg_viscosity as f64) * 10.0; // Normal spreading force for liquids
            let spread_force = spread_factor * dt;

            if dot1.x < dot2.x {
                dot1.vx -= spread_force;
                dot2.vx += spread_force;
            } else {
                dot1.vx += spread_force;
                dot2.vx -= spread_force;
            }
        }
    }
}

// Gas間の衝突処理
fn handle_gas_collision(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64) {
    let e = (dot1.material.elasticity + dot2.material.elasticity) as f64 / 2.0;

    let m1 = dot1.material.density as f64;

    let m2 = dot2.material.density as f64;

    let v1n = dot1.vx * nx + dot1.vy * ny;

    let v2n = dot2.vx * nx + dot2.vy * ny;

    let v1n_new = (m1 * v1n + m2 * v2n - m2 * e * (v1n - v2n)) / (m1 + m2);

    let v2n_new = (m1 * v1n + m2 * v2n + m1 * e * (v1n - v2n)) / (m1 + m2);

    dot1.vx += (v1n_new - v1n) * nx;

    dot1.vy += (v1n_new - v1n) * ny;

    dot2.vx += (v2n_new - v2n) * nx;

    dot2.vy += (v2n_new - v2n) * ny;

    // --- 熱交換 ---
    let temp_diff = dot1.material.temperature - dot2.material.temperature;
    let avg_heat_conductivity =
        (dot1.material.heat_conductivity + dot2.material.heat_conductivity) / 2.0;
    let heat_transfer = (temp_diff * avg_heat_conductivity * 0.1).clamp(-1.0, 1.0); // NaNガード

    dot1.material.temperature = (dot1.material.temperature - heat_transfer).clamp(-1.0, 1.0);
    dot2.material.temperature = (dot2.material.temperature + heat_transfer).clamp(-1.0, 1.0);
}

// Gasが他の物体に押される処理
fn handle_gas_displacement(gas: &mut Dot, other: &Dot, nx: f64, ny: f64) {
    let e = (gas.material.elasticity + other.material.elasticity) as f64 / 2.0;

    let v_gas_n = gas.vx * nx + gas.vy * ny;

    // nx,nyは常に gas->other を指すため、v_gas_nが正なら向かっている
    if v_gas_n > 0.0 {
        gas.vx -= (1.0 + e) * v_gas_n * nx;

        gas.vy -= (1.0 + e) * v_gas_n * ny;
    }
}

// 液体の蓄積の処理
fn handle_liquid_accumulation(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    // Only apply accumulation if both dots are near the bottom
    let near_bottom = dot1.y >= (HEIGHT as f64 - DOT_RADIUS - 5.0)
        || dot2.y >= (HEIGHT as f64 - DOT_RADIUS - 5.0);

    if !near_bottom {
        return;
    }

    // Calculate average viscosity of the two liquid dots (convert f32 to f64)
    let avg_viscosity = ((dot1.material.viscosity + dot2.material.viscosity) / 2.0) as f64;
    let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

    // Calculate how much to spread based on viscosity (less viscous spreads more)
    let spread_factor = (1.0 - avg_viscosity) * (1.0 - avg_hardness) * 0.5; // Reduced factor to make it more natural

    // If the collision is more horizontal than vertical, apply spreading force
    if nx.abs() > ny.abs() {
        // Apply lateral spreading based on viscosity
        if dot1.x < dot2.x {
            dot1.vx -= spread_factor * dt * 10.0;
            dot2.vx += spread_factor * dt * 10.0;
        } else {
            dot1.vx += spread_factor * dt * 10.0;
            dot2.vx -= spread_factor * dt * 10.0;
        }
    }

    // Apply small vertical force to simulate liquid pressure
    // Less viscous liquids will have more vertical movement
    let vertical_factor = avg_viscosity * 0.1; // Very small vertical movement
    dot1.vy += vertical_factor * dt * 5.0;
    dot2.vy += vertical_factor * dt * 5.0;
}

// 固体間の粘度に基づく広がり処理
fn handle_solid_spreading(dot1: &mut Dot, dot2: &mut Dot, nx: f64, ny: f64, dt: f64) {
    let avg_viscosity = ((dot1.material.viscosity + dot2.material.viscosity) / 2.0) as f64;

    // --- 摩擦処理 ---
    // 接線方向のベクトル (tangent)
    let tx = -ny;
    let ty = nx;

    // 接線方向の相対速度
    let v_rel_t = (dot2.vx - dot1.vx) * tx + (dot2.vy - dot1.vy) * ty;

    // 摩擦による速度変化量。v_rel_tを0に近づける方向に力を加える
    // 粘度が高いほど強くなる
    let friction_impulse = v_rel_t * avg_viscosity * 0.5; // 係数は要調整

    // 質量に応じて摩擦力積を適用
    let m1 = dot1.material.density as f64;
    let m2 = dot2.material.density as f64;
    let total_mass = m1 + m2;
    if total_mass > 1e-6 {
        dot1.vx += friction_impulse * (m2 / total_mass) * tx;
        dot1.vy += friction_impulse * (m2 / total_mass) * ty;
        dot2.vx -= friction_impulse * (m1 / total_mass) * tx;
        dot2.vy -= friction_impulse * (m1 / total_mass) * ty;
    }

    let avg_hardness = ((dot1.material.hardness + dot2.material.hardness) / 2.0) as f64;

    // --- 広がりと圧力の処理 ---
    // 粘度が低いほどわずかに広がる力を持たせる
    if avg_viscosity < 0.8 && avg_hardness < 0.5 {
        let spread_factor = (1.0 - avg_viscosity) * (1.0 - avg_hardness) * 0.01; // 係数をさらに小さく
                                                                                 // ほぼ上下の衝突のときだけ、わずかに広げる
        if ny.abs() > 0.8 {
            let spread_force = spread_factor * dt;
            if dot1.x < dot2.x {
                dot1.vx -= spread_force;
                dot2.vx += spread_force;
            } else {
                dot1.vx += spread_force;
                dot2.vx -= spread_force;
            }
        }
    }

    // 縦方向の圧力。粘度が高いほど強くかかる
    // これが積み重なる効果を生むはず
    let vertical_factor = avg_viscosity * 0.05;
    dot1.vy += vertical_factor * dt;
    dot2.vy += vertical_factor * dt;
}
