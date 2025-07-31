mod sse;
use axum::{
    Router};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use crate::sse::sse_handler;

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
    .route("/events",get(sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    let content = include_str!("../index.html");
    Html(content)
}