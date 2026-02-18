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

## 見た目と名前の自動生成
シードを使って：
名前: ノイズ＋ルール生成
色: 特性のスペクトルから導出（熱伝導率や電導率が高いほど青く光る等）
物性: 特性セットを正規化して連動（密度が高いと落下しやすい、熱伝導率高いと燃えやすい等）

## 最小動作サンプル

ウィンドウを開く（winit＋pixels）
ピクセルを描画する（背景グラデーションやノイズでOK）
egui UIを重ねる（スライダーやボタンを1つ）
UI操作が描画結果に反映される（例：スライダーで色が変わる）

## ベース物質のパラメータ（特性）

| カテゴリ              | パラメータ名        | 範囲・型  | 説明                                           |
| --------------------- | --------------------| ----------| ---------------------------------------------- |
| **基本**              | `state`             | enum      | 物質の状態。動作パターンの基本軸になる。       |
|                       |                     | Solid     | 固体。硬い物質。                               |
|                       |                     | Liquid    | 液体。流動性のある物質。                       |
|                       |                     | Gas       | 気体。圧力に影響を受けやすい物質。             |
|                       |                     | Particle  | 風や波のような微細な物質。                     |
| **物理特性**          | `density`           | 0.0〜1.0  | 密度。高いほど重く沈む。                       |
|                       | `viscosity`         | 0.0〜1.0  | 粘度。高いほど流動しなくなる。                 |
|                       | `hardness`          | 0.0〜1.0  | 硬度。衝撃や崩壊に対する耐性。                 |
|                       | `elasticity`        | 0.0〜1.0  | 弾性。反発や波動伝播の強さ。                   |
| **熱・エネルギー系**  | `temperature`       | -1.0〜1.0 | 相対温度。周囲より高いと熱伝導で変化を起こす。 |
|                       | `heat_conductivity` | 0.0〜1.0  | 熱伝導率。温度の伝わりやすさ。                 |
|                       | `heat_capacity`     | 0.0〜1.0  | 熱容量。熱変化に対する耐性。                   |
| **光・見た目系**      | `color_hue`         | 0.0〜1.0  | 色相。ブレンド時に線形補間で混ざる。           |
|                       | `color_saturation`  | 0.0〜1.0  | 彩度。                                         |
|                       | `color_luminance`   | 0.0〜1.0  | 明度。                                         |
|                       | `luminescence`      | 0.0〜1.0  | 自発光度。発熱や電気で光を放つ度合い。         |

## 追加特性

### entropy_bias
0.0 → 安定。平均寄りの反応
1.0 → カオス。色相や遺伝子がブレまくる
効果：
DNA ブレンド時のノイズ量に影響
Hue の非線形補正量が増える
見た目：
0.5以上なら、最大値との差によって振れ幅が異なる、色相、彩度、明度の不安定化、粒子の揺れが発生する

entropy_biasが0.8以上で、volatilityが0.5以上ならば、
その物質はthermal_conductivityが高いほど爆発しやすくなる

### volatility
物質がどれくらい「揮発しやすいか」っていう値。
0.0 → 固体のまま
1.0 → すぐ気化
効果：
heat_capacity_highによるState変化の条件に、volatilityが追加され、
volatilityが0.5以上ならStateが変化する、以下なら変化しない
見た目：
volatilityが0.5以上なら、描画時に周囲に(volatility - 0.5)*適当な係数でちょうどいいブラーをかける

## cohesion（凝集力）
density/viscosity/hardness では表現しきれない
“まとまりやすさ”や“砂と泥の違い”。
0.0 → 粒子がバラバラ
1.0 → すぐ塊になる
効果：
粒子がクラスタ化しやすくなる

## 熱伝導率（thermal_conductivity）
「このセルが、周囲のセルとどれくらい速く温度を共有するか」
を決めるパラメータ。
値域：0.0〜1.0
0.0 → 断熱し、周囲のセルと温度を共有しない
1.0 → 超伝導により、周囲のセルと即座に温度を共有する

