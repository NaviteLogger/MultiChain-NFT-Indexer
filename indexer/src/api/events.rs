use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::models::EventRow;
use crate::validate::is_evm_address;

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub chain_id: Option<i64>,
    pub contract: Option<String>,
    pub limit: Option<i64>,
    pub before_id: Option<i64>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<EventsQuery>,
) -> AppResult<Json<Vec<EventRow>>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let before_id = q.before_id.unwrap_or(i64::MAX);

    if let Some(c) = &q.contract {
        if !is_evm_address(c) {
            return Err(AppError::BadRequest("invalid 'contract'".into()));
        }
    }

    let rows = sqlx::query_as::<_, EventRow>(
        r#"
        SELECT id, chain_id, contract, event, from_address, to_address,
               token_id, uri, tx_hash, log_index, block_number, block_timestamp
        FROM events
        WHERE id < $1
          AND ($2::BIGINT IS NULL OR chain_id = $2)
          AND ($3::TEXT  IS NULL OR contract = $3)
        ORDER BY id DESC
        LIMIT $4
        "#,
    )
    .bind(before_id)
    .bind(q.chain_id)
    .bind(q.contract.as_deref().map(str::to_lowercase))
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

