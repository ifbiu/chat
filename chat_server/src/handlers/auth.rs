use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{AppState, User};
use crate::error::AppError;
use crate::models::user::{CreateUser, SigninUser};


#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput{
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input,&state.pool).await?;
    let token = state.ek.sign(user)?;
    // let mut header = HeaderMap::new();
    // header.insert("X-Token", HeaderValue::from_str(&token)?);
    // Ok((StatusCode::CREATED, token))
    let body = Json(AuthOutput{token});
    Ok((StatusCode::CREATED,body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input,&state.pool).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput{token})).into_response())
        }
        None => {
            Ok((StatusCode::FORBIDDEN, "Invalid email or password").into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use crate::AppConfig;
    use super::*;
    #[tokio::test]
    async fn signup_should_work() -> Result<(), AppError> {
        let (tdb, state) = AppState::try_new_test().await?;
        let input = CreateUser::new("Candide","cifbiu@gmail.com","123456");
        let ret = signup_handler(State(state), Json(input)).await?.into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await.expect("error").to_bytes();
        let ret :AuthOutput = serde_json::from_slice(&body).expect("error");
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<(), AppError> {
        let (tdb, state) = AppState::try_new_test().await?;
        let name = "Candide";
        let email = "cifbiu@gmail.com";
        let password = "123456";
        let user = CreateUser::new(name,email,password);
        User::create(&user,&state.pool).await?;
        let input = SigninUser::new(email,password);
        let ret = signin_handler(State(state), Json(input)).await?.into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await.expect("error").to_bytes();
        let ret :AuthOutput = serde_json::from_slice(&body).expect("error");
        assert_ne!(ret.token,"");
        Ok(())
    }
}