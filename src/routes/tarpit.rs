use std::time::Duration;

use axum::{http, response::Html};
use minijinja::render;
use rand::{Rng, RngCore};
use rand_chacha::ChaCha8Rng;
use rand_seeder::Seeder;

use crate::{get_chain, get_config, markov::markov_generate};

const TARPIT_TEMPLATE: &str = include_str!("../template/tarpit.jinja");

#[axum::debug_handler]
pub async fn tarpit_handler(uri: http::Uri) -> Html<String> {
    let config = get_config();

    let mut seed: String = config.rng_seed.clone();
    seed.push_str(uri.path());
    let mut rng: ChaCha8Rng = Seeder::from(seed.as_str()).into_rng();

    let mut content = Vec::<String>::new();
    while content.join(" ").len() < 1000 {
        let segment = markov_generate(get_chain(), &mut rng);

        for paragraph in segment.split("\n\n") {
            content.push(paragraph.to_string());
        }

        rng.next_u32(); // Make sure next iteration returns a different string
    }

    let author = markov_generate(get_chain(), &mut rng)
        .split_whitespace()
        .next_chunk::<2>()
        .unwrap_or(["Unknown", "author"])
        .join(" ");

    let mut links = Vec::<String>::new();

    // Generate between 2 and 10 links
    for _ in 0..rng.random_range(2..=10) {
        let mut word = String::new();

        // Every link is a word with between 4 and 12 characters
        for _ in 0..rng.random_range(4..=12) {
            // Ascii char between 0x61 and 0x7A (lowercase a-z)
            word.push(rng.random_range(0x61..=0x7A).into());
        }

        links.push(word);
    }

    if config.response_delay_max > 0 {
        tokio::time::sleep(
            Duration::from_millis(
                rand::random_range(config.response_delay_min..=config.response_delay_max)
            )
        ).await;
    }

    let r = render!(TARPIT_TEMPLATE, path => uri.to_string(), content => content, author => author, links => links);
    Html(r)
}
