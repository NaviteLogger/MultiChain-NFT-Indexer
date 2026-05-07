use std::sync::Arc;

use axum::{extract::State, Json};

use crate::api::AppState;
use crate::error::AppResult;
use crate::models::StatsResponse;

pub async fn get(State(state): State<Arc<AppState>>) -> AppResult<Json<StatsResponse>> {
    let events_total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events").fetch_one(&state.pool).await?;
    let nfts_total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM nfts").fetch_one(&state.pool).await?;
    let holders_total: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT current_owner) FROM nfts")
            .fetch_one(&state.pool)
            .await?;
    let last_block: Option<i64> =
        sqlx::query_scalar("SELECT MAX(last_block) FROM indexer_state")
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(StatsResponse {
        events_total,
        nfts_total,
        holders_total,
        last_block,
    }))
}
