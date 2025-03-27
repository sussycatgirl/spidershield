#![feature(iter_next_chunk)]

mod routes;
mod generator;

use std::{net::SocketAddr, path::Path, str::FromStr, sync::OnceLock};

use axum::{routing::get, Router};
use axum_client_ip::SecureClientIpSource;
use dotenv::dotenv;
use generator::init_chain;
use ::markov::Chain;
use prometheus_exporter::prometheus::{self, register_counter_vec, Opts};
use routes::tarpit::tarpit_handler;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, warn};

static CHAIN: OnceLock<Chain<String>> = OnceLock::new();
static CONFIG: OnceLock<ApplicationConfig> = OnceLock::new();
static METRICS: OnceLock<GlobalMetrics> = OnceLock::new();

pub fn get_chain() -> &'static Chain<String> {
    CHAIN.get().unwrap()
}

pub fn get_config() -> &'static ApplicationConfig {
    CONFIG.get().expect("Config is not initialized")
}

pub fn get_metrics() -> &'static GlobalMetrics {
    METRICS.get().unwrap()
}

#[tokio::main]
async fn main() {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Load .env file
    match dotenv() {
        Ok(_) => debug!(".env file loaded"),
        Err(e) => warn!("Failed to load .env file: {}", e)
    }

    // Initialize configuration
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
        listen: std::env::var("LISTEN").unwrap_or("127.0.0.1:3000".to_string()).into(),
        prometheus_listen: std::env::var("PROMETHEUS_LISTEN").unwrap_or("".to_string()).into(),
        client_ip_source: SecureClientIpSource::from_str(
                std::env::var("CLIENT_IP_SOURCE")
                    .unwrap_or("ConnectInfo".to_string())
                    .as_str()
            )
            .expect("Invalid value for CLIENT_IP_SOURCE. Expected one of https://docs.rs/axum-client-ip/0.7.0/axum_client_ip/enum.SecureClientIpSource.html"),
    });
    assert!(get_config().response_delay_max >= get_config().response_delay_min, "RESPONSE_DELAY_MAX must be >= RESPONSE_DELAY_MIN");
    info!("Config loaded: {:?}", get_config());

    // Set up other global variables
    CHAIN.get_or_init(|| init_chain());
    METRICS.get_or_init(|| {
        GlobalMetrics {
            requests: register_counter_vec!(Opts::new("requests", "Incoming requests"), &["ip", "user_agent", "path", "host"]).unwrap(),
        }
    });

    // Configure routes
    let app = Router::new()
        .route("/", get(tarpit_handler))
        .route("/{*path}", get(tarpit_handler))
        .nest_service("/static", axum_static::static_router("src/static"))
        .layer(get_config().client_ip_source.clone().into_extension())
        .layer(TraceLayer::new_for_http());

    // Open TCP socket
    let addr = get_config().listen.as_str();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to open socket");
    info!("Listening on {}", &addr);


    // Start Prometheus server
    if !get_config().prometheus_listen.is_empty() {
        let binding = get_config().prometheus_listen.parse()
            .expect("Invalid prometheus listen address");

        let builder = prometheus_exporter::Builder::new(binding);
        builder.start()
            .expect("Failed to start prometheus server");
    } else {
        info!("PROMETHEUS_LISTEN not set, not starting prometheus exporter");
    }

    // Start app server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await.unwrap();
}

#[derive(Debug)]
pub struct ApplicationConfig {
    rng_seed: String,
    response_delay_min: u64,
    response_delay_max: u64,
    markov_corpus_path: Box<Path>,
    markov_persist_path: Box<Path>,
    listen: String,
    prometheus_listen: String,
    client_ip_source: SecureClientIpSource,
}

pub struct GlobalMetrics {
    requests: prometheus::CounterVec,
}
