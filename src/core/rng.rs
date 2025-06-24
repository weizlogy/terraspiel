//! src/core/rng.rs

use rand::{Rng as RandRng, RngCore, SeedableRng}; // RngCoreをインポート
use rand::rngs::{StdRng, ThreadRng};
use rand::seq::SliceRandom;

// --- Custom Object-Safe RNG Trait ---
/// ワールド生成やゲームプレイで必要な乱数操作を定義する、オブジェクト安全なトレイト。
/// `rand::Rng`トレイトはジェネリックメソッドを持つためオブジェクト安全ではないので、
/// ここで必要なメソッドを再定義し、`dyn`キーワードで利用できるようにします。
pub trait GameRngMethods: RngCore { // RngCoreはオブジェクト安全なので継承できる
  /// 0.0から1.0の範囲の`f64`乱数を生成します。
  fn gen_f64(&mut self) -> f64;

  /// 指定された確率で`bool`値を生成します。
  fn gen_bool_prob(&mut self, p: f64) -> bool;

  /// 指定された範囲の`f64`乱数を生成します。
  fn gen_f64_range(&mut self, low: f64, high: f64) -> f64;

  /// 指定された範囲の`usize`乱数を生成します。
  fn gen_usize_range(&mut self, low: usize, high: usize) -> usize;

  /// 指定された範囲の`i32`乱数を生成します。
  fn gen_i32_range(&mut self, low: i32, high: i32) -> i32;
}

// --- Blanket Implementation for rand::Rng ---
// rand::Rngを実装する任意の型Tに対して、GameRngMethodsを実装します。
// これにより、StdRngなどのrand::Rng実装をdyn GameRngMethodsとして扱えるようになります。
impl<T: RandRng> GameRngMethods for T {
  fn gen_f64(&mut self) -> f64 { self.random::<f64>() }
  fn gen_bool_prob(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn gen_f64_range(&mut self, low: f64, high: f64) -> f64 { self.random_range(low..high) }
  fn gen_usize_range(&mut self, low: usize, high: usize) -> usize { self.random_range(low..high) }
  fn gen_i32_range(&mut self, low: i32, high: i32) -> i32 { self.random_range(low..high) }
}

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
      thread_rng: rand::rng(),
    }
  }

  /// ワールド生成用のRNGへの可変参照を返します。
  /// オブジェクト安全な`GameRngMethods`トレイトオブジェクトとして返します。
  pub fn world_gen_mut(&mut self) -> &mut dyn GameRngMethods {
    &mut self.world_rng
  }

  /// ゲームプレイ用のRNGへの可変参照を返します。
  /// オブジェクト安全な`GameRngMethods`トレイトオブジェクトとして返します。
  pub fn gameplay_gen_mut(&mut self) -> &mut dyn GameRngMethods {
    &mut self.gameplay_rng
  }

  /// スライスをシャッフルします。この操作にはスレッドローカルなRNGを使用します。
  pub fn shuffle<T>(&mut self, slice: &mut [T]) {
    slice.shuffle(&mut self.thread_rng);
  }

}