mod routes;
mod markov;

use std::sync::OnceLock;

use axum::{routing::get, Router};
use markov::{generate, init_chain};
use markov_generator::HashChain;
use routes::tarpit::tarpit_handler;

pub fn get_chain() -> &'static HashChain<char> {
    static CHAIN: OnceLock<HashChain<char>> = OnceLock::new();
    CHAIN.get_or_init(|| init_chain())
}

#[tokio::main]
async fn main() {
    get_chain();

    let app = Router::new()
        .route("/", get(tarpit_handler))
        .route("/{*path}", get(tarpit_handler));

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Can't listen on port");
    println!("Listening on {}", &addr);
    axum::serve(listener, app).await.unwrap();
}
