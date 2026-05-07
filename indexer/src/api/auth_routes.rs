use std::str::FromStr;
use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use siwe::{generate_nonce, Message, VerificationOpts};

use crate::api::AppState;
use crate::auth;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct NonceResponse {
    pub nonce: String,
}

pub async fn nonce(State(state): State<Arc<AppState>>) -> AppResult<Json<NonceResponse>> {
    let nonce = generate_nonce();
    sqlx::query("INSERT INTO auth_nonces (nonce) VALUES ($1)")
        .bind(&nonce)
        .execute(&state.pool)
        .await?;
    Ok(Json(NonceResponse { nonce }))
}

#[derive(Debug, Deserialize)]
pub struct VerifyBody {
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub address: String,
    pub token: String,
}

pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VerifyBody>,
) -> AppResult<Json<VerifyResponse>> {
    let msg = Message::from_str(&body.message)
        .map_err(|e| AppError::BadRequest(format!("malformed SIWE message: {e}")))?;

    if msg.domain.to_string() != state.cfg.siwe_domain {
        return Err(AppError::BadRequest("SIWE domain does not match".into()));
    }

    let nonce_row: Option<(bool,)> =
        sqlx::query_as("SELECT consumed FROM auth_nonces WHERE nonce = $1")
            .bind(&msg.nonce)
            .fetch_optional(&state.pool)
            .await?;
    match nonce_row {
        None => return Err(AppError::BadRequest("unknown nonce".into())),
        Some((true,)) => return Err(AppError::BadRequest("nonce already used".into())),
        Some((false,)) => {}
    }

    let sig_bytes = decode_signature(&body.signature)?;

    let opts = VerificationOpts {
        domain: Some(
            state
                .cfg
                .siwe_domain
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("siwe domain parse: {e}")))?,
        ),
        nonce: Some(msg.nonce.clone()),
        timestamp: None,
    };

    msg.verify(&sig_bytes, &opts)
        .await
        .map_err(|e| AppError::BadRequest(format!("SIWE verify failed: {e}")))?;

    sqlx::query("UPDATE auth_nonces SET consumed = TRUE WHERE nonce = $1")
        .bind(&msg.nonce)
        .execute(&state.pool)
        .await?;

    let address = format!("0x{}", hex::encode(msg.address));
    let token = auth::issue(&state.cfg.jwt_secret, &address)?;

    Ok(Json(VerifyResponse { address, token }))
}

fn decode_signature(s: &str) -> AppResult<[u8; 65]> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).map_err(|_| AppError::BadRequest("bad signature hex".into()))?;
    if bytes.len() != 65 {
        return Err(AppError::BadRequest("signature must be 65 bytes".into()));
    }
    let mut out = [0u8; 65];
    out.copy_from_slice(&bytes);
    Ok(out)
}
