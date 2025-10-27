mod config;
mod faucet;
mod http;
mod jsonrpc;
mod logging;
mod store;
mod telegram;
mod tx;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = config::Config::from_env()?;

    // Initialize logging
    logging::init_tracing(config.tracing_json);

    info!("Starting Celo Faucet...");
    info!("RPC URL: {}", config.rpc_url);
    info!("Chain ID: {}", config.chain_id);
    info!("Bind address: {}", config.bind_addr);

    // Initialize store
    let store = Arc::new(store::Store::new(&config.state_path)?);
    info!("State store initialized at: {}", config.state_path);

    // Initialize faucet service
    let faucet_service = Arc::new(faucet::FaucetService::new(
        config.rpc_url.clone(),
        &config.faucet_private_key,
        config.chain_id,
        &config.faucet_amount_wei,
        store,
    )?);

    info!("Faucet address: {:?}", faucet_service.faucet_address());
    info!("Amount per request: {} wei", config.faucet_amount_wei);

    // Spawn Telegram bot if token is provided
    if let Some(token) = config.telegram_bot_token.clone() {
        let bot_service = faucet_service.clone();
        tokio::spawn(async move {
            telegram::run_bot(token, bot_service).await;
        });
        info!("Telegram bot started");
    } else {
        info!("Telegram bot disabled (no token provided)");
    }

    // Start HTTP server
    let router = http::create_router(faucet_service);
    
    use salvo::conn::TcpListener;
    use salvo::Listener;
    
    let acceptor = TcpListener::new(&config.bind_addr).bind().await;

    info!("HTTP server listening on {}", config.bind_addr);
    info!("Web UI available at http://{}", config.bind_addr);

    salvo::Server::new(acceptor).serve(router).await;

    Ok(())
}