## 各パラメータのシミュレーション影響まとめ
state
セルの移動ロジック・隣接チェック方法を決定
- Solid: 固定または重力で滑落
- Liquid: 下方向＋横方向に流動
- Gas: 上昇＋拡散
- Particle: 落下しつつ堆積（砂挙動）

density
重力下での落下優先度／浮力バランス
- 他セルと比較して「軽い方が上」「重い方が下」へ交換
- fall_speed ∝ density * gravity

viscosity
移動速度・形状維持力・流動抵抗
- high: 水飴状、動きが鈍く隣接流動が抑制される
- low: サラサラした水や煙
- 実装：移動確率を 1 - viscosity でスケーリング

hardness
崩壊・衝突・侵食耐性
- 周囲から押されても形が崩れにくい
- low: 流体や砂のように容易に崩壊
- 他セルとの接触で破壊確率 ∝ 1 - hardness

elasticity
反発・跳ね返り・波動伝播速度
- 落下時の反発係数
- 周囲への力伝達に利用（衝突反応・弾性波）
- 実装：速度反転時に velocity *= elasticity

temperature
state変化トリガー・熱拡散・光度変化
- 高温: 蒸発 or 発光
- 低温: 凝固 or 着色変化

heat_capacity_highを超えるtemperatureの場合に、heat_conductivityが上昇。上限を超えると、
Solidは徐々に融解しLiquidに変化する。
Liquidは徐々に蒸発しGasに変化する。
Gasは5秒間luminanceを最大化し、その後Solidに変化し、Temperatureとluminanceを0にする。

heat_capacity_low未満のtemperatureの場合に、heat_conductivityが下降。下限を下回ると、
Solidは5秒間luminanceを最小化し、その後Gasに変化する。
Liquidは徐々に冷却しSolidに変化する。
Gasは徐々に冷却しLiquidに変化する。

崩壊の手順
- 該当物質の削除

heat_conductivity
熱伝達スピード
- 高いほど温度変化が速くなる。

heat_capacity_high
温度変化閾値（高温）
- 高いと加熱による変化が発生しにくい。

heat_capacity_low
温度変化閾値（低温）
- 低いと冷却による変化が発生しにくい。

color_hue / saturation / luminance
表示上の色彩表現
- hue：ブレンド結果で変化
- saturation：混ざるほど低下（濁る）
- luminance：視覚的なエネルギー量指標（温度と連動しても良い）

luminescence	自発光（明度オフセット）
- 0.8以上で数値に応じて発光する。
- 温度に応じて発光色を変える
　赤→橙→白→青白
- ガスなら発光を揺らす
- 液体なら反射を加える
  発光セルが周囲を少し明るく染める

## 物質DNAのデータ構造
DNAは「その物質のすべてを決定する数値列」。
再現性とブレンド性を両立するには、シンプルな固定長のバイト列 or ベクトルが理想。

struct MaterialDNA {
  seed: u64,              // ハッシュや世界シードとの組み合わせ
  genes: [f32; 14],       // 各特性を0〜1正規化した値
}

ここで genes[i] は BaseMaterialParams の各要素に対応。
DNAブレンドは単に genes[i] をブレンドするだけでOK。
これを BaseMaterialParams に変換する関数を定義：

fn from_dna(dna: &MaterialDNA) -> BaseMaterialParams {
  // genes順序は固定。シンプルにマッピング。
}

### 名前の自動生成
架空言語風・音素生成ルール表

以下は MaterialDNA の特性値をもとに、音素（phoneme）セットとテンプレートを決定するためのルールです。
各カテゴリは相対値を 0.0〜1.0 として評価し、区間ごとに音素傾向を変化させます。

さらに、Weighted選択ロジックにより特性値によって音素選択がゆるくシフトされる。
つまり 特性値が高いほど末尾寄りの音素を選びやすくなる、というイメージです。

