use crate::material::{MaterialDNA, State};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

// --- Phoneme Sets ---

// State-based phonemes
const SOLID_PREFIX: &[&str] = &["gr", "kr", "st", "dr", "br"];
const SOLID_ROOT: &[&str] = &["ar", "or", "an", "ul", "tar", "dor"];
const SOLID_SUFFIX: &[&str] = &["-on", "-ar", "-ite", "-orn"];

const LIQUID_PREFIX: &[&str] = &["el", "lo", "va", "mi", "sa"];
const LIQUID_ROOT: &[&str] = &["ae", "al", "ir", "ol", "ra", "lia"];
const LIQUID_SUFFIX: &[&str] = &["-ine", "-el", "-ra", "-al"];

const GAS_PREFIX: &[&str] = &["ae", "pha", "is", "sy", "lu"];
const GAS_ROOT: &[&str] = &["el", "ar", "ia", "es", "the", "ion"];
const GAS_SUFFIX: &[&str] = &["-is", "-os", "-ion", "-eth"];

const PARTICLE_PREFIX: &[&str] = &["mu", "fi", "sha", "ki", "to"];
const PARTICLE_ROOT: &[&str] = &["ra", "en", "ta", "il", "mi"];
const PARTICLE_SUFFIX: &[&str] = &["-a", "-en", "-um", "-ir"];

// Temperature-based phonemes
const TEMP_COLD: &[&str] = &["cr", "sil", "el", "is", "fr", "ne"];
const TEMP_COLD_SUFFIX: &[&str] = &["-el", "-ine", "-is"];
const TEMP_NEUTRAL: &[&str] = &["al", "er", "an", "ol", "mi"];
const TEMP_NEUTRAL_SUFFIX: &[&str] = &["-ar", "-en"];
const TEMP_HOT: &[&str] = &["ra", "fi", "or", "py", "th", "an"];
const TEMP_HOT_SUFFIX: &[&str] = &["-or", "-as", "-ar"];

// Conductivity-based phonemes
const COND_LOW: &[&str] = &["ka", "ta", "mo", "du"];
const COND_LOW_SUFFIX: &[&str] = &["-on", "-ar", "-a"];
const COND_MID: &[&str] = &["lo", "el", "en", "sa"];
const COND_MID_SUFFIX: &[&str] = &["-in", "-al"];
const COND_HIGH: &[&str] = &["ly", "ele", "ion", "ex", "sy"];
const COND_HIGH_SUFFIX: &[&str] = &["-is", "-ion", "-ex"];

// Magnetism-based phonemes
const MAG_NEG: &[&str] = &["syl", "ne", "lum", "ae", "vi"];
const MAG_NEUTRAL: &[&str] = &["ar", "el", "ra", "mi"];
const MAG_POS: &[&str] = &["pol", "nor", "mag", "dr", "kr"];

// Luminescence-based phonemes
const LUM_DARK: &[&str] = &["mor", "dul", "tar", "ol"];
const LUM_DARK_SUFFIX: &[&str] = &["-ar", "-um"];
const LUM_MID: &[&str] = &["el", "la", "mi", "ae"];
const LUM_MID_SUFFIX: &[&str] = &["-en", "-al"];
const LUM_BRIGHT: &[&str] = &["ael", "lux", "the", "ion", "syl"];
const LUM_BRIGHT_SUFFIX: &[&str] = &["-is", "-iel", "-ae"];

// Viscosity-based phonemes
const VISC_LOW: &[&str] = &["ki", "ra", "to"];
const VISC_MID: &[&str] = &["el", "la", "sa", "mi"];
const VISC_HIGH: &[&str] = &["gr", "dr", "mu", "ul"];

// Hardness-based phonemes
const HARD_SOFT: &[&str] = &["li", "ne", "sa", "el"];
const HARD_SOFT_SUFFIX: &[&str] = &["-a", "-in"];
const HARD_MID: &[&str] = &["ar", "en", "ol"];
const HARD_MID_SUFFIX: &[&str] = &["-en", "-ar"];
const HARD_HARD: &[&str] = &["kr", "gr", "st", "dor"];
const HARD_HARD_SUFFIX: &[&str] = &["-ite", "-orn"];

// --- Helper Functions ---

fn choose<T>(rng: &mut impl Rng, slice: &[T]) -> T
where
    T: Clone,
{
    slice.choose(rng).unwrap().clone()
}

fn get_phonemes_for_state(state: State) -> (&'static [&'static str], &'static [&'static str], &'static [&'static str]) {
    match state {
        State::Solid => (SOLID_PREFIX, SOLID_ROOT, SOLID_SUFFIX),
        State::Liquid => (LIQUID_PREFIX, LIQUID_ROOT, LIQUID_SUFFIX),
        State::Gas => (GAS_PREFIX, GAS_ROOT, GAS_SUFFIX),
        State::Particle => (PARTICLE_PREFIX, PARTICLE_ROOT, PARTICLE_SUFFIX),
    }
}

