 use crate::app::{
     errors::ApiError,
     state::AppState,
 };
 use axum::{
     extract::State,
     http::{header, HeaderMap},
     Json,
 };
 use serde::{Deserialize, Serialize};
 use sqlx::Row;
 use utoipa::ToSchema;
 use uuid::Uuid;
 
 #[derive(Debug, Deserialize, ToSchema)]
 pub enum ClaimStatus {
     LEGIT,
     SUSPECT,
 }
 
 #[derive(Debug, Deserialize, ToSchema)]
 pub struct CreateClaimRequest {
     pub gtin: String,
     pub product_name: Option<String>,
     pub brand: Option<String>,
     pub status: ClaimStatus,
     pub note: Option<String>,
 }
 
 #[derive(Debug, Serialize, ToSchema)]
 pub struct CreateClaimResponse {
     pub claim_id: String,
     pub product_id: String,
     pub community_status: String,
 }
 
 fn bearer_from_headers(headers: &HeaderMap) -> Option<&str> {
     let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
     value.strip_prefix("Bearer ").or_else(|| value.strip_prefix("bearer "))
 }
 
 fn require_user_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, ApiError> {
     let token = bearer_from_headers(headers).ok_or(ApiError::Unauthorized)?;
     let user = state.jwt().verify(token)?;
     Ok(user.user_id)
 }
 
 fn status_to_str(status: &ClaimStatus) -> &'static str {
     match status {
         ClaimStatus::LEGIT => "LEGIT",
         ClaimStatus::SUSPECT => "SUSPECT",
     }
 }
 
 fn recompute_product_status(total_claims: i64, suspect_claims: i64) -> &'static str {
     // MVP rule (transparent + simple):
     // - SUSPECT if >= 3 suspect claims
     // - LIKELY_LEGIT if >= 5 total claims and suspect ratio < 20%
     // - else UNKNOWN
     if suspect_claims >= 3 {
         return "SUSPECT";
     }
     if total_claims >= 5 {
         let ratio = suspect_claims as f64 / total_claims as f64;
         if ratio < 0.2 {
             return "LIKELY_LEGIT";
         }
     }
     "UNKNOWN"
 }
 
 #[utoipa::path(
     post,
     path = "/claims",
     tag = "claims",
     security(
         ("bearerAuth" = [])
     ),
     request_body = CreateClaimRequest,
     responses(
         (status = 200, description = "Claim created", body = CreateClaimResponse),
         (status = 401, description = "Unauthorized")
     )
 )]
 pub async fn create_claim(
     State(state): State<AppState>,
     headers: HeaderMap,
     Json(req): Json<CreateClaimRequest>,
 ) -> Result<Json<CreateClaimResponse>, ApiError> {
     let user_id = require_user_id(&state, &headers)?;
 
     let gtin = req.gtin.trim().to_string();
     if gtin.len() < 8 {
         return Err(ApiError::bad_request("gtin looks too short"));
     }
 
     let mut tx = state
         .db()
         .begin()
         .await
         .map_err(|e| ApiError::Internal(e.into()))?;
 
     let product_id: Uuid = if let Some(row) = sqlx::query(
         r#"
         select p.id
         from product_identifiers pi
         join products p on p.id = pi.product_id
         where pi.kind = 'GTIN' and pi.value = $1
         "#,
     )
     .bind(&gtin)
     .fetch_optional(&mut *tx)
     .await
     .map_err(|e| ApiError::Internal(e.into()))?
     {
         row.get("id")
     } else {
         let row = sqlx::query(
             r#"
             insert into products (name, brand, category, community_status)
             values ($1, $2, 'UNKNOWN', 'UNKNOWN')
             returning id
             "#,
         )
         .bind(req.product_name.as_deref())
         .bind(req.brand.as_deref())
         .fetch_one(&mut *tx)
         .await
         .map_err(|e| ApiError::Internal(e.into()))?;
 
         let id: Uuid = row.get("id");
 
         sqlx::query(
             r#"
             insert into product_identifiers (product_id, kind, value)
             values ($1, 'GTIN', $2)
             on conflict (kind, value) do nothing
             "#,
         )
         .bind(id)
         .bind(&gtin)
         .execute(&mut *tx)
         .await
         .map_err(|e| ApiError::Internal(e.into()))?;
 
         id
     };
 
     let claim_row = sqlx::query(
         r#"
         insert into claims (product_id, user_id, status, note)
         values ($1, $2, $3, $4)
         returning id
         "#,
     )
     .bind(product_id)
     .bind(user_id)
     .bind(status_to_str(&req.status))
     .bind(req.note.as_deref())
     .fetch_one(&mut *tx)
     .await
     .map_err(|e| ApiError::Internal(e.into()))?;
 
     let claim_id: Uuid = claim_row.get("id");
 
     let counts = sqlx::query(
         r#"
         select
             count(*) as total_claims,
             sum(case when status = 'SUSPECT' then 1 else 0 end) as suspect_claims
         from claims
         where product_id = $1
         "#,
     )
     .bind(product_id)
     .fetch_one(&mut *tx)
     .await
     .map_err(|e| ApiError::Internal(e.into()))?;
 
     let total_claims: i64 = counts.get("total_claims");
     let suspect_claims: i64 = counts
         .get::<Option<i64>, _>("suspect_claims")
         .unwrap_or(0);
 
     let new_status = recompute_product_status(total_claims, suspect_claims);
 
     sqlx::query("update products set community_status = $1 where id = $2")
         .bind(new_status)
         .bind(product_id)
         .execute(&mut *tx)
         .await
         .map_err(|e| ApiError::Internal(e.into()))?;
 
     tx.commit()
         .await
         .map_err(|e| ApiError::Internal(e.into()))?;
 
     Ok(Json(CreateClaimResponse {
         claim_id: claim_id.to_string(),
         product_id: product_id.to_string(),
         community_status: new_status.to_string(),
     }))
 }
 
