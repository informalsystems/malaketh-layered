use crate::make_signers;
use crate::tx::{make_signed_eip1559_tx, make_signed_eip4844_tx};
use alloy_network::eip2718::Encodable2718;
use alloy_primitives::Address;
use alloy_signer_local::LocalSigner;
use color_eyre::eyre::{self, Result};
use core::time;
use k256::ecdsa::SigningKey;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::sleep;

pub struct Spammer {
    /// Client for Ethereum RPC node server.
    client: RpcClient,
    /// Ethereum transaction signer.
    signer: LocalSigner<SigningKey>,
    /// Maximum number of transactions to send.
    max_num_txs: u64,
    /// Maximum number of transactions to send per second.
    max_rate: u64,
    /// Whether to send EIP-4844 blob transactions.
    blobs: bool,
}

impl Spammer {
    pub fn new(url: Url, max_num_txs: u64, max_rate: u64, blobs: bool) -> Result<Self> {
        // Initialize runtime, RPC client, and signers.
        let client = RpcClient::new(url);
        let signers = make_signers();

        // Pick one signer to send transactions from.
        let signer = signers[0].clone();

        Ok(Self {
            client,
            signer,
            max_num_txs,
            max_rate,
            blobs,
        })
    }

    pub fn run(&self) -> Result<()> {
        let rt = Runtime::new()?;
        rt.block_on(self.spam())
            .expect("Failed to create transaction");
        Ok(())
    }

    async fn spam(&self) -> Result<()> {
        // Spawn ticker.
        let (tick_sender, tick_receiver) = channel::<()>();
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_secs(1));
            tick_sender.send(()).unwrap();
        });

        // Fetch latest nonce for the sender address.
        let address = self.signer.address();
        let latest_nonce = self.get_latest_nonce(address).await?;
        println!("Spamming {address} starting from nonce={latest_nonce}");

        // Initialize nonce and counters.
        let mut nonce = latest_nonce;
        let mut sent_succeed = 0u64;
        let mut last_sent_succeed = 0u64;
        let mut sent_bytes = 0u64;
        let mut last_sent_bytes = 0u64;
        let mut sent_failed = HashMap::<String, usize>::new(); // counters for each kind of error
        let start_time = std::time::Instant::now();
        loop {
            // Create one transaction and sing it.
            let signed_tx = if self.blobs {
                make_signed_eip4844_tx(&self.signer, nonce).await?
            } else {
                make_signed_eip1559_tx(&self.signer, nonce).await?
            };
            let tx_bytes = signed_tx.encoded_2718();
            let tx_bytes_len = tx_bytes.len() as u64;

            // Send transaction to Ethereum RPC endpoint.
            let payload = hex::encode(tx_bytes);
            match self
                .client
                .rpc_request("eth_sendRawTransaction", json!([payload]))
                .await
            {
                Ok(_response) => {
                    sent_bytes += tx_bytes_len;
                    sent_succeed += 1;
                }
                Err(error) => {
                    sent_failed
                        .entry(error.to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(0);
                }
            }

            nonce += 1;

            // Check if rate limit is reached and show stats every ~one second.
            let sent_txs_last_second = sent_succeed - last_sent_succeed;
            if sent_txs_last_second >= self.max_rate || tick_receiver.try_recv().is_ok() {
                if sent_succeed >= self.max_rate {
                    // Block until next tick, so we don't exceed the rate limit.
                    tick_receiver.recv().unwrap();
                }

                // Show stats
                let elapsed = start_time.elapsed().as_secs_f64();
                let stats = format!(
                    "elapsed {:.3}s: Sent {} txs ({} bytes)",
                    elapsed,
                    sent_txs_last_second,
                    sent_bytes - last_sent_bytes,
                );
                let stats_failed = if sent_failed.is_empty() {
                    String::new()
                } else {
                    format!("; failed {} with {:?}", sent_failed.len(), sent_failed)
                };
                println!(
                    "{stats}{stats_failed}; tx/s={:.1}",
                    sent_succeed as f64 / elapsed,
                );

                // Reset counters
                last_sent_succeed = sent_succeed;
                last_sent_bytes = sent_bytes;
                sent_failed.clear();
            }

            // Stop if number of sent transactions exceeds the limit.
            if sent_succeed >= self.max_num_txs {
                break;
            }
        }
        let elapsed = start_time.elapsed().as_secs_f64();
        println!(
            "Sent {} txs ({} bytes) in {} seconds",
            sent_succeed, sent_bytes, elapsed
        );
        Ok(())
    }

    // Fetch from an Ethereum node the latest used nonce for the given address.
    async fn get_latest_nonce(&self, address: Address) -> Result<u64> {
        let response = self
            .client
            .rpc_request("eth_getTransactionCount", json!([address]))
            .await?;
        // Convert hex string to integer.
        let hex_str = response.as_str().strip_prefix("0x").unwrap();
        Ok(u64::from_str_radix(hex_str, 16)?)
    }
}

struct RpcClient {
    client: Client,
    url: Url,
}

impl RpcClient {
    pub fn new(url: Url) -> Self {
        let client = Client::new();
        Self { client, url }
    }

    pub async fn rpc_request(&self, method: &str, params: serde_json::Value) -> Result<String> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let request = self
            .client
            .post(self.url.clone())
            .timeout(Duration::from_secs(1))
            .header(CONTENT_TYPE, "application/json")
            .json(&body);
        let body: JsonResponseBody = request.send().await?.error_for_status()?.json().await?;

        if let Some(JsonError { code, message }) = body.error {
            match code {
                // txpool is full
                -32003 => {
                    // don't panic, just wait a bit to continue sending
                    println!("Warning: {message}");
                    sleep(Duration::from_secs(1)).await;
                    serde_json::from_value(body.result).map_err(Into::into)
                }
                _ => Err(eyre::eyre!("Server Error {}: {}", code, message)),
            }
        } else {
            serde_json::from_value(body.result).map_err(Into::into)
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonResponseBody {
    pub jsonrpc: String,
    #[serde(default)]
    pub error: Option<JsonError>,
    #[serde(default)]
    pub result: serde_json::Value,
    pub id: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonError {
    pub code: i64,
    pub message: String,
}
