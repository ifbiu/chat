use axum::body::Body;
use axum::http;
use axum::http::StatusCode;
use axum::response::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] http::header::InvalidHeaderValue),

    #[error("from utf8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let status = match &self {
            Self::SqlxError(_) =>StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) =>StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtError(_) =>StatusCode::FORBIDDEN,
            Self::HttpHeaderError(_) =>StatusCode::BAD_REQUEST,
            Self::FromUtf8Error(_) =>StatusCode::BAD_REQUEST,
        };
        (status, Json(json!({"error": self.to_string()}))).into_response()
    }
}