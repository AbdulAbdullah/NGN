 use crate::app::errors::ApiError;
 use sqlx::{PgPool, Row};
 use uuid::Uuid;
 
 pub async fn user_exists_by_email(db: &PgPool, email: &str) -> Result<bool, ApiError> {
     let row = sqlx::query("select 1 from users where email = $1 limit 1")
         .bind(email)
         .fetch_optional(db)
         .await
         .map_err(|e| ApiError::Internal(e.into()))?;
     Ok(row.is_some())
 }
 
 pub struct UserRow {
     pub id: Uuid,
     pub email: String,
     pub display_name: String,
     pub password_hash: String,
     pub role: String,
 }
 
 pub async fn create_user(
     db: &PgPool,
     email: &str,
     display_name: &str,
     password_hash: &str,
 ) -> Result<UserRow, ApiError> {
     let row = sqlx::query(
         r#"
         insert into users (email, display_name, password_hash)
         values ($1, $2, $3)
         returning id, email, display_name, password_hash, role
         "#,
     )
     .bind(email)
     .bind(display_name)
     .bind(password_hash)
     .fetch_one(db)
     .await
     .map_err(|e| {
         if let Some(db_err) = e.as_database_error() {
             if db_err.code().as_deref() == Some("23505") {
                 return ApiError::conflict("email already exists");
             }
         }
         ApiError::Internal(e.into())
     })?;
 
     Ok(UserRow {
         id: row.get::<Uuid, _>("id"),
         email: row.get::<String, _>("email"),
         display_name: row.get::<String, _>("display_name"),
         password_hash: row.get::<String, _>("password_hash"),
         role: row.get::<String, _>("role"),
     })
 }
 
 pub async fn get_user_by_email(db: &PgPool, email: &str) -> Result<Option<UserRow>, ApiError> {
     let row = sqlx::query(
         "select id, email, display_name, password_hash, role from users where email = $1",
     )
     .bind(email)
     .fetch_optional(db)
     .await
     .map_err(|e| ApiError::Internal(e.into()))?;
 
     Ok(row.map(|r| UserRow {
         id: r.get::<Uuid, _>("id"),
         email: r.get::<String, _>("email"),
         display_name: r.get::<String, _>("display_name"),
         password_hash: r.get::<String, _>("password_hash"),
         role: r.get::<String, _>("role"),
     }))
 }
 
 pub async fn get_user_by_id(db: &PgPool, user_id: Uuid) -> Result<Option<UserRow>, ApiError> {
     let row = sqlx::query(
         "select id, email, display_name, password_hash, role from users where id = $1",
     )
     .bind(user_id)
     .fetch_optional(db)
     .await
     .map_err(|e| ApiError::Internal(e.into()))?;
 
     Ok(row.map(|r| UserRow {
         id: r.get::<Uuid, _>("id"),
         email: r.get::<String, _>("email"),
         display_name: r.get::<String, _>("display_name"),
         password_hash: r.get::<String, _>("password_hash"),
         role: r.get::<String, _>("role"),
     }))
 }
 
