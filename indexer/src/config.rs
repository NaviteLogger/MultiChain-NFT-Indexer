use std::net::SocketAddr;

use anyhow::{anyhow, Context};

#[derive(Clone, Debug)]
pub struct Config {
    pub http_bind: SocketAddr,
    pub database_url: String,
    pub evm_ws_url: Option<String>,
    pub evm_chain_id: u64,
    pub evm_contract: Option<String>,
    pub evm_start_block: u64,
    pub jwt_secret: String,
    pub siwe_domain: String,
    pub siwe_origin: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let http_bind: SocketAddr = std::env::var("HTTP_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8080".into())
            .parse()
            .context("HTTP_BIND must be a valid socket addr")?;

        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;

        let evm_ws_url = std::env::var("EVM_WS_URL").ok().filter(|v| !v.is_empty());

        let evm_chain_id: u64 = std::env::var("EVM_CHAIN_ID")
            .unwrap_or_else(|_| "11155111".into())
            .parse()
            .context("EVM_CHAIN_ID must be a number")?;

        let evm_contract = std::env::var("EVM_CONTRACT").ok().filter(|v| !v.is_empty());

        let evm_start_block: u64 = std::env::var("EVM_START_BLOCK")
            .unwrap_or_else(|_| "0".into())
            .parse()
            .context("EVM_START_BLOCK must be a number")?;

        let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET is required")?;
        if jwt_secret.len() < 32 {
            return Err(anyhow!("JWT_SECRET must be at least 32 characters"));
        }

        let siwe_domain =
            std::env::var("SIWE_DOMAIN").unwrap_or_else(|_| "localhost:3000".into());
        let siwe_origin =
            std::env::var("SIWE_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());

        Ok(Self {
            http_bind,
            database_url,
            evm_ws_url,
            evm_chain_id,
            evm_contract,
            evm_start_block,
            jwt_secret,
            siwe_domain,
            siwe_origin,
        })
    }

    pub fn db_url_redacted(&self) -> String {
        url::Url::parse(&self.database_url)
            .map(|mut u| {
                let _ = u.set_password(Some("***"));
                u.to_string()
            })
            .unwrap_or_else(|_| "<unparseable>".into())
    }
}
