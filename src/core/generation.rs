// src/core/generation.rs

use rand::Rng; // 乱数生成のために追加！
use noise::{OpenSimplex, NoiseFn}; // ✨ ノイズ生成のために追加！
use crate::core::world::{World, WIDTH, HEIGHT};
use crate::core::material::{Terrain, Overlay};

/// ワールド内の異なる環境を示すバイオーム。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Biome {
  Forest,   // 森林：草木が生い茂る標準的なバイオーム
  Desert,   // 砂漠：砂とサボテンが特徴の乾燥地帯
  Snowland, // 雪原：雪と氷に覆われた寒冷地帯
  // Plains,   // 草原：木が少なく開けた土地 (将来的に追加するかも？)
}

/// ワールドの初期地形を生成する。
///
/// # Arguments
/// * `world` - 地形を生成する対象の `World` インスタンスへの可変参照。
/// * `rng` - 地形生成に使用する、シード済みの乱数生成器への可変参照。
pub fn generate_initial_world(world: &mut World, rng: &mut impl Rng) {

  // --- 地表生成用のノイズ関数 ---
  // ノイズ関数のシードは、RNGから新しい乱数を取得して設定します。
  let surface_noise_seed: u32 = rng.random();
  let surface_noise = OpenSimplex::new(surface_noise_seed as u32);
  let surface_scale = 0.025; // 地表の起伏のスケール。小さいほど滑らか、大きいほどギザギザになるよ。 (0.02 ~ 0.03 がおすすめ)
  let surface_amplitude = 20.0; // 地表の起伏の最大高さ。この値の範囲で地表が上下するよ。
  let base_surface_y_level = (HEIGHT as f64 / 2.5).max(20.0); // 地表の基準Y座標。画面の上から1/3～半分くらいの高さが良いかな。

  // --- 洞窟生成用のノイズ関数を初期化 ---
  // ワールド生成シードから派生させたシードを使うと、ワールドごとに洞窟のパターンが変わるよ！
  // プライマリノイズ: 細かい洞窟の形状や通路を決定するよ。
  let primary_cave_noise_seed: u32 = rng.random();
  let primary_cave_noise = OpenSimplex::new(primary_cave_noise_seed as u32);
  let primary_cave_scale = 0.055; // 細かい洞窟形状用のスケール。(0.05～0.07くらいで調整してみてね)
  let cave_threshold_low = -0.15; // このノイズ値より大きく、かつ...
  let cave_threshold_high = 0.15; // このノイズ値より小さい範囲を洞窟にするよ。(値を近づけると洞窟が細くなる)

  // リージョンノイズ: 大きな洞窟群が生成される「領域」を決定するよ。
  // これにより、洞窟がワールド全体に均一に分布するのではなく、特定のエリアに集中するようになるんだ。
  let region_cave_noise_seed: u32 = rng.random();
  let region_cave_noise = OpenSimplex::new(region_cave_noise_seed as u32);
  // これらのスケールと閾値はバイオームごとに調整するよ！

  // --- 地層設定 ---
  // バイオームごとに地下の構成を変えるので、グローバルな地層オフセットは簡略化するか、
  // バイオームごとの設定に置き換えるよ。
  // ここでは、岩盤が始まるおおよその深さの目安だけ残しておくね。
  let rock_layer_start_depth_offset_min: i32 = 25;
  let rock_layer_start_depth_offset_max: i32 = 45;

  // --- バイオーム決定用のノイズ関数 ---
  // 温度と降水量のノイズを組み合わせてバイオームを決定するよ。
  // スケールを小さく（値を大きく）するとバイオームの境界が細かくなり、大きくすると広大なバイオームになる。
  let temperature_noise_seed: u32 = rng.random();
  let temperature_noise = OpenSimplex::new(temperature_noise_seed as u32);
  let temperature_scale = 0.007; // 温度変化のスケール (0.005 ~ 0.01 くらいがおすすめ)

  let precipitation_noise_seed: u32 = rng.random();
  let precipitation_noise = OpenSimplex::new(precipitation_noise_seed as u32);
  let precipitation_scale = 0.009; // 降水量変化のスケール (同上)

  // バイオーム判定の閾値 (これらの値は実験して調整してみてね！)
  let desert_temp_threshold = 0.25;    // この温度より「高く」、かつ
  let desert_precip_threshold = -0.3; // この降水量より「低い」と砂漠になりやすい
  let snow_temp_threshold = -0.35;   // この温度より「低い」と雪原になりやすい

  for x in 0..WIDTH {
    // --- 各列の地表の高さとバイオームを決定 ---
    let surface_offset_noise = surface_noise.get([
        x as f64 * surface_scale,
        rng.random_range(0.0..1.0) * 50.0 // X座標だけでなく、ランダムな値からも影響を受けるようにして、ワールドごとのパターンを増やすよ。
    ]);
    let surface_y_fluctuation = surface_offset_noise * surface_amplitude;
    let current_dirt_surface_y = (base_surface_y_level + surface_y_fluctuation)
                                  .round()
                                  .max(15.0) // 地表が画面上部に行き過ぎないように最低値を設定。
                                  .min((HEIGHT - 40) as f64) as usize; // 地表が画面下部に行き過ぎないように最大値を設定。

    // バイオーム決定: X座標とシードに基づいて温度と降水量のノイズ値を取得
    // Y座標は使わず、ワールドの水平方向の変化でバイオームが決まるようにするよ。
    let temp_val = temperature_noise.get([x as f64 * temperature_scale, rng.random_range(0.0..1.0) * 2.0]);
    let precip_val = precipitation_noise.get([x as f64 * precipitation_scale, rng.random_range(0.0..1.0) * 2.0 + 50.0]);

    let current_biome = if temp_val > desert_temp_threshold && precip_val < desert_precip_threshold {
      Biome::Desert
    } else if temp_val < snow_temp_threshold {
      Biome::Snowland
    } else {
      Biome::Forest // デフォルトは森林
    };

    // --- バイオームごとの設定 ---
    let rock_layer_start_y: usize; // このY座標より下は主に岩盤になる
    let mut current_primary_cave_scale = primary_cave_scale;
    let mut current_cave_threshold_low = cave_threshold_low;
    let mut current_cave_threshold_high = cave_threshold_high;
    let mut current_region_cave_scale = 0.02; // デフォルトのリージョンスケール
    let mut current_region_cave_influence_threshold = 0.05; // デフォルトのリージョン影響閾値

    match current_biome {
      Biome::Forest => {
        rock_layer_start_y = (current_dirt_surface_y as i32 + rng.random_range(rock_layer_start_depth_offset_min..rock_layer_start_depth_offset_max)) as usize;
        // 森林の洞窟は標準的
      }
      Biome::Desert => {
        rock_layer_start_y = (current_dirt_surface_y as i32 + rng.random_range(rock_layer_start_depth_offset_min - 5 .. rock_layer_start_depth_offset_max - 5).max(10)) as usize; // 砂漠は岩盤が少し浅めかも
        current_primary_cave_scale = 0.045; // 少し大きめの洞窟ができやすい
        current_cave_threshold_low = -0.12;
        current_cave_threshold_high = 0.12;
        current_region_cave_influence_threshold = 0.02; // 洞窟ができやすい
      }
      Biome::Snowland => {
        rock_layer_start_y = (current_dirt_surface_y as i32 + rng.random_range(rock_layer_start_depth_offset_min + 5 .. rock_layer_start_depth_offset_max + 5)) as usize; // 雪原は岩盤が少し深めかも
        current_primary_cave_scale = 0.065; // やや細かく、氷柱のような洞窟をイメージ
        current_cave_threshold_low = -0.18;
        current_cave_threshold_high = 0.18;
        current_region_cave_scale = 0.025;
        current_region_cave_influence_threshold = 0.08; // 洞窟はややまばら
      }
    }

    for y in 0..HEIGHT {
      if y < current_dirt_surface_y {
        // 地表より上は空
        world.set_terrain(x, y, Terrain::Empty);
        world.set_overlay(x, y, Overlay::Air);
      } else {
        // --- バイオームと深さに基づいた地形タイプの決定 ---
        let terrain_roll: f64 = rng.random(); // 0.0 から 1.0 の乱数を生成。
        let mut determined_terrain;
        let mut determined_overlay = Overlay::Air; // これから決定するオーバーレイタイプ、デフォルトは空気

        if y >= rock_layer_start_y { // 岩盤層
          // 岩盤層内の地形決定。深さとバイオームの影響をブレンドするよ！

          // バイオームの影響度を計算 (表層のブレンドロジックで使った値を再利用、またはここで再計算)
          // ここでは、X座標に基づくtemp_valとprecip_valを再度利用する
          let desert_influence_rock = (temp_val - desert_temp_threshold).max(0.0) + (-precip_val - (-desert_precip_threshold)).max(0.0);
          let snow_influence_rock = (-temp_val - (-snow_temp_threshold)).max(0.0);
          let forest_influence_rock = 1.0 - desert_influence_rock.max(snow_influence_rock).min(1.0);
          let total_influence_rock = forest_influence_rock + desert_influence_rock + snow_influence_rock;
          let forest_weight_rock = if total_influence_rock > 0.0 { forest_influence_rock / total_influence_rock } else { 1.0 };
          let desert_weight_rock = if total_influence_rock > 0.0 { desert_influence_rock / total_influence_rock } else { 0.0 };
          let snow_weight_rock = if total_influence_rock > 0.0 { snow_influence_rock / total_influence_rock } else { 0.0 };

          // 深さによる影響度 (岩盤層内での変化)
          let depth_in_rock_layer = (y - rock_layer_start_y) as f64;
          // これらのオフセット値は、ワールドの深さや好みに応じて調整してね！
          let deep_rock_zone_start_offset = (rng.random_range(25..50)) as f64; // 「深い岩盤層」が始まるオフセット
          let deepest_rock_zone_start_offset = (rng.random_range(70..100)) as f64; // 「最深岩盤層」が始まるオフセット

          if depth_in_rock_layer > deepest_rock_zone_start_offset { // 最深岩盤層 (Obsidian, Gold, Rock)
            // 最深層はObsidianやGoldが出やすく、基本はRock
            let rock_prob = 0.65 - desert_weight_rock * 0.1; // 砂漠の地下は少し岩が減るかも
            let obsidian_prob = 0.25 + desert_weight_rock * 0.1; // 砂漠の地下は黒曜石が増えるイメージ
            let gold_prob = 0.1;

            if terrain_roll < rock_prob { determined_terrain = Terrain::Rock; }
            else if terrain_roll < rock_prob + obsidian_prob { determined_terrain = Terrain::Obsidian; }
            else { determined_terrain = Terrain::Gold; }

          } else if depth_in_rock_layer > deep_rock_zone_start_offset { // 深い岩盤層 (Iron, Coal, Rock, Biome specific)
            // 深い岩盤層はIronやCoalが出やすく、基本はRock。バイオームの影響も少し。
            let rock_prob = 0.75;
            let iron_prob = 0.12;
            let coal_prob = 0.08;
            let hardice_prob_deep = snow_weight_rock * 0.05; // 深い雪原岩盤にHardIceが少し

            if terrain_roll < rock_prob { determined_terrain = Terrain::Rock; }
            else if terrain_roll < rock_prob + iron_prob { determined_terrain = Terrain::Iron; }
            else if terrain_roll < rock_prob + iron_prob + coal_prob { determined_terrain = Terrain::Coal; }
            else if terrain_roll < rock_prob + iron_prob + coal_prob + hardice_prob_deep { determined_terrain = Terrain::HardIce; }
            else { determined_terrain = Terrain::Rock; } // 残りは岩盤

          } else { // 通常の岩盤層 (rock_layer_start_y から deep_rock_zone_start_offset まで)
            // 通常の岩盤層はCopperやCoalが出やすく、基本はRock。バイオームの影響をより強く受ける。
            let base_rock_prob = 0.65;
            let hardice_prob_normal = snow_weight_rock * 0.15; // 雪原の岩盤にはHardIceが出やすい
            let sand_prob_normal = desert_weight_rock * 0.1;  // 砂漠の岩盤にはSandが混じりやすい
            let dirt_prob_normal = forest_weight_rock * 0.05; // 森林の岩盤にはDirtが少し混じる
            let copper_prob_normal = 0.08;
            let coal_prob_normal = 0.05;

            if terrain_roll < base_rock_prob { determined_terrain = Terrain::Rock; }
            else if terrain_roll < base_rock_prob + hardice_prob_normal { determined_terrain = Terrain::HardIce; }
            else if terrain_roll < base_rock_prob + hardice_prob_normal + sand_prob_normal { determined_terrain = Terrain::Sand; }
            else if terrain_roll < base_rock_prob + hardice_prob_normal + sand_prob_normal + dirt_prob_normal { determined_terrain = Terrain::Dirt; }
            else if terrain_roll < base_rock_prob + hardice_prob_normal + sand_prob_normal + dirt_prob_normal + copper_prob_normal { determined_terrain = Terrain::Copper; }
            else if terrain_roll < base_rock_prob + hardice_prob_normal + sand_prob_normal + dirt_prob_normal + copper_prob_normal + coal_prob_normal { determined_terrain = Terrain::Coal; }
            else { determined_terrain = Terrain::Rock; } // 残りは岩盤
          }
        } else { // 表層～中間層
          // バイオームの境界を滑らかにするため、Y座標とノイズ値に基づいて地形をブレンドする
          // 各バイオームの理想的な地形構成を定義し、現在のノイズ値に応じてそれらを混ぜ合わせるイメージ
          // ここではシンプルに、ノイズ値が閾値に近いほど、対応するバイオームの地形が出やすくなるように調整する

          // 温度と降水量のノイズ値を0～1の範囲にスケーリング
          let temp_factor = (temp_val + 1.0) / 2.0; // -1..1 -> 0..1
          let precip_factor = (precip_val + 1.0) / 2.0; // -1..1 -> 0..1

          // 各バイオームの「らしさ」をノイズ値から計算 (単純な線形補間やスムーズステップ関数などを使うとより滑らかになる)
          // ここでは簡単のため、閾値からの距離で影響度を調整する
          let desert_influence = (temp_val - desert_temp_threshold).max(0.0) + (-precip_val - (-desert_precip_threshold)).max(0.0);
          let snow_influence = (-temp_val - (-snow_temp_threshold)).max(0.0);
          // Forestの影響度は、DesertとSnowlandの影響度が低いほど高くなるように調整
          let forest_influence = 1.0 - desert_influence.max(snow_influence).min(1.0); // 影響度の合計が1を超える場合があるのでmin(1.0)でクランプ

          // 影響度に基づいて地形の出現確率を調整
          // 例: DirtはForestで出やすく、SandはDesertで出やすく、Snow/HardIce/Overlay::IceはSnowlandで出やすい
          let total_influence = forest_influence + desert_influence + snow_influence;
          let forest_weight = if total_influence > 0.0 { forest_influence / total_influence } else { 1.0 };
          let desert_weight = if total_influence > 0.0 { desert_influence / total_influence } else { 0.0 };
          let snow_weight = if total_influence > 0.0 { snow_influence / total_influence } else { 0.0 };

          // 乱数と重みに基づいて地形を決定
          // 各地形タイプの「基本確率」を定義し、それをバイオームの重みで調整する
          // ここではシンプルに、乱数範囲をバイオームの重みで分割して決定する
          if terrain_roll < forest_weight * 0.8 + desert_weight * 0.2 + snow_weight * 0.1 {
              determined_terrain = Terrain::Dirt;
              if snow_weight > 0.3 && rng.random_bool(snow_weight * 0.5) { determined_overlay = Overlay::Ice; } // 雪原の影響が強ければ凍った土に
          } else if terrain_roll < forest_weight * 0.9 + desert_weight * 0.7 + snow_weight * 0.2 {
              determined_terrain = Terrain::Sand;
          } else if terrain_roll < forest_weight * 0.95 + desert_weight * 0.8 + snow_weight * 0.6 {
                determined_terrain = Terrain::HardIce;
          } else { // それ以外はRockや鉱石など (バイオームの影響は小さめ)
                determined_terrain = Terrain::Rock; // たまに岩が混じる
                if rng.random_bool(0.05) { determined_terrain = Terrain::Coal; } // 鉱石も少し
          }
        }


        // バイオームによる地表ブロックの最終調整 (これは以前のロジックを流用)
        if y == current_dirt_surface_y {
          match current_biome {
            Biome::Forest => determined_terrain = Terrain::Dirt,
            Biome::Desert => determined_terrain = Terrain::Sand,
            Biome::Snowland => determined_terrain = Terrain::Dirt,
          } // 地表のオーバーレイは一旦Airに (草や積雪は後の処理で上書き)
          determined_overlay = Overlay::Air;
        };

        // バイオームによる地表および表層の調整
        if y == current_dirt_surface_y { // 地表ブロック
          match current_biome {
            Biome::Forest => determined_terrain = Terrain::Dirt, // 森林の地表は土 (草が生える基盤) (上の分岐で設定済みだが念のため)
            Biome::Desert => determined_terrain = Terrain::Sand, // 砂漠の地表は砂
            Biome::Snowland => determined_terrain = Terrain::Dirt, // 雪原の地表は雪ブロック (これは上の分岐で設定済みなので重複してるかも。後で整理)
            // determined_overlay はここでは変更せず、地表ならAirのまま
          }
        } else if y > current_dirt_surface_y && y < current_dirt_surface_y + rng.random_range(3..6) { // 地表から数ブロック下の調整
          match current_biome { // このブロックは、より詳細な表層の調整に使えそう。雪原の表層も雪や氷っぽくする？
            Biome::Desert => { // 砂漠なら、表層も砂で置き換える確率を上げる
              if rng.random_bool(0.75) { determined_terrain = Terrain::Sand; }
            }
            Biome::Snowland => { // 雪原なら、表層が土の場合、一部を雪や氷（まだないけど）っぽくしても面白いかも
              if determined_terrain == Terrain::Dirt && rng.random_bool(0.3) {
                // ここで Overlay::Ice を設定しても良いが、「凍った土」の表現は上の分岐で対応済みなので不要
              }
            }
            _ => {} // 森林は特に変更なし
          }
        }

        world.set_terrain(x, y, determined_terrain);
        world.set_overlay(x, y, determined_overlay); // 決定されたオーバーレイを設定。水や洞窟は後で上書きするよ。

        // --- 地下水脈の生成 (洞窟とは別に、土や砂の層に) ---
        // 地形が水を通すタイプで、かつ地表から少し離れた場所なら、低確率で水源を生成するよ。
        if determined_terrain.allows_fluid_passthrough() && determined_overlay == Overlay::Air && y > current_dirt_surface_y + rng.random_range(3..8) {
            if rng.random_bool(0.0015) { // 0.15% の確率で水源を設置！
                world.set_overlay(x, y, Overlay::Water);
            }
        }

        // --- 洞窟を生成 ---
        // 地形とオーバーレイを設定した後で、洞窟を掘るか判定するよ。
        // 地表からある程度深く、かつ「洞窟領域ノイズ」が閾値を超えている場合にのみ、洞窟生成を試みるよ。
        let min_cave_depth_from_surface = rng.random_range(8..15); // 地表から最低でもこの深さから洞窟が始まるようにする。
        let cave_start_y = current_dirt_surface_y + min_cave_depth_from_surface;

        if y >= cave_start_y {
          // 洞窟領域ノイズで、洞窟ができやすい/できにくい大きな領域を作るんだ。
          let region_noise_val = region_cave_noise.get([
            x as f64 * current_region_cave_scale, // バイオームごとのスケールを使用
            y as f64 * current_region_cave_scale,
          ]);

          // リージョンノイズが閾値より大きい場合のみ、細かい洞窟生成を試みる。
          if region_noise_val > current_region_cave_influence_threshold { // バイオームごとの閾値を使用
            let primary_noise_val = primary_cave_noise.get([
              x as f64 * current_primary_cave_scale, // バイオームごとのスケールを使用
              y as f64 * current_primary_cave_scale,
            ]);

            // プライマリノイズが特定の範囲内なら、そこを洞窟にする！
            if primary_noise_val > current_cave_threshold_low && primary_noise_val < current_cave_threshold_high { // バイオームごとの閾値を使用
              world.set_terrain(x, y, Terrain::Empty);
              if rng.random_bool(0.008) { // 洞窟内にごく稀に水源を設置。
                world.set_overlay(x, y, Overlay::Water);
              } else {
                world.set_overlay(x, y, Overlay::Air); // 基本は空気。
              }
            }
          }
        }
      }
    }
  }

  // --- バイオームに応じたオーバーレイの配置 (草、木、雪など) ---
  // 地形生成が終わった後に、各バイオーム特有の装飾を配置するよ！
  for x in 0..WIDTH {
    // 地表の高さを再計算 (ループ内で計算済みだけど、念のため)
    let surface_offset_noise = surface_noise.get([x as f64 * surface_scale, rng.random_range(0.0..1.0) * 50.0]);
    let surface_y_fluctuation = surface_offset_noise * surface_amplitude;
    let current_surface_y = (base_surface_y_level + surface_y_fluctuation).round().max(15.0).min((HEIGHT - 40) as f64) as usize;

    // バイオームを再決定
    let temp_val = temperature_noise.get([x as f64 * temperature_scale, rng.random_range(0.0..1.0) * 2.0]);
    let precip_val = precipitation_noise.get([x as f64 * precipitation_scale, rng.random_range(0.0..1.0) * 2.0 + 50.0]);
    let biome_at_x = if temp_val > desert_temp_threshold && precip_val < desert_precip_threshold { Biome::Desert }
                     else if temp_val < snow_temp_threshold { Biome::Snowland }
                     else { Biome::Forest };

    if current_surface_y > 0 { // 地表がワールド最上部でなければ
      let y_on_surface = current_surface_y -1; // 地表のすぐ上のY座標
      match biome_at_x {
        Biome::Forest => {
          // 地表のタイルを取得
          if let Some(surface_tile) = world.get(x, current_surface_y) {
            // 地表が土 (Dirt) であることを確認
            if surface_tile.terrain == Terrain::Dirt {
              // 地表のすぐ上のオーバーレイを取得
              if let Some(overlay_above) = world.get_overlay(x, y_on_surface) {
                // オーバーレイが空気 (Air) であることを確認
                if *overlay_above == Overlay::Air {
                  if rng.random_bool(0.7) { world.set_overlay(x, y_on_surface, Overlay::Grass); } // 高確率で草
                  if rng.random_bool(0.15) { world.set_overlay(x, y_on_surface, Overlay::Tree); } // たまに木
                }
              }
            }
          }
        }
        Biome::Desert => {
          // 地表のタイルを取得
          if let Some(surface_tile) = world.get(x, current_surface_y) {
            // 地表が砂 (Sand) であることを確認
            if surface_tile.terrain == Terrain::Sand {
              // 地表のすぐ上のオーバーレイを取得し、それが空気 (Air) ならサボテンを試みる
              if world.get_overlay(x, y_on_surface) == Some(&Overlay::Air) && rng.random_bool(0.03) {
                world.set_overlay(x, y_on_surface, Overlay::Tree); // サボテンのつもり (Overlay::Treeを流用)
              }
            }
          }
        }
        Biome::Snowland => {
          // 地表のタイルを取得
          if let Some(surface_tile) = world.get(x, current_surface_y) {
            // 地表が雪 (Snow) であることを確認
            if surface_tile.terrain == Terrain::Dirt {
              // 地表のすぐ上のオーバーレイを取得し、それが空気 (Air) なら積雪や針葉樹を試みる
              if world.get_overlay(x, y_on_surface) == Some(&Overlay::Air) {
                if rng.random_bool(0.4) { world.set_overlay(x, y_on_surface, Overlay::Snow); } // 地表の雪ブロックの上にさらに積もった雪
                if rng.random_bool(0.02) { world.set_overlay(x, y_on_surface, Overlay::Tree); } // 雪原にも稀に針葉樹
              }
            }
          }
        }
      }
    }
  }
}
