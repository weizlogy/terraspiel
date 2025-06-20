// src/core/seed_generator.rs

use chrono::Utc;
use sha2::{Digest, Sha256};

/// 現在の日時を基に、SHA-256ハッシュを使用してユニークなシード値を生成します。
///
/// 生成プロセス：
/// 1. 現在のUTC日時をRFC3339形式の文字列として取得します。
/// 2. その文字列をUTF-8バイト列に変換します。
/// 3. SHA-256ハッシュアルゴリズムを適用します。
/// 4. 計算されたハッシュ値の最初の8バイトを取り出します。
/// 5. これら8バイトを `u64` 型の数値として解釈し、シード値として返します。
///
/// これにより、実行するたびに異なる（ただし、非常に短い時間内に連続して実行された場合は同じになる可能性のある）
/// シード値が得られ、ゲームワールドの多様性が生まれます。
pub fn generate_seed() -> u64 {
  // 1. 現在のUTC日時を文字列として取得
  let now_str = Utc::now().to_rfc3339();

  // 2. 文字列をSHA-256でハッシュ化
  let mut hasher = Sha256::new();
  hasher.update(now_str.as_bytes());
  let result = hasher.finalize();

  // 3. ハッシュ値の最初の8バイトをu64に変換
  let mut seed_bytes = [0u8; 8];
  seed_bytes.copy_from_slice(&result[0..8]);
  u64::from_le_bytes(seed_bytes) // リトルエンディアンとして解釈
}