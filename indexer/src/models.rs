use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRow {
    pub id: i64,
    pub chain_id: i64,
    pub contract: String,
    pub event: String,
    pub from_address: Option<String>,
    pub to_address: String,
    pub token_id: String,
    pub uri: Option<String>,
    pub tx_hash: String,
    pub log_index: i64,
    pub block_number: i64,
    pub block_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub chain_id: u64,
    pub contract: String,
    pub event: String,
    pub from_address: Option<String>,
    pub to_address: String,
    pub token_id: String,
    pub uri: Option<String>,
    pub tx_hash: String,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct NftRow {
    pub chain_id: i64,
    pub contract: String,
    pub token_id: String,
    pub current_owner: String,
    pub uri: Option<String>,
    pub mint_block: i64,
    pub mint_tx: String,
    pub last_transfer_block: i64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsResponse {
    pub events_total: i64,
    pub nfts_total: i64,
    pub holders_total: i64,
    pub last_block: Option<i64>,
}
