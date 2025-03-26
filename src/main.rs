mod routes;
mod markov;

use axum::{routing::get, Router};
use markov::{generate, init_chain};
use routes::tarpit::tarpit_handler;

#[tokio::main]
async fn main() {
    let chain = init_chain();
    println!("{}", generate(&chain));

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
