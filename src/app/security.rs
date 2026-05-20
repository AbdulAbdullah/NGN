 use crate::app::errors::ApiError;
 use argon2::{
     password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
     Argon2,
 };
 use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
 use serde::{Deserialize, Serialize};
 use time::{Duration, OffsetDateTime};
 use uuid::Uuid;
 
 #[derive(Clone)]
 pub struct Jwt {
     encoding: EncodingKey,
     decoding: DecodingKey,
     issuer: String,
     audience: String,
     access_ttl_seconds: i64,
 }
 
 #[derive(Debug, Serialize, Deserialize)]
 pub struct Claims {
     pub sub: String,
     pub iss: String,
     pub aud: String,
     pub exp: usize,
     pub iat: usize,
     pub role: String,
 }
 
 #[derive(Debug, Clone)]
 pub struct AuthUser {
     pub user_id: Uuid,
     pub role: String,
 }
 
 impl Jwt {
     pub fn new(secret: String, issuer: String, audience: String, access_ttl_seconds: i64) -> Self {
         Self {
             encoding: EncodingKey::from_secret(secret.as_bytes()),
             decoding: DecodingKey::from_secret(secret.as_bytes()),
             issuer,
             audience,
             access_ttl_seconds,
         }
     }
 
     pub fn sign_access_token(&self, user_id: Uuid, role: &str) -> anyhow::Result<String> {
         let now = OffsetDateTime::now_utc();
         let exp = now + Duration::seconds(self.access_ttl_seconds);
 
         let claims = Claims {
             sub: user_id.to_string(),
             iss: self.issuer.clone(),
             aud: self.audience.clone(),
             iat: now.unix_timestamp() as usize,
             exp: exp.unix_timestamp() as usize,
             role: role.to_string(),
         };
 
         Ok(jsonwebtoken::encode(&Header::default(), &claims, &self.encoding)?)
     }
 
     pub fn verify(&self, token: &str) -> Result<AuthUser, ApiError> {
         let mut validation = Validation::default();
         validation.set_issuer(&[self.issuer.clone()]);
         validation.set_audience(&[self.audience.clone()]);
 
         let data = jsonwebtoken::decode::<Claims>(token, &self.decoding, &validation)
             .map_err(|_| ApiError::Unauthorized)?;
 
         let user_id = Uuid::parse_str(&data.claims.sub).map_err(|_| ApiError::Unauthorized)?;
         Ok(AuthUser {
             user_id,
             role: data.claims.role,
         })
     }
 }
 
 pub fn hash_password(password: &str) -> Result<String, ApiError> {
     if password.len() < 8 {
         return Err(ApiError::bad_request("password must be at least 8 characters"));
     }
 
     let salt = SaltString::generate(&mut OsRng);
     let argon2 = Argon2::default();
     argon2
         .hash_password(password.as_bytes(), &salt)
         .map(|ph| ph.to_string())
         .map_err(|_| ApiError::Internal(anyhow::anyhow!("password hashing failed")))
 }
 
 pub fn verify_password(hash: &str, password: &str) -> Result<bool, ApiError> {
     let parsed = PasswordHash::new(hash)
         .map_err(|_| ApiError::Internal(anyhow::anyhow!("invalid password hash stored")))?;
     Ok(Argon2::default()
         .verify_password(password.as_bytes(), &parsed)
         .is_ok())
 }
