use crate::models::*;
use crate::strategy::TokenEvent;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Scanner {
    client: Client,
    rpc_url: String,
    dexscreener_key: Option<String>,
}

// Solana RPC structures
#[derive(Debug, serde::Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ProgramAccount {
    pubkey: String,
    account: AccountData,
}

#[derive(Debug, Deserialize)]
struct AccountData {
    data: Vec<String>, // [base64_data, encoding]
    lamports: u64,
}

impl Scanner {
    pub fn new(dexscreener_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("sol-memebot/0.1")
            .build()
            .unwrap();

        Scanner {
            client,
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            dexscreener_key,
        }
    }

    /// Fetch recent new mints / token listings from Pump.fun using PumpPortal WebSocket (FREE)
    /// Connects to PumpPortal's free WebSocket API and listens for new token creation events
    pub async fn fetch_pumpfun_listings(&self) -> Result<Vec<PumpFunListing>> {
        use futures::{SinkExt, StreamExt};
        use tokio_tungstenite::{connect_async, tungstenite::Message};

        const PUMPPORTAL_WS: &str = "wss://pumpportal.fun/api/data";

        println!("[fetch_pumpfun_listings] Connecting to PumpPortal WebSocket...");

        // Connect to WebSocket
        let (ws_stream, _) = match connect_async(PUMPPORTAL_WS).await {
            Ok(conn) => conn,
            Err(e) => {
                println!(
                    "[fetch_pumpfun_listings] WebSocket connection failed: {}",
                    e
                );
                return Ok(Vec::new());
            }
        };

        println!("[fetch_pumpfun_listings] Connected! Subscribing to new tokens...");

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to new token events
        let subscribe_msg = serde_json::json!({
            "method": "subscribeNewToken"
        });

        if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
            println!("[fetch_pumpfun_listings] Failed to subscribe: {}", e);
            return Ok(Vec::new());
        }

        println!("[fetch_pumpfun_listings] Subscribed! Listening for new tokens (3 seconds)...");

        let mut listings = Vec::new();
        let start_time = std::time::Instant::now();
        let listen_duration = std::time::Duration::from_secs(3);

