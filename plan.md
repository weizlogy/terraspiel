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
| **電磁特性**          | `conductivity`      | 0.0〜1.0  | 電導率。電流・光エネルギーの伝播度合い。       |
|                       | `magnetism`         | -1.0〜1.0 | 磁性。正でN極、負でS極的性質を表現可能。       |
| **光・見た目系**      | `color_hue`         | 0.0〜1.0  | 色相。ブレンド時に線形補間で混ざる。           |
|                       | `color_saturation`  | 0.0〜1.0  | 彩度。                                         |
|                       | `color_luminance`   | 0.0〜1.0  | 明度。                                         |
|                       | `luminescence`      | 0.0〜1.0  | 自発光度。発熱や電気で光を放つ度合い。         |

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
相変化トリガー・熱拡散・光度変化
- 高温: 蒸発 or 発光
- 低温: 凝固 or 着色変化
- 周囲温度との差で heat_flux = (neighbor.temp - self.temp) * heat_conductivity

heat_capacityを超えるtemperatureの値に応じて、
Solidは徐々に融解しLiquidに変化する。
Liquidは徐々に蒸発しGasに変化する。
Gasは爆発する。

爆発の手順
- 一瞬の発光（luminescence最大化）
- 周囲を加熱（自身のtemperatureを周囲に伝達）
- 爆風による吹き飛ばし
- 該当物質の消滅

cool_capacity未満のtemperatureの値に応じて、
Solidは崩壊する。
Liquidは徐々に冷却しSolidに変化する。
Gasは徐々に冷却しLiquidに変化する。

崩壊の手順
- 該当物質の削除

heat_conductivity
熱伝達スピード
- 高いほど温度が早く均一化
- 実装：セル間で温度を線形補間（diffusion）

heat_capacity
温度変化閾値（高温）
- 高いと加熱による変化が発生しにくい。

cool_capacity
温度変化閾値（低温）
- 低いと冷却による変化が発生しにくい。

conductivity
電流・エネルギー伝播率
- 近傍の電位差に応じて電流伝搬
- 高ければ連鎖的に発光や加熱が広がる

magnetism
近距離での引力／斥力
- 正負極で方向性のある力を発生
- 他セルとの距離に応じたベクトルを加算（磁力線的挙動）

color_hue / saturation / luminance
表示上の色彩表現
- hue：ブレンド結果で変化
- saturation：混ざるほど低下（濁る）
- luminance：視覚的なエネルギー量指標（温度と連動しても良い）

luminescence	自発光（明度オフセット）
- render_color = base_color + luminescence * glow
- 熱や電気で増加するよう連動させると“生命感”が出る

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
- 電導率・発光連動
高電導率：母音上昇
高発光率：“-is” “-ion” “-iel” の接尾優先

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

- ■ 電導率（Conductivity）
範囲	音素傾向	音例
0.0〜0.3（絶縁）	ka, ta, mo, du / -on, -ar, -a	鈍い・土っぽい
0.3〜0.7（中）	lo, el, en, sa / -in, -al	標準的・液体寄り
0.7〜1.0（高導電）	ly, ele, ion, ex, sy / -is, -ion, -ex	光・電気的：「Lyion, Sylex」

- ■ 磁性（Magnetism）
範囲	音素傾向	音例
-1.0〜-0.3（負磁・S極）	柔音中心：syl, ne, lum, ae, vi	「Sylvae, Nelum」
-0.3〜+0.3（中性）	バランス型：ar, el, ra, mi	「Armel, Ralia」
+0.3〜+1.0（正磁・N極）	強音中心：pol, nor, mag, dr, kr	「Magnor, Polkran」

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

追加演出として：
temperatureが高ければ暖色寄り、低ければ寒色寄りに補正。
luminescenceが高ければ発光エフェクトを追加。

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
