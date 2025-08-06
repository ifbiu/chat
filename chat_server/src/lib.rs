mod config;
mod handlers;
mod models;
mod error;
mod utils;

use std::ops::Deref;
use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post, put};
pub use config::AppConfig;
use crate::handlers::*;
pub use models::User;


#[derive(Debug, Clone)]
pub(crate) struct AppState{
    inner: Arc<AppStateInner>,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner{config}),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}
pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);
    let api = Router::new()
        .route("/signin",post(signin_handler))
        .route("/signup",post(signup_handler))
        .route("/chat",get(list_chat_handler).post(create_chat_handler))
        .route("/chat/{id}",put(update_chat_handler).delete(delete_chat_handler))
        .route("/chat/{id}/messages",get(list_message_handler));
    Router::new()
    .route("/", get(index_handler))
        .nest("/api",api)
        .with_state(state)
}