use std::mem;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use sqlx::PgPool;
use crate::error::AppError;
use crate::User;
use anyhow::Result;

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
        email: &str,
        fullname: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as(
            r#"insert into users(email, fullname, password_hash)
                values ($1, $2, $3)
                returning *
                "#,
        ).bind(email).bind(fullname).bind(password)
        .fetch_one(pool)
            .await?;
        Ok(user)
    }

    pub async fn verify(
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as("select * from users where email = $1")
        .bind(email)
            .fetch_optional(pool)
            .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(password, &password_hash)?;
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
        let user = User::create(email,name,password,&pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.id > 0);
        Ok(())
    }
}