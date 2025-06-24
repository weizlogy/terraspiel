//! src/core/generation.rs

use crate::core::rng::GameRngMethods; // オブジェクト安全な乱数生成器メソッドトレイトをインポート
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex}; // ノイズ生成と設定用のトレイトをインポート
use crate::core::world::{World, WIDTH, HEIGHT};
use crate::core::material::{Terrain, Overlay};
use crate::core::biomes::{self, Biome, BiomeGenerator}; // biomes モジュールと関連アイテムをインポート

// --- バイオームのパラメータとウェイトに関するデータ構造 ---

/// `determine_tile` メソッドに渡すパラメータをまとめた構造体。
pub struct DetermineTileParameters<'a> {
  pub x: usize,
  pub y: usize,
  pub surface_y: usize,
  pub rock_layer_y: usize,
  pub ore_noise: OreNoiseValues,
  pub biome_weights: &'a BiomeWeights,
}

/// バイオームのパラメータを保持する構造体。
pub struct BiomeParameters {
  pub rock_layer_y: usize,
  // 他にも洞窟パラメータなどを追加できます
}

/// 各種鉱石の生成判定に使うノイズ値をまとめた構造体。
pub struct OreNoiseValues {
  pub coal: f64,
  pub iron: f64,
}

/// バイオームの影響度を保持する構造体。
pub struct BiomeWeights {
  pub forest: f64,
  pub desert: f64,
  pub plains: f64,
  pub snow: f64,
}

/// `Biome` enum の値に応じて、対応する `BiomeGenerator` のインスタンスを生成するファクトリー関数。
///
/// `Box<dyn Trait>` を使うことで、実行時にどのバイオーム戦略を利用するかを決定できます。
pub fn create_biome_generator(biome: Biome) -> Box<dyn BiomeGenerator> {
  match biome {
    Biome::Forest => Box::new(biomes::forest::ForestBiome),
    Biome::Desert => Box::new(biomes::desert::DesertBiome),
    Biome::Plains => Box::new(biomes::plains::PlainsBiome),
    Biome::Snowland => Box::new(biomes::snowland::SnowlandBiome),
  }
}