さらに、派生変形（Morph）ロジックにより変化させる。
- 温度連動
低温：短縮（末尾2文字を削除）
高温：母音展開・語尾伸長
- 粘度・硬度連動
高粘度：母音連続挿入
高硬度：子音強化

音素連鎖モデル（Markov Chain / Bigramモデル）により、
有限音素セットでも、統計的連鎖確率を持たせることで自然語のような無限バリエーションを生成可能とする。

各音素をノードとし、遷移確率 P(next|current) を設定
特性値＋SEEDで確率分布をわずかに歪ませる
Solid：硬音遷移率高 r→d, d→r
Liquid：母音遷移率高 a→e, e→a
Gas：滑音遷移率高 ae→l, l→ion

- ■ 状態（State）別ベース音素セット
状態	音素セット（prefix/root/suffix）	音感	備考

Solid（固体）
Prefix: gr, kr, st, dr, br
Root: ar, or, an, ul, tar, dor
Suffix: -on, -ar, -ite, -orn
重厚・鈍重	石・金属・鉱石系。「硬い」閉音節多め

Liquid（液体）
Prefix: el, lo, va, mi, sa
Root: ae, al, ir, ol, ra, lia
Suffix: -ine, -el, -ra, -al
滑らか・流動的	母音多めで音のつながりが柔らかい

Gas（気体）
Prefix: ae, pha, is, sy, lu
Root: el, ar, ia, es, the, ion
Suffix: -is, -os, -ion, -eth
軽い・風っぽい	気配・風音のような語感を重視

Particle（粒子）
Prefix: mu, fi, sha, ki, to
Root: ra, en, ta, il, mi
Suffix: -a, -en, -um, -ir
細かい・繊細	粒子・粉末・霧など微細感

- ■ 温度（Temperature）
範囲	音素傾向	音例
-1.0〜-0.3（冷）	cr, sil, el, is, fr, ne / -el, -ine, -is	“氷”や“静寂”を連想：「Crinels, Silais」
-0.3〜+0.3（中性）	al, er, an, ol, mi / -ar, -en	自然・穏やか：「Molar, Enel」
+0.3〜+1.0（熱）	ra, fi, or, py, th, an / -or, -as, -ar	炎・発熱：「Pyraor, Firan」

- ■ 自発光（Luminescence）
範囲	音素傾向	音例
0.0〜0.3（暗）	mor, dul, tar, ol / -ar, -um	鈍い・闇的
0.3〜0.7（中）	el, la, mi, ae / -en, -al	通常物質
0.7〜1.0（光）	ael, lux, the, ion, syl / -is, -iel, -ae	聖・幻想的：「Aelion, Luxiel」

- ■ 粘度（Viscosity）
範囲	音素傾向	傾向
0.0〜0.4（低）	明瞭短音節 (ki, ra, to)	サラサラ・軽快
0.4〜0.7（中）	柔音 (el, la, sa, mi)	標準流体
0.7〜1.0（高）	濁音多め (gr, dr, mu, ul)	ネバネバ・重厚

- ■ 硬度（Hardness）
範囲	音素傾向	傾向
0.0〜0.3（柔）	li, ne, sa, el / -a, -in	やわらか
0.3〜0.7（中）	ar, en, ol / -en, -ar	標準
0.7〜1.0（硬）	kr, gr, st, dor / -ite, -orn	石っぽい・鋭い

- ⚙ テンプレート定義（構文ルール）
ID	構文	用途・特徴	例
T1	[Prefix][Root]	単純物質	Gral, Lia, Syl
T2	[Prefix][Root][Suffix]	標準形	Lumeron, Kratite
T3	[Root][Suffix]	小物質／派生物	Aelis, Ranor
T4	[Prefix][Root]-[Variant]	合成・変種表現	Cryth-ion, Aera-lis
T5	[AttrPrefix]-[BaseName]	属性付与形	Pyro-Elin, Lux-Aerion

テンプレートは状態＋ルミネセンスを軸に選択：
Solid系: T2 or T3
Liquid系: T2
Gas系: T1 or T4
高Luminescence: T5（冠詞つき）

