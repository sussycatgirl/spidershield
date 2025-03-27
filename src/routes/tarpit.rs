use std::{collections::HashMap, time::Duration};

use axum::{http, response::Html};
use axum_client_ip::SecureClientIp;
use axum_extra::{headers, TypedHeader};
use minijinja::{render, Environment};
use rand::{Rng, RngCore};
use rand_chacha::ChaCha8Rng;
use rand_seeder::Seeder;
use tracing::debug;

use crate::{generator::{markov_generate, random_phrase, random_word}, get_chain, get_config, get_metrics};

const TARPIT_TEMPLATE: &str = include_str!("../template/tarpit.jinja");

#[axum::debug_handler]
pub async fn tarpit_handler(
    uri: http::Uri,
    client_ip: SecureClientIp,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    TypedHeader(host): TypedHeader<headers::Host>,
) -> Html<String> {
    let config = get_config();
    let metrics = get_metrics();

    let mut labels: HashMap<&str, &str> = HashMap::new();
    let client_ip_str = client_ip.0.to_string();
    labels.insert("ip", client_ip_str.as_str());
    labels.insert("path", uri.path());
    let host_str = host.to_string();
    labels.insert("host", host_str.as_str());
    labels.insert("user_agent", user_agent.as_str());
    metrics.requests.with(&labels).inc();

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
        links.push(random_word(&mut rng, 4, 12));
    }

    // Title is also made from random words
    let title = random_phrase(get_chain(), &mut rng, 2, 5);

    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| { minijinja::AutoEscape::Html });
    let r = render!(
        in env,
        TARPIT_TEMPLATE,
        path => uri.to_string(),
        title => title,
        content => content,
        author => author,
        links => links,
    );

    // Delay the response
    if config.response_delay_max > 0 {
        let duration = Duration::from_millis(
            rand::random_range(config.response_delay_min..=config.response_delay_max)
        );

        debug!("Delaying response for {:?}", duration);
        tokio::time::sleep(
            duration
        ).await;
    }
    Html(r)
}
