mod config;
mod handlers;
mod models;
mod error;
mod utils;

use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use anyhow::Context;
use axum::Router;
use axum::routing::{get, post, put};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx_db_tester::TestPg;
pub use config::AppConfig;
use crate::handlers::*;
pub use models::User;
use crate::error::AppError;
use crate::utils::{DecodingKey, EncodingKey};

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
    pub async fn try_new(config: AppConfig) -> Result<Self,AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
       Ok(Self {
           inner: Arc::new(AppStateInner{
               config,
               dk,
               ek,
               pool,
           }),
       })
    }
    #[cfg(test)]
    pub async fn try_new_test() -> Result<(TestPg,Self),AppError> {
        let config = AppConfig::load()?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let server_url = config.server.db_url.split('/').next().unwrap();
        let tdb = TestPg::new(
            server_url.to_string(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        Ok((tdb,Self {
            inner: Arc::new(AppStateInner{
                config,
                dk,
                ek,
                pool,
            }),
        }))
    }
}


#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}

impl Debug for AppStateInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
        .field("config", &self.config)
        .finish()
    }
}

pub async  fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/signin",post(signin_handler))
        .route("/signup",post(signup_handler))
        .route("/chat",get(list_chat_handler).post(create_chat_handler))
        .route("/chat/{id}",put(update_chat_handler).delete(delete_chat_handler))
        .route("/chat/{id}/messages",get(list_message_handler));
    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api",api)
        .with_state(state);
    Ok(app)
}