### DNAから物質名・色を生成する方法
DNAの決定性を維持しつつ、多様性を演出する手続き生成がカギ！

色の決定
color_hue, saturation, luminance はDNAから直接決定。
- color_hue
    非線形ブレンド + ノイズ付きブレンド
- saturation
    加重平均 + 温度補正（温度が高いほど鮮やか）
- luminance
    ランダム

### ブレンド処理の後処理
ブレンドに使用した物質はどうするのがいいか？

相互変化型（Reaction）
AとBが両方とも同時に新物質に変化

触媒型（Catalytic）
一方（触媒セル）は変化せず、もう一方のみ変化する

通常は相互変化型
温度差・状態差が極端な場合は、片方だけ変化（燃焼や蒸発の表現）

状態差（state difference）
状態（Solid, Liquid, Gas, Particle）間には、物理的に大きな差があります。
この差をブレンド反応モードを選ぶための「優先ルール」に使います。

各状態のエネルギーレベル
Solid    0.2
Liquid   0.5
Gas      0.8

状態差による反応分類ルール
ΔE < 0.2	    Reaction（両方変化）	     同質系の混合反応
0.2 ≤ ΔE < 0.5	Catalytic（片方が変化）	エネルギーレベルの高いほうは変化せず、低いほうを変化させる
ΔE ≥ 0.5	    Catalytic（片方が変化）    エネルギーレベルの高いほうが変化し、低いほうを消滅させる


## MaterialDNA 距離仕様
Weighted Euclidean Genetic Distance with Phase Separation Bonus

| State    | Energy Value |
| -------- | ------------ |
| Solid    | 0.2          |
| Particle | 0.4          |
| Liquid   | 0.6          |
| Gas      | 0.9          |

### 重み定義
const WEIGHTS: [f32; 14] = [
  2.5, // state（相の差は重要）
  1.5, // density
  1.0, // viscosity
  1.5, // hardness
  1.2, // elasticity
  1.5, // temperature耐性系
  2.0, // heat_conductivity
  1.5, // heat_capacity
  1.0, // conductivity
  1.0, // magnetism
  0.2, // hue
  0.2, // saturation
  0.2, // luminance
  0.5, // luminescence
];

設計思想
- 物理挙動に直結する遺伝子は重みを大きくする
- 見た目系は軽くする
- state と 熱系は特に重要視する

### 基本距離計算（重み付きユークリッド距離）

### State差ボーナス仕様

目的
- 異相間の物質を明確に分離する
- 同相内は滑らかに近縁判定する

| Δstate  | 意味      | 追加距離  |
| ------- | ------- | ----- |
| < 0.3   | 同相または近縁 | +0.0  |
| 0.3〜0.5 | 異相寄り    | +0.1  |
| ≥ 0.5   | 明確に異相   | +0.25 |

### 距離の意味

| Distance | 解釈      |
| -------- | ------- |
| 0.0      | 完全同一DNA |
| 0.3      | 近縁物質    |
| 0.6      | 別種      |
| 0.9+     | ほぼ無関係   |
| 1.0      | 完全断絶    |

### 熱伝播への適用

特徴
- 近縁物質は強く熱を伝える
- 少し違うだけで急減衰
- 異相はほぼ断熱状態になる

### 利用用途
この距離は以下に共通利用可能：
- 熱伝播係数スケーリング
- 電導伝播
- 反応確率補正
- 爆発伝播強度
- 波動伝達係数
-

### 計算コストと最適化
14次元ループ
sqrt 1回
powf 1回

推奨最適化
- MaterialID 同士の距離をキャッシュ
- 同一フレーム内で再計算しない
- 同DNA同士は距離0固定で即return

### 設計効果
- 物質の「系統」が物理挙動に影響する
- 相の違いが自然な断絶を生む
- アルケミー的“近縁性”が世界ルールになる
- 単なるフォーリングサンドから一段上の構造へ進化する
