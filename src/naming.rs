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

fn build_markov_transitions(state: State, _rng: &mut StdRng) -> Transitions {
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