 use crate::app::features::{auth, claims, lookup};
 use utoipa::OpenApi;
 use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
 
 #[derive(OpenApi)]
 #[openapi(
     paths(
         auth::signup,
         auth::login,
         auth::me,
         lookup::lookup,
         claims::create_claim
     ),
     components(
         schemas(
             auth::SignupRequest,
             auth::AuthResponse,
             auth::LoginRequest,
             auth::MeResponse,
             lookup::LookupResponse,
             lookup::CommunityStatus,
             claims::CreateClaimRequest,
             claims::ClaimStatus,
             claims::CreateClaimResponse
         )
     ),
     tags(
         (name = "auth", description = "Authentication"),
         (name = "lookup", description = "Product lookup"),
         (name = "claims", description = "Community claims")
     ),
     modifiers(&SecurityAddon)
 )]
 pub struct ApiDoc;

 struct SecurityAddon;

 impl utoipa::Modify for SecurityAddon {
     fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
         let bearer = SecurityScheme::Http(
             HttpBuilder::new()
                 .scheme(HttpAuthScheme::Bearer)
                 .bearer_format("JWT")
                 .build(),
         );
         let mut components = openapi
             .components
             .take()
             .unwrap_or_else(utoipa::openapi::Components::new);
         components.add_security_scheme("bearerAuth", bearer);
         openapi.components = Some(components);
     }
 }
