use std::ops::Deref;
use jwt_simple::prelude::*;
use crate::error::AppError;
use crate::User;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";
pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

impl Deref for EncodingKey {
    type Target = Ed25519KeyPair;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String,AppError> {
        let user = user.into();
        let mut claims = Claims::with_custom_claims(user,Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER)
            .with_audience(JWT_AUDIENCE);
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self,token:&str) -> Result<User, AppError> {
        let mut options = VerificationOptions::default();
        options.allowed_issuers = Some(HashSet::from_strings(&[JWT_ISSUER]));
        options.allowed_audiences = Some(HashSet::from_strings(&[JWT_AUDIENCE]));
        let claims = self.0.verify_token::<User>(token, None)?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_jwt()  ->Result<()>{
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");
        let ek = EncodingKey::load(&encoding_pem)?;
        let dk = DecodingKey::load(&decoding_pem)?;

        let user = User::new(1,"Candide","cifbiu@gmail.com");
        let token = ek.sign(user.clone())?;
        let user2 = dk.verify(&token)?;
        Ok(())
    }
}


