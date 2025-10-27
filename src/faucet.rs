use crate::jsonrpc::JsonRpcClient;
use crate::store::Store;
use crate::tx::TxBuilder;
use alloy_primitives::{Address, U256};
use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct FaucetService {
    rpc: JsonRpcClient,
    tx_builder: TxBuilder,
    store: Arc<Store>,
    amount_wei: U256,
    send_mutex: Arc<Mutex<()>>,
}

impl FaucetService {
    pub fn new(
        rpc_url: String,
        private_key: &str,
        chain_id: u64,
        amount_wei: &str,
        store: Arc<Store>,
    ) -> Result<Self> {
        let rpc = JsonRpcClient::new(rpc_url);
        let tx_builder = TxBuilder::new(private_key, chain_id)?;
        let amount_wei =
            U256::from_str_radix(amount_wei, 10).context("Failed to parse FAUCET_AMOUNT_WEI")?;

        Ok(Self {
            rpc,
            tx_builder,
            store,
            amount_wei,
            send_mutex: Arc::new(Mutex::new(())),
        })
    }

    pub fn faucet_address(&self) -> Address {
        self.tx_builder.faucet_address()
    }

    #[tracing::instrument(skip(self), fields(to = %to_address))]
    pub async fn send_native(&self, to_address: &str) -> Result<String> {
        // Validate address
        let to = to_address
            .parse::<Address>()
            .context("Invalid Ethereum address")?;

        info!("Processing faucet request for {}", to);

        // Check if address has already received tokens
        if self.store.has_received(&to.to_string())? {
            warn!("Address {} has already received tokens", to);
            return Err(anyhow!("already_sent"));
        }

        // Acquire mutex to serialize sends and avoid nonce races
        let _guard = self.send_mutex.lock().await;

        let faucet_addr = self.tx_builder.faucet_address();
        let faucet_addr_str = format!("{:?}", faucet_addr);

        // Fetch nonce
        let nonce = self
            .rpc
            .get_transaction_count(&faucet_addr_str)
            .await
            .context("Failed to get transaction count")?;

        // Fetch gas price
        let gas_price = self
            .rpc
            .get_gas_price()
            .await
            .context("Failed to get gas price")?;

        // Estimate gas
        let gas_limit = self
            .rpc
            .estimate_gas(
                &faucet_addr_str,
                &format!("{:?}", to),
                &format!("{:#x}", self.amount_wei),
            )
            .await
            .context("Failed to estimate gas")?;

        info!(
            "Building transaction: nonce={}, gas_price={}, gas_limit={}",
            nonce, gas_price, gas_limit
        );

        // Build and sign transaction
        let raw_tx = self
            .tx_builder
            .build_and_sign(to, self.amount_wei, nonce, gas_price, gas_limit)
            .await
            .context("Failed to build and sign transaction")?;

        // Send transaction
        let tx_hash = self
            .rpc
            .send_raw_transaction(&raw_tx)
            .await
            .context("Failed to send transaction")?;

        info!("Transaction sent: {}", tx_hash);

        // Mark address as received
        self.store
            .mark_received(&to.to_string())
            .context("Failed to mark address as received")?;

        Ok(tx_hash)
    }
}
