 use crate::app::{config::Config, security::Jwt};
 use sqlx::{postgres::PgPoolOptions, PgPool};
 use std::sync::Arc;
 
 #[derive(Clone)]
 pub struct AppState {
     inner: Arc<AppStateInner>,
 }
 
 struct AppStateInner {
     pub db: PgPool,
     pub jwt: Jwt,
 }
 
 impl AppState {
     pub async fn new(config: &Config) -> anyhow::Result<Self> {
         let db = PgPoolOptions::new()
             .max_connections(10)
             .connect(&config.database_url)
             .await?;
 
         let jwt = Jwt::new(
             config.jwt_secret.clone(),
             config.jwt_issuer.clone(),
             config.jwt_audience.clone(),
             config.jwt_access_ttl_seconds,
         );
 
         Ok(Self {
             inner: Arc::new(AppStateInner { db, jwt }),
         })
     }
 
     pub fn db(&self) -> &PgPool {
         &self.inner.db
     }
 
     pub fn jwt(&self) -> &Jwt {
         &self.inner.jwt
     }
 }
