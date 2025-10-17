# Terraspiel - Alchemy Pixel Playground

## コンセプト

フォーリングサンド系の物理シミュレーションに「アルケミー合成」要素を組み合わせる。
ユーザーが置ける物質は１種類だけど、その物質には様々な特性（個体液体気体、粘度密度、硬度、色などなど）があってユーザーが任意に設定したものを置ける。
物質はそれぞれ接触したときに保持している特性が混ざり合って新たな物質に変化する。みたいなのも考えた。
変換テーブル的なのはなくて、特性とそのときのシードで物質名からなにもかもを決定してしまうシステム。

## 基本コンセプト：「生成される物質そのものが世界のルール」

ユーザーは「一つのベース物質」を置く。
ベース物質の**パラメータ（特性）**を自由に設定できる：
状態: 固体／液体／気体
密度, 粘度, 温度耐性, 電導率, 光反射率, 色, 熱伝導率, 反応性…など
シミュレーション中、セル同士が接触すると、それぞれの特性がブレンドされて**新しい「物質DNA」**を作る。
そのDNAをハッシュ or シードにして、物質名・色・物理特性などを一貫して生成。

ブレンド方法は単純な線形補間でもいいし、ノイズ関数（Perlin/Simplex）を挟むのも面白い。
各特性は範囲（例: 密度=0〜1）を持ち、そこから中間値を生成。

見た目と名前の自動生成
シードを使って：
名前: ノイズ＋ルール生成（例: "Aeralite", "Mundra", "Saphone" みたいな）
色: 特性のスペクトルから導出（熱伝導率や電導率が高いほど青く光る等）
物性: 特性セットを正規化して連動（密度が高いと落下しやすい、熱伝導率高いと燃えやすい等）

## 技術スタック（Rust + Pixels + Egui 構成）

### 🧠 コア層（シミュレーションエンジン）

| 機能 | 採用技術 / クレート | 概要 |
|------|----------------------|------|
| **メインループ / イベント** | [`winit`](https://crates.io/crates/winit) | クロスプラットフォームなウィンドウ・入力管理 |
| **ピクセル描画** | [`pixels`](https://crates.io/crates/pixels) | GPU上に直接ピクセルを書き込む軽量2Dレンダラ |
| **ノイズ生成** | [`noise`](https://crates.io/crates/noise) | 特性ブレンドや物質DNA生成で使用 |
| **乱数生成** | [`rand`](https://crates.io/crates/rand), [`fastrand`](https://crates.io/crates/fastrand) | シードベースで決定的なランダム性を付与 |
| **シリアライズ / データ保存** | [`serde`](https://crates.io/crates/serde), [`bincode`](https://crates.io/crates/bincode) | シミュレーション状態の保存・ロード |
| **並列処理** | [`rayon`](https://crates.io/crates/rayon) | チャンク単位でセル更新を並列化 |
| **プロファイル / ログ** | [`tracing`](https://crates.io/crates/tracing), [`tracy-client`](https://crates.io/crates/tracy-client) | パフォーマンス計測とデバッグ |

---

### 🎨 描画・UI層（Pixels + Egui）

| 機能 | 採用技術 / クレート | 概要 |
|------|----------------------|------|
| **UIオーバーレイ** | [`egui`](https://crates.io/crates/egui) + [`egui-winit`](https://crates.io/crates/egui-winit) + [`egui-wgpu`](https://crates.io/crates/egui-wgpu) | ピクセル画面上にスライダーやパネルを重ねて表示 |
| **フォント描画** | [`ab_glyph`](https://crates.io/crates/ab_glyph), [`fontdue`](https://crates.io/crates/fontdue) | 物質名やUIテキストの描画 |
| **カラーモデル処理** | [`palette`](https://crates.io/crates/palette) | 特性→色変換（熱伝導率・反射率などを可視化） |

---

### ⚗️ シミュレーション構造（Terraspiel Engine）

| 要素 | 実装方針 |
|------|-----------|
| **セル構造体** | 固定グリッド（`Vec<Cell>` or `ndarray>`）。各セルは`Material`を持ち、状態・密度・粘度などの特性を格納。 |
| **更新アルゴリズム** | スキャン方式（行方向／ランダム順序）。アクティブセルマップで不要領域をスキップ。 |
| **ブレンド処理** | 線形補間＋ノイズ補正による非線形特性ブレンド。 |
| **DNA生成** | 特性値をハッシュ化（`blake3`など）して擬似的な「物質DNA」を決定。 |
| **物質命名** | DNAハッシュから疑似語を生成（Markovや自作ルール）。 |
| **並列処理** | チャンク分割して`rayon`による並列ステップ更新。 |

---

### 🧩 UI構成（Eguiパネル例）

| パネル | 内容 |
|--------|------|
| **Material Editor** | 各特性スライダー（密度、粘度、導電率、熱耐性、反射率など） |
| **State Selector** | 固体／液体／気体トグルボタン |
| **Color Preview** | 現在の特性スペクトルに基づく色のリアルタイムプレビュー |
| **DNA Viewer** | ハッシュと生成された物質名の表示 |
| **Statistics HUD** | FPS、セル更新数、使用メモリなどの監視情報 |

---

### ⚙️ 実行・ビルド環境

| 項目 | 使用技術 / ツール | 備考 |
|------|-------------------|------|
| **ビルドシステム** | `cargo` | 標準Rustツールチェーン |
| **ターゲット** | Windows / Linux / macOS | `winit` と `wgpu` によりクロス対応 |
| **配布** | `cargo-bundle`, `cargo-dist` | スタンドアロン実行ファイル生成 |
| **デバッグ / プロファイル** | `tracing`, `tracy`, `profiling` | 性能ボトルネック解析 |
| **フォーマット / Lint** | `rustfmt`, `clippy` | コード整形・静的解析 |

---

### 🚀 拡張計画（Future Options）

| 分類 | 機能 | 詳細 |
|------|------|------|
| **Web対応** | `wasm32-unknown-unknown`ターゲット | Pixels + eguiのWebバックエンドでブラウザ動作 |
| **GPU計算** | `wgpu::compute_shader` | 物理演算をGPGPU化して更なる高速化 |
| **サウンド反応** | `rodio` / `cpal` | 物質反応時の音生成 |
| **モジュラー構成** | `terraspiel_core` / `terraspiel_ui` / `terraspiel_app` | エンジンとUIを分離したモジュール構成 |

---

### 📁 ディレクトリ構成（案）

terraspiel/
├─ Cargo.toml
├─ src/
│ ├─ main.rs
│ ├─ sim/
│ │ ├─ mod.rs
│ │ ├─ cell.rs
│ │ ├─ material.rs
│ │ ├─ world.rs
│ │ └─ update.rs
│ ├─ ui/
│ │ ├─ mod.rs
│ │ └─ controls.rs
│ ├─ render.rs
│ └─ dna.rs
└─ assets/
├─ fonts/
└─ shaders/

## 最小動作サンプル

ウィンドウを開く（winit＋pixels）
ピクセルを描画する（背景グラデーションやノイズでOK）
egui UIを重ねる（スライダーやボタンを1つ）
UI操作が描画結果に反映される（例：スライダーで色が変わる）

## UI
