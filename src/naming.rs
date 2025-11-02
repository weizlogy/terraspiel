use crate::material::{MaterialDNA, State};
use rand::distributions::{Distribution, WeightedIndex};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

// --- Markov Chain Model ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Phoneme(String);

type Transitions = std::collections::HashMap<Phoneme, Vec<(Phoneme, f32)>>;

// --- Phoneme Sets ---

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

fn choose_weighted<T>(rng: &mut impl Rng, slice: &[T], weight: f32) -> T
where
    T: Clone,
{
    if slice.is_empty() {
        panic!("Cannot choose from an empty slice.");
    }
    let weights: Vec<f32> = (0..slice.len())
        .map(|i| {
            let i_normalized = i as f32 / (slice.len() - 1) as f32;
            1.0 - (i_normalized - weight).abs()
        })
        .collect();

    let dist = WeightedIndex::new(&weights).unwrap();
    slice[dist.sample(rng)].clone()
}

fn get_phonemes_for_state(state: State) -> (&'static [&'static str], &'static [&'static str], &'static [&'static str]) {
    match state {
        State::Solid => (SOLID_PREFIX, SOLID_ROOT, SOLID_SUFFIX),
        State::Liquid => (LIQUID_PREFIX, LIQUID_ROOT, LIQUID_SUFFIX),
        State::Gas => (GAS_PREFIX, GAS_ROOT, GAS_SUFFIX),
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



fn build_markov_transitions(state: State, rng: &mut StdRng) -> Transitions {
    let mut transitions = Transitions::new();
    let all_phonemes: Vec<Phoneme> = SOLID_PREFIX.iter().chain(SOLID_ROOT.iter()).chain(SOLID_SUFFIX.iter())
        .chain(LIQUID_PREFIX.iter()).chain(LIQUID_ROOT.iter()).chain(LIQUID_SUFFIX.iter())
        .chain(GAS_PREFIX.iter()).chain(GAS_ROOT.iter()).chain(GAS_SUFFIX.iter())
        .map(|s| Phoneme(s.to_string().replace("-", ""))).collect();

    for p1 in &all_phonemes {
        let mut dests = Vec::new();
        for p2 in &all_phonemes {
            if p1 != p2 {
                let mut weight = 1.0;
                // State-specific transition biases
                match state {
                    State::Solid => {
                        if (p1.0.contains('r') && p2.0.contains('d')) || (p1.0.contains('d') && p2.0.contains('r')) {
                            weight = 5.0; // Increase probability for r <-> d
                        }
                    }
                    State::Liquid => {
                        if (p1.0.contains('a') && p2.0.contains('e')) || (p1.0.contains('e') && p2.0.contains('a')) {
                            weight = 5.0; // Increase probability for a <-> e
                        }
                    }
                    State::Gas => {
                        if (p1.0 == "ae" && p2.0 == "l") || (p1.0 == "l" && p2.0 == "ion") {
                            weight = 5.0; // Increase probability for ae -> l -> ion
                        }
                    }
                }
                dests.push((p2.clone(), weight));
            }
        }
        transitions.insert(p1.clone(), dests);
    }
    transitions
}

pub fn generate_name(dna: &MaterialDNA) -> String {
    let mut rng = StdRng::seed_from_u64(dna.seed);
    let material = crate::material::from_dna(dna);

    let transitions = build_markov_transitions(material.state, &mut rng);

    // --- Choose Template ---
    let template_len = match material.state {
        State::Solid => rng.gen_range(2..=3),
        State::Liquid => 3,
        State::Gas => rng.gen_range(1..=2),
    };

    // --- Generate Name ---
    let all_phonemes: Vec<Phoneme> = transitions.keys().cloned().collect();
    let mut name_phonemes = Vec::new();
    let mut current_phoneme = all_phonemes.choose(&mut rng).unwrap().clone();
    name_phonemes.push(current_phoneme.clone());

    for _ in 1..template_len {
        if let Some(dests) = transitions.get(&current_phoneme) {
            if !dests.is_empty() {
                let dist = WeightedIndex::new(dests.iter().map(|(_, w)| *w).collect::<Vec<_>>()).unwrap();
                current_phoneme = dests[dist.sample(&mut rng)].0.clone();
                name_phonemes.push(current_phoneme.clone());
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let name = name_phonemes.iter().map(|p| p.0.as_str()).collect::<String>();

    // Capitalize the first letter
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}