 use axum::{
     http::StatusCode,
     response::{IntoResponse, Response},
     Json,
 };
 use serde::Serialize;
 
 #[derive(Debug, Serialize)]
 pub struct ErrorBody {
     pub error: String,
 }
 
 #[derive(thiserror::Error, Debug)]
 pub enum ApiError {
     #[error("unauthorized")]
     Unauthorized,
     #[error("forbidden")]
     Forbidden,
     #[error("bad request: {0}")]
     BadRequest(String),
     #[error("not found")]
     NotFound,
     #[error("conflict: {0}")]
     Conflict(String),
     #[error("internal server error")]
     Internal(#[from] anyhow::Error),
 }
 
 impl ApiError {
     pub fn bad_request(msg: impl Into<String>) -> Self {
         Self::BadRequest(msg.into())
     }
 
     pub fn conflict(msg: impl Into<String>) -> Self {
         Self::Conflict(msg.into())
     }
 }
 
 impl IntoResponse for ApiError {
     fn into_response(self) -> Response {
         let (status, msg) = match &self {
             ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
             ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
             ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
             ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
             ApiError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
             ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
         };
 
         (status, Json(ErrorBody { error: msg })).into_response()
     }
 }
