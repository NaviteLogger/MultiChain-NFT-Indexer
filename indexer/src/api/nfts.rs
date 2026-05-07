use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::models::NftRow;
use crate::validate::{is_evm_address, is_token_id};

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path((contract, token_id)): Path<(String, String)>,
) -> AppResult<Json<NftRow>> {
    if !is_evm_address(&contract) {
        return Err(AppError::BadRequest("invalid contract".into()));
    }
    if !is_token_id(&token_id) {
        return Err(AppError::BadRequest("invalid token_id".into()));
    }

    let row = sqlx::query_as::<_, NftRow>(
        r#"
        SELECT chain_id, contract, token_id, current_owner, uri,
               mint_block, mint_tx, last_transfer_block, last_updated
        FROM nfts
        WHERE contract = $1 AND token_id = $2
        ORDER BY chain_id ASC
        LIMIT 1
        "#,
    )
    .bind(contract.to_lowercase())
    .bind(token_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(row))
}

