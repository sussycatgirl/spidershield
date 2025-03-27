use markov::Chain;
use rand::Rng;
use tracing::info;

use crate::get_config;

pub fn init_chain() -> Chain<String> {
    let config = get_config();

    info!("Loading markov chain from {:?}", config.markov_persist_path);
    match Chain::load(config.markov_persist_path.clone()) {
        Ok(chain) => {
            info!("Load finished");
            chain
        },
        Err(_) => {
            info!("Generating new markov chain from corpus at {:?}", config.markov_corpus_path);
            let mut chain: Chain<String> = Chain::new();
            chain.feed_file(config.markov_corpus_path.clone()).expect("Failed to open corpus file");
            info!("Chain generated, persisting to file");
            chain.save(config.markov_persist_path.clone()).unwrap();
            info!("Chain written to disk");

            chain
        }
    }
}

pub fn markov_generate<T: Rng>(chain: &Chain<String>, rng: &mut T) -> String {
    chain.generate_with_rng(rng).join(" ")
}

// Generates a random lowercase "word" consisting of between min and max random characters
pub fn random_word<T: Rng>(rng: &mut T, min: usize, max: usize) -> String {
    let mut word = String::new();

    for _ in 0..rng.random_range(min..=max) {
        // Ascii char between 0x61 and 0x7A (lowercase a-z)
        word.push(rng.random_range(0x61..=0x7A).into());
    }

    word
}

// Markov generates a phrase trimmed between min and max words
pub fn random_phrase<T: Rng>(chain: &Chain<String>, rng: &mut T, min: usize, max: usize) -> String {
    let mut words: Vec<String> = Vec::new();

    while words.len() < min {
        let phrase = markov_generate(chain, rng);
        for word in phrase.split(" ") { words.push(String::from(word)) }
    }

    words.truncate(max);
    return words.join(" ");
}
