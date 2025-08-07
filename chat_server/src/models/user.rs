use std::mem;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use sqlx::PgPool;
use crate::error::AppError;
use crate::User;
use anyhow::Result;
use chrono::Utc;
use jwt_simple::prelude::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

struct EmailPass {
    email: String,
    password_hash: String,
}

impl User{
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as("select * from users where email = $1")
        .bind(email)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }

    pub async fn create(
        input: &CreateUser,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as(
            r#"insert into users(email, fullname, password_hash)
                values ($1, $2, $3)
                returning *
                "#,
        ).bind(&input.email).bind(&input.fullname).bind(&input.password)
        .fetch_one(pool)
            .await?;
        Ok(user)
    }

    pub async fn verify(
        input: &SigninUser,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as("select * from users where email = $1")
        .bind(&input.email)
            .fetch_optional(pool)
            .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(&input.password, &password_hash)?;
                if is_valid{
                    Ok(Some(user))
                }else{
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;
    let is_valid = argon2.verify_password(password.as_bytes(), &password_hash).is_ok();
    Ok(is_valid)
}

#[cfg(test)]
impl CreateUser {
    pub  fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self{
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use sqlx_db_tester::TestPg;
    use super::*;
    #[tokio::test]
    async fn create_user_should_work() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:123456@127.0.0.1:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let email = "cifbiu@gmail.com";
        let name = "candide";
        let password = "11111";
        let input = CreateUser::new(name, email, password);
        let user = User::create(&input,&pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);

        let input = SigninUser::new(&input.email,&input.password);
        let user = User::verify(&input, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}