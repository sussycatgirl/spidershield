use axum::{http, response::Html};
use minijinja::render;

const TARPIT_TEMPLATE: &str = include_str!("../template/tarpit.jinja");

#[axum::debug_handler]
pub async fn tarpit_handler(path: http::Uri) -> Html<String> {
    let r = render!(TARPIT_TEMPLATE, title => path.to_string(), content => "waowaowaow");
    Html(r)
}
