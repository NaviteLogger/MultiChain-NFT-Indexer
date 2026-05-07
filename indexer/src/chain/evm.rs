use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use alloy::primitives::{Address, B256};
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::eth::Filter;
use alloy::sol;
use alloy::sol_types::SolEvent;
use chrono::Utc;
use futures::StreamExt;

use crate::api::AppState;
use crate::embed::embed;
use crate::models::EventEnvelope;

sol! {
    #[derive(Debug)]
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);

    #[derive(Debug)]
    event Minted(address indexed to, uint256 indexed tokenId, string uri);
}

pub async fn run(state: Arc<AppState>, ws_url: String) -> anyhow::Result<()> {
    loop {
        match watch_once(state.clone(), &ws_url).await {
            Ok(()) => {
                tracing::warn!("watcher stream ended cleanly; reconnecting in 5s");
            }
            Err(err) => {
                tracing::error!(error = ?err, "watcher errored; reconnecting in 5s");
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn watch_once(state: Arc<AppState>, ws_url: &str) -> anyhow::Result<()> {
    let ws = WsConnect::new(ws_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let mut filter = Filter::new().from_block(state.cfg.evm_start_block);

    if let Some(c) = &state.cfg.evm_contract {
        let addr = Address::from_str(c)?;
        filter = filter.address(addr);
    }

    let signatures = vec![Transfer::SIGNATURE_HASH, Minted::SIGNATURE_HASH];
    filter = filter.event_signature(signatures);

    tracing::info!(
        chain_id = state.cfg.evm_chain_id,
        contract = ?state.cfg.evm_contract,
        from_block = state.cfg.evm_start_block,
        "subscribing to logs"
    );

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        if let Err(err) = handle_log(&state, log).await {
            tracing::warn!(error = ?err, "failed to handle log");
        }
    }

    Ok(())
}

async fn handle_log(
    state: &Arc<AppState>,
    log: alloy::rpc::types::eth::Log,
) -> anyhow::Result<()> {
    let topic0 = log
        .topic0()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("log has no topic0"))?;
    let contract_lc = format!("0x{}", hex::encode(log.address()));
    let tx_hash = log
        .transaction_hash
        .map(|h| format!("0x{}", hex::encode(h)))
        .unwrap_or_default();
    let log_index = log.log_index.unwrap_or_default() as i64;
    let block_number = log.block_number.unwrap_or_default() as i64;
    let block_timestamp = Utc::now();

    if topic0 == Transfer::SIGNATURE_HASH {
        let decoded = Transfer::decode_log(&log.inner, true)?.data;
        let from = format!("0x{}", hex::encode(decoded.from));
        let to = format!("0x{}", hex::encode(decoded.to));
        let token_id = decoded.tokenId.to_string();

        let envelope = EventEnvelope {
            chain_id: state.cfg.evm_chain_id,
            contract: contract_lc.clone(),
            event: "Transfer".into(),
            from_address: Some(from.clone()),
            to_address: to.clone(),
            token_id: token_id.clone(),
            uri: None,
            tx_hash: tx_hash.clone(),
            log_index: log_index as u64,
            block_number: block_number as u64,
            block_timestamp,
        };

        persist_event(state, &envelope).await?;
        upsert_nft_owner(state, &envelope, from == zero_addr()).await?;
        let _ = state.event_tx.send(envelope);
    } else if topic0 == Minted::SIGNATURE_HASH {
        let decoded = Minted::decode_log(&log.inner, true)?.data;
        let to = format!("0x{}", hex::encode(decoded.to));
        let token_id = decoded.tokenId.to_string();
        let uri = decoded.uri.clone();

        let envelope = EventEnvelope {
            chain_id: state.cfg.evm_chain_id,
            contract: contract_lc.clone(),
            event: "Minted".into(),
            from_address: None,
            to_address: to.clone(),
            token_id: token_id.clone(),
            uri: Some(uri.clone()),
            tx_hash: tx_hash.clone(),
            log_index: log_index as u64,
            block_number: block_number as u64,
            block_timestamp,
        };

        persist_event(state, &envelope).await?;
        attach_uri(state, &envelope).await?;
        let _ = state.event_tx.send(envelope);
    }

    update_high_watermark(state, block_number).await?;
    Ok(())
}

async fn persist_event(state: &Arc<AppState>, env: &EventEnvelope) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO events (chain_id, contract, event, from_address, to_address,
                            token_id, uri, tx_hash, log_index, block_number, block_timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (chain_id, tx_hash, log_index) DO NOTHING
        "#,
    )
    .bind(env.chain_id as i64)
    .bind(&env.contract)
    .bind(&env.event)
    .bind(&env.from_address)
    .bind(&env.to_address)
    .bind(&env.token_id)
    .bind(&env.uri)
    .bind(&env.tx_hash)
    .bind(env.log_index as i64)
    .bind(env.block_number as i64)
    .bind(env.block_timestamp)
    .execute(&state.pool)
    .await?;
    Ok(())
}

async fn upsert_nft_owner(
    state: &Arc<AppState>,
    env: &EventEnvelope,
    is_mint: bool,
) -> anyhow::Result<()> {
    if is_mint {
        sqlx::query(
            r#"
            INSERT INTO nfts (chain_id, contract, token_id, current_owner, mint_block, mint_tx, last_transfer_block, last_updated)
            VALUES ($1, $2, $3, $4, $5, $6, $5, NOW())
            ON CONFLICT (chain_id, contract, token_id)
            DO UPDATE SET current_owner = EXCLUDED.current_owner,
                          last_transfer_block = EXCLUDED.last_transfer_block,
                          last_updated = NOW()
            "#,
        )
        .bind(env.chain_id as i64)
        .bind(&env.contract)
        .bind(&env.token_id)
        .bind(&env.to_address)
        .bind(env.block_number as i64)
        .bind(&env.tx_hash)
        .execute(&state.pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE nfts
               SET current_owner = $4,
                   last_transfer_block = $5,
                   last_updated = NOW()
             WHERE chain_id = $1 AND contract = $2 AND token_id = $3
            "#,
        )
        .bind(env.chain_id as i64)
        .bind(&env.contract)
        .bind(&env.token_id)
        .bind(&env.to_address)
        .bind(env.block_number as i64)
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}

async fn attach_uri(state: &Arc<AppState>, env: &EventEnvelope) -> anyhow::Result<()> {
    let uri = env.uri.as_deref().unwrap_or("");
    let text = format!(
        "{contract} {token_id} {uri}",
        contract = env.contract,
        token_id = env.token_id,
        uri = uri
    );
    let vec = embed(&text);
    let pgvec = format_pgvector(&vec);

    sqlx::query(
        r#"
        UPDATE nfts SET uri = $4,
                        embedding = $5::vector,
                        last_updated = NOW()
         WHERE chain_id = $1 AND contract = $2 AND token_id = $3
        "#,
    )
    .bind(env.chain_id as i64)
    .bind(&env.contract)
    .bind(&env.token_id)
    .bind(uri)
    .bind(&pgvec)
    .execute(&state.pool)
    .await?;
    Ok(())
}

fn format_pgvector(v: &[f32]) -> String {
    let inner = v
        .iter()
        .map(|x| format!("{x:.6}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{inner}]")
}

async fn update_high_watermark(state: &Arc<AppState>, block_number: i64) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO indexer_state (chain_id, last_block, last_block_time)
        VALUES ($1, $2, NOW())
        ON CONFLICT (chain_id) DO UPDATE SET
            last_block = GREATEST(indexer_state.last_block, EXCLUDED.last_block),
            last_block_time = NOW()
        "#,
    )
    .bind(state.cfg.evm_chain_id as i64)
    .bind(block_number)
    .execute(&state.pool)
    .await?;
    Ok(())
}

fn zero_addr() -> String {
    "0x0000000000000000000000000000000000000000".into()
}

#[allow(dead_code)]
fn parse_b256(s: &str) -> anyhow::Result<B256> {
    let trimmed = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(trimmed)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(B256::from(out))
}
