//! src/core/rng.rs

use rand::{Rng, SeedableRng};
use rand::rngs::{StdRng, ThreadRng};
use rand::seq::SliceRandom;

/// ゲーム全体で使用される乱数生成器を管理する構造体。
///
/// これにより、乱数生成ロジックを一元管理し、テストや再現性を容易にします。
/// 異なる目的（ワールド生成、ゲームプレイ中のイベントなど）で別々のRNGインスタンスを保持することで、
/// 乱数列が意図せず他の処理に影響を与えることを防ぎます。
pub struct GameRng {
  /// ワールド生成など、決定論的な結果が必要な場合に使用するシード付き乱数生成器。
  world_rng: StdRng,
  /// 草の成長や物理演算のランダム要素など、ゲームプレイ中の動的なイベントに使用する乱数生成器。
  gameplay_rng: StdRng,
  /// 決定論的である必要がない、UIや一時的なエフェクトなどに使用するスレッドローカルな乱数生成器。
  thread_rng: ThreadRng,
}

impl GameRng {
  /// 指定されたシード値から新しい `GameRng` インスタンスを生成します。
  ///
  /// ワールド生成用とゲームプレイ用のRNGは、それぞれ異なる派生シードで初期化され、
  /// 互いに独立した乱数列を生成します。
  pub fn new(seed: u64) -> Self {
    Self {
      world_rng: StdRng::seed_from_u64(seed),
      // ゲームプレイ用RNGには、ワールド生成用とは異なるシードを使用します。
      gameplay_rng: StdRng::seed_from_u64(seed.wrapping_add(1)),
      thread_rng: rand::thread_rng(),
    }
  }

  /// ワールド生成用のRNGへの可変参照を返します。
  pub fn world_mut(&mut self) -> &mut StdRng {
    &mut self.world_rng
  }

  /// スライスをシャッフルします。この操作にはスレッドローカルなRNGを使用します。
  pub fn shuffle<T>(&mut self, slice: &mut [T]) {
    slice.shuffle(&mut self.thread_rng);
  }

  /// ゲームプレイ用のRNGを使用して、指定された確率で `bool` 値を生成します。
  pub fn random_bool(&mut self, p: f64) -> bool {
    self.gameplay_rng.random_bool(p)
  }
}