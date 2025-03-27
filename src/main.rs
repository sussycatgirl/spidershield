#![feature(iter_next_chunk)]

mod routes;
mod generator;

use std::{path::Path, sync::OnceLock};

use axum::{routing::get, Router};
use dotenv::dotenv;
use generator::init_chain;
use ::markov::Chain;
use routes::tarpit::tarpit_handler;

static CHAIN: OnceLock<Chain<String>> = OnceLock::new();
pub fn get_chain() -> &'static Chain<String> {
    CHAIN.get().unwrap()
}

static CONFIG: OnceLock<ApplicationConfig> = OnceLock::new();
pub fn get_config() -> &'static ApplicationConfig {
    CONFIG.get().expect("Config is not initialized")
}

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(_) => {},
        Err(e) => println!("Failed to load .env file: {}", e)
    }

    CONFIG.get_or_init(|| ApplicationConfig {
        rng_seed: std::env::var("RNG_SEED").unwrap_or("".to_string()).into(),
        response_delay_min: std::env::var("RESPONSE_DELAY_MIN")
            .unwrap_or("0".to_string())
            .parse()
            .expect("Failed to parse RESPONSE_DELAY_MIN as number"),
        response_delay_max: std::env::var("RESPONSE_DELAY_MAX")
            .unwrap_or("0".to_string())
            .parse()
            .expect("Failed to parse RESPONSE_DELAY_MAX as number"),
        markov_corpus_path: Path::new(&std::env::var("MARKOV_CORPUS_PATH").unwrap_or("datasets/hdg.txt".to_string())).into(),
        markov_persist_path: Path::new(&std::env::var("MARKOV_PERSIST_PATH").unwrap_or("data.chain".to_string())).into(),
    });
    println!("Config loaded: {:?}", get_config());

    CHAIN.get_or_init(|| init_chain());

    let app = Router::new()
        .route("/", get(tarpit_handler))
        .route("/{*path}", get(tarpit_handler))
        .nest_service("/static", axum_static::static_router("src/static"));

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Can't listen on port");
    println!("Listening on {}", &addr);
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
pub struct ApplicationConfig {
    rng_seed: String,
    response_delay_min: u64,
    response_delay_max: u64,
    markov_corpus_path: Box<Path>,
    markov_persist_path: Box<Path>,
}
