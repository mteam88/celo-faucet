use alloy_consensus::TxLegacy;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_rlp::Encodable;
use alloy_signer_local::PrivateKeySigner;
use anyhow::{Context, Result};

pub struct TxBuilder {
    signer: PrivateKeySigner,
    chain_id: u64,
}

impl TxBuilder {
    pub fn new(private_key: &str, chain_id: u64) -> Result<Self> {
        let signer = private_key
            .parse::<PrivateKeySigner>()
            .context("Failed to parse private key")?;
        Ok(Self { signer, chain_id })
    }

    pub fn faucet_address(&self) -> Address {
        self.signer.address()
    }

    pub async fn build_and_sign(
        &self,
        to: Address,
        value: U256,
        nonce: u64,
        gas_price: u128,
        gas_limit: u64,
    ) -> Result<String> {
        use alloy_network::TxSigner;

        let tx = TxLegacy {
            chain_id: Some(self.chain_id),
            nonce,
            gas_price,
            gas_limit,
            to: TxKind::Call(to),
            value,
            input: Bytes::new(),
        };

        let signature = self
            .signer
            .sign_transaction(&mut tx.clone())
            .await
            .context("Failed to sign transaction")?;

        // Manually RLP encode the signed legacy transaction
        // [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
        let mut buf = Vec::new();
        
        let list_header = alloy_rlp::Header {
            list: true,
            payload_length: tx.nonce.length()
                + tx.gas_price.length()
                + tx.gas_limit.length()
                + tx.to.length()
                + tx.value.length()
                + tx.input.length()
                + signature.v().length()
                + signature.r().length()
                + signature.s().length(),
        };
        
        list_header.encode(&mut buf);
        tx.nonce.encode(&mut buf);
        tx.gas_price.encode(&mut buf);
        tx.gas_limit.encode(&mut buf);
        tx.to.encode(&mut buf);
        tx.value.encode(&mut buf);
        tx.input.encode(&mut buf);
        signature.v().encode(&mut buf);
        signature.r().encode(&mut buf);
        signature.s().encode(&mut buf);

        Ok(format!("0x{}", hex::encode(buf)))
    }
}