        // Listen for messages for 3 seconds
        while start_time.elapsed() < listen_duration {
            let timeout =
                tokio::time::timeout(std::time::Duration::from_millis(500), read.next()).await;

            match timeout {
                Ok(Some(Ok(Message::Text(text)))) => {
                    // Parse the message
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Extract token information from the event
                        if let Some(mint) = data.get("mint").and_then(|v| v.as_str()) {
                            println!("[fetch_pumpfun_listings] ✅ New token: {}", mint);

                            let listing = PumpFunListing {
                                token_address: mint.to_string(),
                                name: data
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                symbol: data
                                    .get("symbol")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                logo: data
                                    .get("uri")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .or_else(|| {
                                        data.get("image")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                    }),
                                decimals: Some("6".to_string()),
                                price_native: data
                                    .get("initialBuy")
                                    .and_then(|v| v.as_f64())
                                    .map(|p| p.to_string()),
                                price_usd: None,
                                liquidity: data
                                    .get("virtualSolReserves")
                                    .and_then(|v| v.as_f64())
                                    .map(|l| (l / 1_000_000_000.0).to_string()),
                                fully_diluted_valuation: data
                                    .get("marketCap")
                                    .and_then(|v| v.as_f64())
                                    .map(|m| m.to_string()),
                                created_at: Some(chrono::Utc::now().timestamp().to_string()),
                            };

                            listings.push(listing);
                        }
                    }
                }
                Ok(Some(Ok(Message::Close(_)))) => {
                    println!("[fetch_pumpfun_listings] WebSocket closed by server");
                    break;
                }
                Ok(Some(Err(e))) => {
                    println!("[fetch_pumpfun_listings] WebSocket error: {}", e);
                    break;
                }
                Ok(None) => {
                    println!("[fetch_pumpfun_listings] WebSocket stream ended");
                    break;
                }
                Err(_) => {
                    // Timeout - continue listening
                    continue;
                }
                _ => continue,
            }
        }

        println!(
            "[fetch_pumpfun_listings] Collected {} new tokens",
            listings.len()
        );
        Ok(listings)
    }

    /// Query Solana RPC to get token holder stats using HTTP JSON-RPC
    pub async fn query_token_holder_stats(&self, mint: &str) -> Result<Option<HolderStats>> {
        // Build RPC request for getProgramAccounts
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getProgramAccounts".to_string(),
            params: serde_json::json!([
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // SPL Token Program
                {
                    "encoding": "base64",
                    "filters": [
                        { "dataSize": 165 },
                        { "memcmp": { "offset": 0, "bytes": mint } }
                    ]
                }
            ]),
        };

        // Send HTTP request
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let rpc_response: RpcResponse<Vec<ProgramAccount>> = response.json().await?;

        if let Some(accounts) = rpc_response.result {
            let total_holders = accounts.len() as u64;
            Ok(Some(HolderStats {
                total: Some(total_holders),
                supply_distribution: None,
            }))
        } else {
            Ok(None)
        }
    }

    /// Query Solana RPC to get top token holders using HTTP JSON-RPC
    pub async fn query_token_top_holders(&self, mint: &str) -> Result<Option<TopHoldersResponse>> {
        // Build RPC request for getProgramAccounts
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getProgramAccounts".to_string(),
            params: serde_json::json!([
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // SPL Token Program
                {
                    "encoding": "base64",
                    "filters": [
                        { "dataSize": 165 },
                        { "memcmp": { "offset": 0, "bytes": mint } }
                    ]
                }
            ]),
        };

        // Send HTTP request
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let rpc_response: RpcResponse<Vec<ProgramAccount>> = response.json().await?;

        if let Some(accounts) = rpc_response.result {
            // Parse token account data to get balances
            let mut holders: Vec<(String, u64)> = Vec::new();
            let mut total_supply: u64 = 0;

            for account_info in accounts {
                // Decode base64 account data
                if let Some(base64_data) = account_info.account.data.get(0) {
                    use base64::Engine;
                    if let Ok(data) = base64::engine::general_purpose::STANDARD.decode(base64_data)
                    {
                        // Token account layout:
                        // 0-32: mint (32 bytes)
                        // 32-64: owner (32 bytes)
                        // 64-72: amount (8 bytes, little-endian u64)
                        if data.len() >= 72 {
                            let owner = bs58::encode(&data[32..64]).into_string();
                            let amount_bytes: [u8; 8] = data[64..72].try_into().unwrap_or([0; 8]);
                            let amount = u64::from_le_bytes(amount_bytes);

                            if amount > 0 {
                                holders.push((owner, amount));
                                total_supply += amount;
                            }
                        }
                    }
                }
            }

            // Sort by amount descending
            holders.sort_by(|a, b| b.1.cmp(&a.1));

            // Take top 20 holders
            let top_holders: Vec<TopHolder> = holders
                .iter()
                .take(20)
                .map(|(owner, amount)| {
                    let percentage = if total_supply > 0 {
                        (*amount as f64 / total_supply as f64) * 100.0
                    } else {
                        0.0
                    };

                    TopHolder {
                        owner_address: Some(owner.clone()),
                        amount: Some(amount.to_string()),
                        amount_formatted: Some(amount.to_string()),
                        percentage_relative_to_total_supply: Some(percentage),
                        usd_value: None,
                    }
                })
                .collect();

            Ok(Some(TopHoldersResponse {
                result: Some(top_holders),
            }))
        } else {
            Ok(None)
        }
    }

    /// Query DEX-Screener for liquidity information
    pub async fn query_dexscreener_pair(&self, mint: &str) -> Result<Option<DexScreenerPair>> {
        // DexScreener API (placeholder)
        // Real endpoint: https://api.dexscreener.com/latest/dex/tokens/{chain}/{token_address}
        let url = format!(
            "https://api.dexscreener.com/latest/dex/tokens/solana/{}",
            mint
        );
        let mut req = self.client.get(&url);
        if let Some(k) = &self.dexscreener_key {
            req = req.header("x-api-key", k);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        //println!("[query_dexscreener_pair] URL={} STATUS={} RESPONSE_BODY={}", url, status, body);
        if status.is_success() {
            let p: DexScreenerPair = serde_json::from_str(&body)?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }
}

impl From<PumpFunListing> for TokenEvent {
    fn from(p: PumpFunListing) -> Self {
        // helper to parse optional numeric strings
        fn parse_opt_f64(s: Option<String>) -> f64 {
            s.as_deref()
                .and_then(|v| v.replace(',', "").parse::<f64>().ok())
                .unwrap_or(0.0)
        }

        let market_cap = parse_opt_f64(p.fully_diluted_valuation);
        let base_price = parse_opt_f64(p.price_usd);
        let liquidity_usd = parse_opt_f64(p.liquidity);

        TokenEvent {
            id: p.token_address.clone(),
            token_type: p.symbol.unwrap_or_else(|| "unknown".to_string()),
            market_cap_usd: market_cap,
            dev_hold_pct: 0.0,
            liquidity_usd,
            holders: 0,
            upgradeable: false,
            freeze_authority: false,
            momentum: false,
            graduation: false,
            base_price,
            // New fields - will be populated by simulator
            dev_wallet_address: None,
            is_dev_known_rugger: false,
            entry_market_cap: market_cap,
            raydium_lp_detected: false,
        }
    }
}
