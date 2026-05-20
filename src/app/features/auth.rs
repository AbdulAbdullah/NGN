 use crate::app::{
     db,
     errors::ApiError,
     security::{self, AuthUser},
     state::AppState,
 };
 use axum::{
     extract::State,
     http::{header, HeaderMap},
     Json,
 };
 use serde::{Deserialize, Serialize};
 use utoipa::ToSchema;
 
 #[derive(Debug, Deserialize, ToSchema)]
 pub struct SignupRequest {
     pub email: String,
     pub password: String,
     pub display_name: String,
 }
 
 #[derive(Debug, Deserialize, ToSchema)]
 pub struct LoginRequest {
     pub email: String,
     pub password: String,
 }
 
 #[derive(Debug, Serialize, ToSchema)]
 pub struct AuthResponse {
     pub access_token: String,
     pub token_type: String,
     pub user: MeResponse,
 }
 
 #[derive(Debug, Serialize, ToSchema)]
 pub struct MeResponse {
     pub id: String,
     pub email: String,
     pub display_name: String,
     pub role: String,
 }
 
 fn normalize_email(email: &str) -> String {
     email.trim().to_lowercase()
 }
 
 #[utoipa::path(
     post,
     path = "/auth/signup",
     tag = "auth",
     request_body = SignupRequest,
     responses(
         (status = 200, description = "Created user + token", body = AuthResponse),
         (status = 409, description = "Email exists")
     )
 )]
 pub async fn signup(
     State(state): State<AppState>,
     Json(req): Json<SignupRequest>,
 ) -> Result<Json<AuthResponse>, ApiError> {
     let email = normalize_email(&req.email);
 
     if db::user_exists_by_email(state.db(), &email).await? {
         return Err(ApiError::conflict("email already exists"));
     }
 
     let password_hash = security::hash_password(&req.password)?;
     let user =
         db::create_user(state.db(), &email, req.display_name.trim(), &password_hash).await?;
 
     let token = state.jwt().sign_access_token(user.id, &user.role)?;
 
     Ok(Json(AuthResponse {
         access_token: token,
         token_type: "Bearer".to_string(),
         user: MeResponse {
             id: user.id.to_string(),
             email: user.email,
             display_name: user.display_name,
             role: user.role,
         },
     }))
 }
 
 #[utoipa::path(
     post,
     path = "/auth/login",
     tag = "auth",
     request_body = LoginRequest,
     responses(
         (status = 200, description = "Token", body = AuthResponse),
         (status = 401, description = "Invalid credentials")
     )
 )]
 pub async fn login(
     State(state): State<AppState>,
     Json(req): Json<LoginRequest>,
 ) -> Result<Json<AuthResponse>, ApiError> {
     let email = normalize_email(&req.email);
     let user = db::get_user_by_email(state.db(), &email)
         .await?
         .ok_or(ApiError::Unauthorized)?;
 
     let ok = security::verify_password(&user.password_hash, &req.password)?;
     if !ok {
         return Err(ApiError::Unauthorized);
     }
 
     let token = state.jwt().sign_access_token(user.id, &user.role)?;
     Ok(Json(AuthResponse {
         access_token: token,
         token_type: "Bearer".to_string(),
         user: MeResponse {
             id: user.id.to_string(),
             email: user.email,
             display_name: user.display_name,
             role: user.role,
         },
     }))
 }
 
 fn bearer_from_headers(headers: &HeaderMap) -> Option<&str> {
     let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
     value.strip_prefix("Bearer ").or_else(|| value.strip_prefix("bearer "))
 }
 
 fn require_auth(state: &AppState, headers: &HeaderMap) -> Result<AuthUser, ApiError> {
     let token = bearer_from_headers(headers).ok_or(ApiError::Unauthorized)?;
     state.jwt().verify(token)
 }
 
 #[utoipa::path(
     get,
     path = "/me",
     tag = "auth",
     security(
         ("bearerAuth" = [])
     ),
     responses(
         (status = 200, description = "Current user", body = MeResponse),
         (status = 401, description = "Unauthorized")
     )
 )]
 pub async fn me(
     State(state): State<AppState>,
     headers: HeaderMap,
 ) -> Result<Json<MeResponse>, ApiError> {
     let auth = require_auth(&state, &headers)?;
     let user = db::get_user_by_id(state.db(), auth.user_id)
         .await?
         .ok_or(ApiError::Unauthorized)?;
 
     Ok(Json(MeResponse {
         id: user.id.to_string(),
         email: user.email,
         display_name: user.display_name,
         role: user.role,
     }))
 }
 