fn get_phonemes_for_temperature(temp: f32) -> (&'static [&'static str], &'static [&'static str]) {
    if temp < -0.3 {
        (TEMP_COLD, TEMP_COLD_SUFFIX)
    } else if temp > 0.3 {
        (TEMP_HOT, TEMP_HOT_SUFFIX)
    } else {
        (TEMP_NEUTRAL, TEMP_NEUTRAL_SUFFIX)
    }
}

fn get_phonemes_for_conductivity(cond: f32) -> (&'static [&'static str], &'static [&'static str]) {
    if cond < 0.3 {
        (COND_LOW, COND_LOW_SUFFIX)
    } else if cond > 0.7 {
        (COND_HIGH, COND_HIGH_SUFFIX)
    } else {
        (COND_MID, COND_MID_SUFFIX)
    }
}

fn get_phonemes_for_magnetism(mag: f32) -> &'static [&'static str] {
    if mag < -0.3 {
        MAG_NEG
    } else if mag > 0.3 {
        MAG_POS
    } else {
        MAG_NEUTRAL
    }
}

fn get_phonemes_for_luminescence(lum: f32) -> (&'static [&'static str], &'static [&'static str]) {
    if lum < 0.3 {
        (LUM_DARK, LUM_DARK_SUFFIX)
    } else if lum > 0.7 {
        (LUM_BRIGHT, LUM_BRIGHT_SUFFIX)
    } else {
        (LUM_MID, LUM_MID_SUFFIX)
    }
}

fn get_phonemes_for_viscosity(visc: f32) -> &'static [&'static str] {
    if visc < 0.4 {
        VISC_LOW
    } else if visc > 0.7 {
        VISC_HIGH
    } else {
        VISC_MID
    }
}

fn get_phonemes_for_hardness(hard: f32) -> (&'static [&'static str], &'static [&'static str]) {
    if hard < 0.3 {
        (HARD_SOFT, HARD_SOFT_SUFFIX)
    } else if hard > 0.7 {
        (HARD_HARD, HARD_HARD_SUFFIX)
    } else {
        (HARD_MID, HARD_MID_SUFFIX)
    }
}



pub fn generate_name(dna: &MaterialDNA) -> String {
    let mut rng = StdRng::seed_from_u64(dna.seed);

    let material = crate::material::from_dna(dna);

    // --- Select Phonemes based on DNA ---
    let (state_prefix, state_root, state_suffix) = get_phonemes_for_state(material.state);
    let (temp_phonemes, temp_suffix) = get_phonemes_for_temperature(material.temperature);
    let (cond_phonemes, cond_suffix) = get_phonemes_for_conductivity(material.conductivity);
    let mag_phonemes = get_phonemes_for_magnetism(material.magnetism);
    let (lum_phonemes, lum_suffix) = get_phonemes_for_luminescence(material.luminescence);
    let visc_phonemes = get_phonemes_for_viscosity(material.viscosity);
    let (hard_phonemes, hard_suffix) = get_phonemes_for_hardness(material.hardness);

    // --- Choose Template ---
    let template_id = match material.state {
        State::Solid => if rng.gen_bool(0.5) { "T2" } else { "T3" },
        State::Liquid => "T2",
        State::Gas => if rng.gen_bool(0.5) { "T1" } else { "T4" },
        State::Particle => "T2", // Default for Particle
    };

    let final_template = if material.luminescence > 0.7 && rng.gen_bool(0.3) {
        "T5"
    } else {
        template_id
    };

    // --- Generate Name based on Template ---
    let mut name = String::new();

    match final_template {
        "T1" => {
            let prefix = choose(&mut rng, state_prefix);
            let root = choose(&mut rng, state_root);
            name = format!("{}{}", prefix, root);
        }
        "T2" => {
            let prefix = choose(&mut rng, state_prefix);
            let root = choose(&mut rng, state_root);
            let suffix = choose(&mut rng, state_suffix);
            name = format!("{}{}{}", prefix, root, suffix.strip_prefix('-').unwrap_or(suffix));
        }
        "T3" => {
            let root = choose(&mut rng, state_root);
            let suffix = choose(&mut rng, state_suffix);
            name = format!("{}{}", root, suffix.strip_prefix('-').unwrap_or(suffix));
        }
        "T4" => {
            let prefix = choose(&mut rng, state_prefix);
            let root = choose(&mut rng, state_root);
            let variant_suffix = choose(&mut rng, &[temp_suffix, cond_suffix, lum_suffix].concat());
            name = format!("{}{}-{}", prefix, root, variant_suffix.strip_prefix('-').unwrap_or(variant_suffix));
        }
        "T5" => {
            let attr_prefix = choose(&mut rng, lum_phonemes);
            // Generate a base name using T2 template
            let base_prefix = choose(&mut rng, state_prefix);
            let base_root = choose(&mut rng, state_root);
            let base_suffix = choose(&mut rng, state_suffix);
            let base_name = format!("{}{}{}", base_prefix, base_root, base_suffix.strip_prefix('-').unwrap_or(base_suffix));
            name = format!("{}-{}{}", attr_prefix, base_name.chars().next().unwrap().to_uppercase(), &base_name[1..]);
        }
        _ => unreachable!(),
    }

    // Capitalize the first letter
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}