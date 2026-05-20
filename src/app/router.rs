 use crate::app::{
     features::{auth, claims, lookup},
     openapi::ApiDoc,
     state::AppState,
 };
 use axum::{
     http::Method,
     routing::{get, post},
     Router,
 };
 use tower_http::{
     cors::{Any, CorsLayer},
     trace::TraceLayer,
 };
use utoipa::OpenApi;
 use utoipa_swagger_ui::SwaggerUi;
 
 pub fn build_router(state: AppState, cors_allow_any_origin: bool) -> Router {
     let cors = if cors_allow_any_origin {
         CorsLayer::new()
             .allow_origin(Any)
             .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
             .allow_headers(Any)
     } else {
         CorsLayer::permissive()
     };
 
     Router::new()
         .route("/health", get(|| async { "ok" }))
         .nest(
             "/auth",
             Router::new()
                 .route("/signup", post(auth::signup))
                 .route("/login", post(auth::login)),
         )
         .route("/me", get(auth::me))
         .route("/lookup", get(lookup::lookup))
         .route("/claims", post(claims::create_claim))
         .merge(
             SwaggerUi::new("/docs")
                 .url("/api-docs/openapi.json", ApiDoc::openapi()),
         )
         .layer(TraceLayer::new_for_http())
         .layer(cors)
         .with_state(state)
 }
