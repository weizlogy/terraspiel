mod app;
mod material;
mod naming;
mod physics;
mod renderer;

use app::{App, BlendResult};
use material::{decide_reaction_type, from_dna, ReactionType};
use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    // 衝突イベント送受信用チャネル
    let (collision_tx, collision_rx) = mpsc::channel();
    // ブレンド結果送受信用チャネル
    let (result_tx, result_rx) = mpsc::channel::<BlendResult>();

    let mut app = App::new(collision_tx, result_rx);

    // --- ワーカースレッドを起動 ---
    thread::spawn(move || {
        // 衝突イベントをバッチ処理するためのベクトル
        let mut collision_batch = Vec::with_capacity(1024);

        loop {
            // 最初のイベントをブロックして待つ
            match collision_rx.recv() {
                Ok(first_event) => {
                    collision_batch.push(first_event);
                    // キューに残っているイベントをすべて取得
                    collision_batch.extend(collision_rx.try_iter());

                    // バッチを並列処理
                    let results: Vec<BlendResult> = collision_batch
                        .par_iter()
                        .flat_map(|((index_a, dna_a), (index_b, dna_b))| {
                            if dna_a.seed == dna_b.seed {
                                return Vec::new(); // 同じseedを持つドットはブレンドしない
                            }

                            let params_a = from_dna(dna_a);
                            let params_b = from_dna(dna_b);

                            let reaction_type = decide_reaction_type(params_a.state, params_b.state);
                            let new_dna = dna_a.blend(dna_b, 0.5);

                            let mut results = Vec::new();

                            match reaction_type {
                                ReactionType::Reaction => {
                                    results.push(BlendResult::Change { index: *index_a, new_dna: new_dna.clone() });
                                    results.push(BlendResult::Change { index: *index_b, new_dna });
                                }
                                ReactionType::CatalyticLowChanges => {
                                    let energy_a = params_a.state.get_energy_level();
                                    let energy_b = params_b.state.get_energy_level();
                                    if energy_a < energy_b {
                                        results.push(BlendResult::Change { index: *index_a, new_dna });
                                    } else {
                                        results.push(BlendResult::Change { index: *index_b, new_dna });
                                    }
                                }
                                ReactionType::CatalyticHighChangesAndLowVanishes => {
                                    let energy_a = params_a.state.get_energy_level();
                                    let energy_b = params_b.state.get_energy_level();
                                    if energy_a > energy_b {
                                        results.push(BlendResult::Change { index: *index_a, new_dna });
                                        results.push(BlendResult::Vanish { index: *index_b });
                                    } else {
                                        results.push(BlendResult::Change { index: *index_b, new_dna });
                                        results.push(BlendResult::Vanish { index: *index_a });
                                    }
                                }
                            }
                            results
                        })
                        .collect();

                    // 結果をメインスレッドに送信
                    for result in results {
                        if result_tx.send(result).is_err() {
                            // メインスレッドが終了した場合
                            break;
                        }
                    }

                    collision_batch.clear();
                }
                Err(_) => {
                    // 送信側がすべて破棄されたらループを抜ける
                    break;
                }
            }
        }
    });

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
            match event {
                Event::Resumed => {
                    app.handle_resume(event_loop);
                }
                Event::WindowEvent { event, window_id } => {
                    let window = match app.window.as_ref() {
                        Some(w) => w.clone(),
                        None => return,
                    };

                    let consumed_by_egui = app.handle_window_event(&window, &event);
                    if consumed_by_egui {
                        return;
                    }

                    if window.id() == window_id {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                app.resize(physical_size);
                            }
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
                _ => {}
            }

            // アニメーションや更新が必要な場合は、再描画をリクエスト
            if let Some(ref window) = app.window {
                window.request_redraw();
            }
        })
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
