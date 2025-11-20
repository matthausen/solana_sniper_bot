use anyhow::Result;
use serde::Deserialize;
use reqwest::Client;
use crate::strategy::TokenEvent;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Scanner {
    client: Client,
    moralis_key: Option<String>,
    solscan_key: Option<String>,
    dexscreener_key: Option<String>,
}

impl Scanner {
    pub fn new(moralis_key: Option<String>, solscan_key: Option<String>, dexscreener_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("sol-memebot/0.1")
            .build()
            .unwrap();
        Scanner { client, moralis_key, solscan_key, dexscreener_key }
    }

    /// Fetch recent new mints / token listings from Pump.fun using the Moralis API
    pub async fn fetch_pumpfun_listings(&self) -> Result<Vec<PumpFunListing>> {
        // Example endpoint (placeholder) — replace with the real Pump.fun endpoint
        let url = "https://solana-gateway.moralis.io/token/mainnet/exchange/pumpfun/new?limit=100";
        let mut req = self.client.get(url);
        if let Some(k) = &self.moralis_key { req = req.header("X-API-Key", k); }
        let resp = req.send().await?.error_for_status()?;
        let status = resp.status();
        let body = resp.text().await?;
        // Print the raw API response for debugging/inspection
        println!("[fetch_pumpfun_listings] URL={} STATUS={} RESPONSE_BODY={}", url, status, body);

        let listings: Vec<PumpFunListing> = serde_json::from_str(&body).unwrap_or_default();
        Ok(listings)
    }

    /// Query Solscan to get token metadata (holders, supply distribution)
    pub async fn query_solscan_token(&self, mint: &str) -> Result<Option<SolscanToken>> {
        // Solscan API endpoint pattern (real-ish)
        let url = format!("https://api.solscan.io/token/meta?tokenAddress={}", mint);
        let mut req = self.client.get(&url);
        if let Some(k) = &self.solscan_key { req = req.header("api-key", k); }
        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        println!("[query_solscan_token] URL={} STATUS={} RESPONSE_BODY={}", url, status, body);
        if status.is_success() {
            // API returns { success: bool, data: { ... } }
            let r: SolscanResponse = serde_json::from_str(&body)?;
            Ok(r.data)
        } else {
            Ok(None)
        }
    }

    /// Query DEX-Screener for liquidity information
    pub async fn query_dexscreener_pair(&self, mint: &str) -> Result<Option<DexScreenerPair>> {
        // DexScreener API (placeholder)
        // Real endpoint: https://api.dexscreener.com/latest/dex/tokens/{chain}/{token_address}
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/solana/{}", mint);
        let mut req = self.client.get(&url);
        if let Some(k) = &self.dexscreener_key { req = req.header("x-api-key", k); }
        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        println!("[query_dexscreener_pair] URL={} STATUS={} RESPONSE_BODY={}", url, status, body);
        if status.is_success() {
            let p: DexScreenerPair = serde_json::from_str(&body)?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }
}

// Placeholder structs based on expected API shapes. Adjust to match real API responses.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PumpFunListing {
    pub token_address: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub logo: Option<String>,
    pub decimals: Option<String>,
    pub price_native: Option<String>,
    pub price_usd: Option<String>,
    pub liquidity: Option<String>,
    pub fully_diluted_valuation: Option<String>,
    pub created_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SolscanResponse {
    pub success: bool,
    pub data: Option<SolscanToken>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SolscanToken {
    pub address: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub icon: Option<String>,
    pub decimals: Option<u8>,
    #[serde(rename = "holder")]
    pub holders: Option<u64>,
    pub creator: Option<String>,
    pub create_tx: Option<String>,
    pub created_time: Option<u64>,
    pub metadata: Option<serde_json::Value>,
    pub metadata_uri: Option<String>,
    pub mint_authority: Option<Option<String>>,
    pub freeze_authority: Option<Option<String>>,
    pub supply: Option<String>,
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<i64>,
    pub price_change_24h: Option<f64>,
    pub total_dex_vol_24h: Option<f64>,
    pub dex_vol_change_24h: Option<f64>,
    pub owner_amount: Option<Vec<(String, f64)>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DexScreenerPair {
    pub pairs: Option<Vec<DexPairInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DexPairInfo {
    pub liquidity_usd: Option<f64>,
    pub price_usd: Option<f64>,
}

impl From<PumpFunListing> for TokenEvent {
    fn from(p: PumpFunListing) -> Self {
        // helper to parse optional numeric strings
        fn parse_opt_f64(s: Option<String>) -> f64 {
            s.as_deref().and_then(|v| v.replace(',', "").parse::<f64>().ok()).unwrap_or(0.0)
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
        }
    }
}