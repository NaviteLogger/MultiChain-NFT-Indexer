use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::models::NftRow;
use crate::validate::is_evm_address;

#[derive(Debug, Deserialize)]
pub struct HoldingsQuery {
    pub chain_id: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(q): Query<HoldingsQuery>,
) -> AppResult<Json<Vec<NftRow>>> {
    if !is_evm_address(&address) {
        return Err(AppError::BadRequest("invalid address".into()));
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 500);

    let rows = sqlx::query_as::<_, NftRow>(
        r#"
        SELECT chain_id, contract, token_id, current_owner, uri,
               mint_block, mint_tx, last_transfer_block, last_updated
        FROM nfts
        WHERE current_owner = $1
          AND ($2::BIGINT IS NULL OR chain_id = $2)
        ORDER BY last_transfer_block DESC
        LIMIT $3
        "#,
    )
    .bind(address.to_lowercase())
    .bind(q.chain_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