/// ワールドの初期地形を生成する。
///
/// # Arguments
/// * `world` - 地形を生成する対象の `World` インスタンスへの可変参照。
/// * `rng` - 地形生成に使用する、シード済みの乱数生成器への可変参照。
pub fn generate_initial_world(world: &mut World, rng: &mut dyn GameRngMethods) {

  // --- 地表生成用のノイズ関数 ---
  // ノイズ関数のシードは、RNGから新しい乱数を取得して設定します。
  let surface_noise_seed: u32 = rng.next_u32();
  let surface_noise = OpenSimplex::new(surface_noise_seed as u32);
  let surface_scale = 0.025; // 地表の起伏のスケール。小さいほど滑らか、大きいほどギザギザになるよ。 (0.02 ~ 0.03 がおすすめ)
  let surface_amplitude = 20.0; // 地表の起伏の最大高さ。この値の範囲で地表が上下するよ。
  let base_surface_y_level = (HEIGHT as f64 / 2.5).max(20.0); // 地表の基準Y座標。画面の上から1/3～半分くらいの高さが良いかな。

  // --- 洞窟生成用のノイズ関数を初期化 ---
  // ワールド生成シードから派生させたシードを使うと、ワールドごとに洞窟のパターンが変わるよ！
  // プライマリノイズ: 細かい洞窟の形状や通路を決定するよ。
  let primary_cave_noise_seed: u32 = rng.next_u32();
  let primary_cave_noise = Fbm::<OpenSimplex>::new(primary_cave_noise_seed as u32)
    .set_octaves(4) // 4つのノイズレイヤーを重ねて複雑さを出すよ
    .set_persistence(0.5) // 各レイヤーの寄与度。0.5が一般的
    .set_lacunarity(2.0); // 各レイヤーの周波数増加率。2.0が一般的

  // リージョンノイズ: 大きな洞窟群が生成される「領域」を決定するよ。
  // これにより、洞窟がワールド全体に均一に分布するのではなく、特定のエリアに集中するようになるんだ。
  let region_cave_noise_seed: u32 = rng.next_u32();
  let region_cave_noise = Fbm::<OpenSimplex>::new(region_cave_noise_seed as u32)
    .set_octaves(3) // 領域は少し滑らかにするため、オクターブを少なめに
    .set_persistence(0.6) // 領域のコントラストを少し強めに
    .set_lacunarity(2.0);

  // --- 水脈・溶岩溜まり生成用のノイズ関数 ---
  let water_noise_seed: u32 = rng.next_u32();
  let water_noise = Fbm::<OpenSimplex>::new(water_noise_seed)
    .set_octaves(3) // 水脈は少し複雑な形状に
    .set_persistence(0.5);

  let lava_noise_seed: u32 = rng.next_u32();
  let lava_noise = Fbm::<OpenSimplex>::new(lava_noise_seed)
    .set_octaves(2) // 溶岩溜まりはより大きく滑らかな形状に
    .set_persistence(0.6); // コントラストを少し強めに


  // --- 鉱脈生成用のノイズ関数 ---
  let coal_noise_seed: u32 = rng.next_u32();
  let coal_noise = Fbm::<OpenSimplex>::new(coal_noise_seed)
    .set_octaves(5) // 石炭はより大きな塊になりやすいように
    .set_persistence(0.5);
  let coal_scale = 0.08;

  let iron_noise_seed: u32 = rng.next_u32();
  let iron_noise = Fbm::<OpenSimplex>::new(iron_noise_seed)
    .set_octaves(6) // 鉄鉱石は少し細かく、複雑な鉱脈に
    .set_persistence(0.6);
  let iron_scale = 0.12;


  // --- バイオーム決定用のノイズ関数 ---
  // 温度と降水量のノイズを組み合わせてバイオームを決定するよ。
  // スケールを小さく（値を大きく）するとバイオームの境界が細かくなり、大きくすると広大なバイオームになる。
  let temperature_noise_seed: u32 = rng.next_u32();
  let temperature_noise = OpenSimplex::new(temperature_noise_seed as u32);
  let temperature_scale = 0.007; // 温度変化のスケール (0.005 ~ 0.01 くらいがおすすめ)

  let precipitation_noise_seed: u32 = rng.next_u32();
  let precipitation_noise = OpenSimplex::new(precipitation_noise_seed as u32);
  let precipitation_scale = 0.009; // 降水量変化のスケール (同上)

  // バイオーム判定の閾値 (これらの値は実験して調整してみてね！)
  let desert_temp_threshold = 0.25;    // この温度より「高く」、かつ
  let desert_precip_threshold = -0.3; // この降水量より「低い」と砂漠になりやすい
  let snow_temp_threshold = -0.35;   // この温度より「低い」と雪原になりやすい
  let plains_precip_threshold = 0.1;   // この降水量より「低い」と平原になりやすい（砂漠ほどではない）

  for x in 0..WIDTH {
    // --- 各列の地表の高さとバイオームを決定 ---
    let surface_offset_noise = surface_noise.get([
        x as f64 * surface_scale,
          rng.gen_f64_range(0.0, 1.0) * 50.0 // X座標だけでなく、ランダムな値からも影響を受けるようにして、ワールドごとのパターンを増やすよ。
    ]);
    let surface_y_fluctuation = surface_offset_noise * surface_amplitude;
    let current_dirt_surface_y = (base_surface_y_level + surface_y_fluctuation)
                                  .round()
                                  .max(15.0) // 地表が画面上部に行き過ぎないように最低値を設定。
                                  .min((HEIGHT - 40) as f64) as usize; // 地表が画面下部に行き過ぎないように最大値を設定。

    // バイオーム決定: X座標とシードに基づいて温度と降水量のノイズ値を取得
    // Y座標は使わず、ワールドの水平方向の変化でバイオームが決まるようにするよ。
      let temp_val = temperature_noise.get([x as f64 * temperature_scale, rng.gen_f64_range(0.0, 1.0) * 2.0]);
      let precip_val = precipitation_noise.get([x as f64 * precipitation_scale, rng.gen_f64_range(0.0, 1.0) * 2.0 + 50.0]);

    let current_biome = if temp_val > desert_temp_threshold && precip_val < desert_precip_threshold {
      Biome::Desert
    } else if temp_val < snow_temp_threshold {
      Biome::Snowland
    } else if precip_val < plains_precip_threshold {
      Biome::Plains
    } else {
      Biome::Forest // デフォルトは森林
    };

    // --- ストラテジーパターンとファクトリーの活用 + バイオームのブレンド ---
    // ファクトリーから現在のバイオームに対応するジェネレータ（戦略）を取得
    let biome_generator = create_biome_generator(current_biome);
    // 取得したジェネレータから、バイオーム固有のパラメータを取得
    let biome_params = biome_generator.generate_biome_parameters(current_dirt_surface_y, rng);

    // バイオームのブレンド用ウェイトを計算
    let desert_influence = (temp_val - desert_temp_threshold).max(0.0) + (-precip_val - (-desert_precip_threshold)).max(0.0);
    let snow_influence = (-temp_val - (-snow_temp_threshold)).max(0.0);
    let plains_influence = (plains_precip_threshold - precip_val).max(0.0) * (1.0 - desert_influence); // 砂漠の影響を打ち消す
    let forest_influence = 1.0 - desert_influence.max(snow_influence).max(plains_influence).min(1.0);
    let total_influence = forest_influence + desert_influence + snow_influence + plains_influence;
    let biome_weights = BiomeWeights {
      forest: if total_influence > 0.0 { forest_influence / total_influence } else { 1.0 },
      desert: (if total_influence > 0.0 { desert_influence / total_influence } else { 0.0 }) * (1.0 - plains_influence), // 平原の影響を打ち消す
      plains: if total_influence > 0.0 { plains_influence / total_influence } else { 0.0 },
      snow: if total_influence > 0.0 { snow_influence / total_influence } else { 0.0 },
    };

    for y in 0..HEIGHT {
      if y < current_dirt_surface_y {
        // 地表より上は空
        world.set_terrain(x, y, Terrain::Empty);
        world.set_overlay(x, y, Overlay::Air);
      } else {
        // --- 鉱石ノイズの計算 ---
        let ore_noise_values = OreNoiseValues {
          coal: coal_noise.get([x as f64 * coal_scale, y as f64 * coal_scale]),
          iron: iron_noise.get([x as f64 * iron_scale, y as f64 * iron_scale]),
        };

        // バイオームジェネレータにタイルの種類を決定してもらう
        let tile_params = DetermineTileParameters {
          x, y,
          surface_y: current_dirt_surface_y,
          rock_layer_y: biome_params.rock_layer_y,
          ore_noise: ore_noise_values,
          biome_weights: &biome_weights,
        };
        let (determined_terrain, determined_overlay) = biome_generator.determine_tile(&tile_params, rng);

        world.set_terrain(x, y, determined_terrain);
        world.set_overlay(x, y, determined_overlay); // 決定されたオーバーレイを設定。水や洞窟は後で上書きするよ。

        // --- 洞窟生成 ---
        // 2つのノイズを組み合わせて、より自然な洞窟を生成するよ。
        let cave_region_scale = 0.015; // 洞窟が存在する「領域」のスケールを大きくして、より広範囲に洞窟を分布させる
        let primary_cave_scale = 0.04; // 洞窟自体の形状のスケールを小さくして、より滑らかな通路を作る

        // 洞窟領域ノイズの値を取得。この値が大きいほど、洞窟が生成されやすくなる。
        let region_val = region_cave_noise.get([x as f64 * cave_region_scale, y as f64 * cave_region_scale]);
        // プライマリノイズの値を取得。これが洞窟の壁になる。
        let primary_val = primary_cave_noise.get([x as f64 * primary_cave_scale, y as f64 * primary_cave_scale]);

        // 洞窟生成の閾値。region_val が高い（洞窟地帯の中心に近い）ほど、この閾値は下がり、洞窟ができやすくなる。
        let cave_threshold = 0.5 - (region_val * 0.4);

        // プライマリノイズの値が閾値を超えていたら、そこは洞窟（空洞）にする。
        // ただし、地表近くには洞窟ができないように、ある程度の深さ(surface_y + 10)から判定する。
        if y > current_dirt_surface_y + 10 && primary_val.abs() > cave_threshold {
          // まず地形を空洞にし、オーバーレイを空気で初期化する。
          world.set_terrain(x, y, Terrain::Empty);
          world.set_overlay(x, y, Overlay::Air);

          // --- 洞窟内の液体生成 ---
          // どの液体を生成するかは、各バイオームの戦略に委譲する
          let water_scale = 0.05;
          let water_val = water_noise.get([x as f64 * water_scale, y as f64 * water_scale]);
          let lava_scale = 0.07;
          let lava_val = lava_noise.get([x as f64 * lava_scale, y as f64 * lava_scale]);

          // バイオームジェネレータに液体の配置を依頼
          biome_generator.place_fluids_in_cave(world, &tile_params, water_val, lava_val);
        }
      }
    }
  }

  // --- バイオームに応じたオーバーレイの配置 (草、木、雪など) ---
  // 地形生成が終わった後に、各バイオーム特有の装飾を配置するよ！
  for x in 0..WIDTH {
    // 地表の高さを再計算 (ループ内で計算済みだけど、念のため)
      let surface_offset_noise = surface_noise.get([x as f64 * surface_scale, rng.gen_f64_range(0.0, 1.0) * 50.0]);
    let surface_y_fluctuation = surface_offset_noise * surface_amplitude;
    let current_surface_y = (base_surface_y_level + surface_y_fluctuation).round().max(15.0).min((HEIGHT - 40) as f64) as usize;

    // バイオームを再決定
      let temp_val = temperature_noise.get([x as f64 * temperature_scale, rng.gen_f64_range(0.0, 1.0) * 2.0]);
      let precip_val = precipitation_noise.get([x as f64 * precipitation_scale, rng.gen_f64_range(0.0, 1.0) * 2.0 + 50.0]);
    let biome_at_x = if temp_val > desert_temp_threshold && precip_val < desert_precip_threshold { Biome::Desert }
                     else if temp_val < snow_temp_threshold { Biome::Snowland }
                     else if precip_val < plains_precip_threshold { Biome::Plains }
                     else { Biome::Forest };

    // ファクトリーからジェネレータを取得
    let biome_generator = create_biome_generator(biome_at_x);
    // ジェネレータに地表の装飾を配置してもらう
    biome_generator.place_surface_decorations(world, x, current_surface_y, rng);
  }
}
