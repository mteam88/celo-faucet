use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub rpc_url: String,
    pub chain_id: u64,
    pub faucet_private_key: String,
    pub faucet_amount_wei: String,
    pub bind_addr: String,
    pub state_path: String,
    pub telegram_bot_token: Option<String>,
    pub tracing_json: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_url = env::var("RPC_URL").context("RPC_URL not set")?;
        let chain_id = env::var("CHAIN_ID")
            .context("CHAIN_ID not set")?
            .parse()
            .context("CHAIN_ID must be a valid u64")?;
        let faucet_private_key =
            env::var("FAUCET_PRIVATE_KEY").context("FAUCET_PRIVATE_KEY not set")?;
        let faucet_amount_wei =
            env::var("FAUCET_AMOUNT_WEI").context("FAUCET_AMOUNT_WEI not set")?;
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let state_path = env::var("STATE_PATH").unwrap_or_else(|_| "./state".to_string());
        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN").ok();
        let tracing_json = env::var("TRACING_JSON")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Ok(Self {
            rpc_url,
            chain_id,
            faucet_private_key,
            faucet_amount_wei,
            bind_addr,
            state_path,
            telegram_bot_token,
            tracing_json,
        })
    }
}
