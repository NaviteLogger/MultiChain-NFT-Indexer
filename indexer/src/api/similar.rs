use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::validate::{is_evm_address, is_token_id};

#[derive(Debug, Deserialize)]
pub struct SimilarQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SimilarItem {
    pub chain_id: i64,
    pub contract: String,
    pub token_id: String,
    pub current_owner: String,
    pub uri: Option<String>,
    pub distance: f64,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path((contract, token_id)): Path<(String, String)>,
    Query(q): Query<SimilarQuery>,
) -> AppResult<Json<Vec<SimilarItem>>> {
    if !is_evm_address(&contract) {
        return Err(AppError::BadRequest("invalid contract".into()));
    }
    if !is_token_id(&token_id) {
        return Err(AppError::BadRequest("invalid token_id".into()));
    }
    let limit = q.limit.unwrap_or(10).clamp(1, 100);

    let rows = sqlx::query(
        r#"
        WITH anchor AS (
            SELECT embedding FROM nfts
            WHERE contract = $1 AND token_id = $2 AND embedding IS NOT NULL
            LIMIT 1
        )
        SELECT n.chain_id, n.contract, n.token_id, n.current_owner, n.uri,
               n.embedding <=> (SELECT embedding FROM anchor) AS distance
          FROM nfts n
         WHERE n.embedding IS NOT NULL
           AND NOT (n.contract = $1 AND n.token_id = $2)
         ORDER BY n.embedding <=> (SELECT embedding FROM anchor) ASC
         LIMIT $3
        "#,
    )
    .bind(contract.to_lowercase())
    .bind(&token_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| SimilarItem {
            chain_id: r.get("chain_id"),
            contract: r.get("contract"),
            token_id: r.get("token_id"),
            current_owner: r.get("current_owner"),
            uri: r.get("uri"),
            distance: r.get::<f64, _>("distance"),
        })
        .collect::<Vec<_>>();

    Ok(Json(items))
}
