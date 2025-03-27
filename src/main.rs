#![feature(iter_next_chunk)]

mod routes;
mod markov;

use std::sync::OnceLock;

use axum::{routing::get, Router};
use markov::init_chain;
use ::markov::Chain;
use routes::tarpit::tarpit_handler;

pub fn get_chain() -> &'static Chain<String> {
    static CHAIN: OnceLock<Chain<String>> = OnceLock::new();
    CHAIN.get_or_init(|| init_chain())
}

#[tokio::main]
async fn main() {
    get_chain();

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
