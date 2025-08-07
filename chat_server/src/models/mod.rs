pub mod user;
use chrono::{DateTime, Utc};
use serde::{ Serialize, Deserialize };
use sqlx::FromRow;


#[derive(Serialize, Deserialize, Debug, Clone, FromRow, PartialEq)]
pub struct User{
    pub id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
impl User{
    pub fn new(id:i64, fullname: &str, email: &str) -> Self{
        Self{
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: String::default(),
            created_at: Utc::now(),
        }
    }
}