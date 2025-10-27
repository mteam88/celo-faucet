use crate::store::Store;
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use url::Url;

pub struct FaucetService {
    rpc_url: String,
    signer: PrivateKeySigner,
    chain_id: u64,
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
        let signer = private_key
            .parse::<PrivateKeySigner>()
            .context("Failed to parse FAUCET_PRIVATE_KEY")?;
        let amount_wei =
            U256::from_str_radix(amount_wei, 10).context("Failed to parse FAUCET_AMOUNT_WEI")?;

        Ok(Self {
            rpc_url,
            signer,
            chain_id,
            store,
            amount_wei,
            send_mutex: Arc::new(Mutex::new(())),
        })
    }

    pub fn faucet_address(&self) -> Address {
        self.signer.address()
    }

    pub fn store(&self) -> &Store {
        &self.store
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

        let faucet_addr = self.faucet_address();

        // Build alloy provider with local wallet and recommended fillers
        let provider = ProviderBuilder::new()
            .wallet(self.signer.clone())
            .connect_http(self.rpc_url.parse::<Url>().unwrap());

        // Compose minimal transaction request; fillers will set nonce/gas/chain id
        let tx = TransactionRequest::default()
            .with_from(faucet_addr)
            .with_to(to)
            .with_chain_id(self.chain_id)
            .with_value(self.amount_wei);

        // Send via provider; returns tx hash
        let builder = provider
            .send_transaction(tx)
            .await
            .context("Failed to send transaction")?;

        let receipt = builder.get_receipt().await?;

        info!("Transaction receipt: {:?}", receipt);

        // Mark address as received
        self.store
            .mark_received(&to.to_string())
            .context("Failed to mark address as received")?;

        Ok(format!("0x{:x}", receipt.transaction_hash))
    }
}
