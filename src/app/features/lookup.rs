 use crate::app::{errors::ApiError, state::AppState};
 use axum::extract::{Query, State};
 use serde::{Deserialize, Serialize};
 use sqlx::Row;
 use utoipa::ToSchema;
 
 #[derive(Debug, Deserialize)]
 pub struct LookupQuery {
     pub gtin: String,
 }
 
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
 pub enum CommunityStatus {
    Unknown,
    LikelyLegit,
    Suspect,
 }
 
 #[derive(Debug, Serialize, ToSchema)]
 pub struct LookupResponse {
     pub gtin: String,
     pub product_id: Option<String>,
     pub product_name: Option<String>,
     pub brand: Option<String>,
     pub community_status: CommunityStatus,
     pub total_claims: i64,
     pub suspect_claims: i64,
 }
 
 #[utoipa::path(
     get,
     path = "/lookup",
     tag = "lookup",
     params(
         ("gtin" = String, Query, description = "Barcode/GTIN")
     ),
     responses(
         (status = 200, description = "Lookup result", body = LookupResponse)
     )
 )]
 pub async fn lookup(
     State(state): State<AppState>,
     Query(q): Query<LookupQuery>,
 ) -> Result<axum::Json<LookupResponse>, ApiError> {
     let gtin = q.gtin.trim().to_string();
     if gtin.len() < 8 {
         return Err(ApiError::bad_request("gtin looks too short"));
     }
 
     let row = sqlx::query(
         r#"
         select
             p.id as product_id,
             p.name as product_name,
             p.brand as brand,
             p.community_status as community_status,
             coalesce(c.total_claims, 0) as total_claims,
             coalesce(c.suspect_claims, 0) as suspect_claims
         from product_identifiers pi
         join products p on p.id = pi.product_id
         left join (
             select
                 product_id,
                 count(*) as total_claims,
                 sum(case when status = 'SUSPECT' then 1 else 0 end) as suspect_claims
             from claims
             group by product_id
         ) c on c.product_id = p.id
         where pi.kind = 'GTIN' and pi.value = $1
         "#,
     )
     .bind(&gtin)
     .fetch_optional(state.db())
     .await
     .map_err(|e| ApiError::Internal(e.into()))?;
 
     if let Some(r) = row {
         let status_str: String = r.get("community_status");
         let status = match status_str.as_str() {
            "LIKELY_LEGIT" => CommunityStatus::LikelyLegit,
            "SUSPECT" => CommunityStatus::Suspect,
            _ => CommunityStatus::Unknown,
         };
 
         return Ok(axum::Json(LookupResponse {
             gtin,
             product_id: Some(r.get::<uuid::Uuid, _>("product_id").to_string()),
             product_name: r.get::<Option<String>, _>("product_name"),
             brand: r.get::<Option<String>, _>("brand"),
             community_status: status,
             total_claims: r.get::<i64, _>("total_claims"),
             suspect_claims: r.get::<i64, _>("suspect_claims"),
         }));
     }
 
     Ok(axum::Json(LookupResponse {
         gtin,
         product_id: None,
         product_name: None,
         brand: None,
        community_status: CommunityStatus::Unknown,
         total_claims: 0,
         suspect_claims: 0,
     }))
 }
 
