use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct JsonRpcClient {
    client: reqwest::Client,
    url: String,
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    message: String,
}

impl JsonRpcClient {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }

    async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .context("Failed to send JSON-RPC request")?
            .json::<JsonRpcResponse>()
            .await
            .context("Failed to parse JSON-RPC response")?;

        if let Some(error) = response.error {
            return Err(anyhow!("JSON-RPC error: {}", error.message));
        }

        response
            .result
            .ok_or_else(|| anyhow!("JSON-RPC response missing result"))
    }

    pub async fn get_transaction_count(&self, address: &str) -> Result<u64> {
        let result = self
            .call("eth_getTransactionCount", json!([address, "pending"]))
            .await?;
        let hex_str = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid nonce format"))?;
        let nonce = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)?;
        Ok(nonce)
    }

    pub async fn get_gas_price(&self) -> Result<u128> {
        let result = self.call("eth_gasPrice", json!([])).await?;
        let hex_str = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid gas price format"))?;
        let gas_price = u128::from_str_radix(hex_str.trim_start_matches("0x"), 16)?;
        Ok(gas_price)
    }

    pub async fn estimate_gas(&self, from: &str, to: &str, value: &str) -> Result<u64> {
        let result = self
            .call(
                "eth_estimateGas",
                json!([{
                    "from": from,
                    "to": to,
                    "value": value
                }]),
            )
            .await;

        match result {
            Ok(val) => {
                let hex_str = val.as_str().ok_or_else(|| anyhow!("Invalid gas format"))?;
                let gas = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)?;
                Ok(gas)
            }
            Err(_) => {
                // Fallback to a reasonable default if estimation fails
                Ok(21000)
            }
        }
    }

    pub async fn send_raw_transaction(&self, raw_tx: &str) -> Result<String> {
        let result = self
            .call("eth_sendRawTransaction", json!([raw_tx]))
            .await?;
        let tx_hash = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid transaction hash format"))?
            .to_string();
        Ok(tx_hash)
    }
}

