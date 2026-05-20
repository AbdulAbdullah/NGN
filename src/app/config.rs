 use anyhow::Context;
 
 #[derive(Clone)]
 pub struct Config {
     pub env: String,
     pub host: std::net::IpAddr,
     pub port: u16,
 
     pub database_url: String,
 
     pub jwt_secret: String,
     pub jwt_issuer: String,
     pub jwt_audience: String,
     pub jwt_access_ttl_seconds: i64,
 
     pub cors_allow_any_origin: bool,
 }
 
 impl Config {
     pub fn from_env() -> anyhow::Result<Self> {
         let env = env_var("APP_ENV").unwrap_or_else(|| "development".to_string());
         let host: std::net::IpAddr = env_var("APP_HOST")
             .unwrap_or_else(|| "0.0.0.0".to_string())
             .parse()
             .context("APP_HOST must be a valid IP")?;
         let port: u16 = env_var("APP_PORT")
             .unwrap_or_else(|| "8080".to_string())
             .parse()
             .context("APP_PORT must be a valid u16")?;
 
         let database_url =
             env_var("DATABASE_URL").context("DATABASE_URL is required (postgres://...)")?;
 
         let jwt_secret = env_var("JWT_SECRET").context("JWT_SECRET is required")?;
         let jwt_issuer = env_var("JWT_ISSUER").unwrap_or_else(|| "ngn-verify".to_string());
         let jwt_audience = env_var("JWT_AUDIENCE").unwrap_or_else(|| "ngn-verify-api".to_string());
         let jwt_access_ttl_seconds: i64 = env_var("JWT_ACCESS_TTL_SECONDS")
             .unwrap_or_else(|| "3600".to_string())
             .parse()
             .context("JWT_ACCESS_TTL_SECONDS must be an integer")?;
 
         let cors_allow_any_origin = env == "development";
 
         Ok(Self {
             env,
             host,
             port,
             database_url,
             jwt_secret,
             jwt_issuer,
             jwt_audience,
             jwt_access_ttl_seconds,
             cors_allow_any_origin,
         })
     }
 }
 
 fn env_var(key: &str) -> Option<String> {
     std::env::var(key).ok().filter(|s| !s.trim().is_empty())
 }
