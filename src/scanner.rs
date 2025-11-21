use crate::strategy::TokenEvent;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Scanner {
    client: Client,
    moralis_key: Option<String>,
    solscan_key: Option<String>,
    dexscreener_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PumpFunResponse {
    pub result: Option<Vec<PumpFunListing>>,
}

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
#[serde(rename_all = "camelCase")]
pub struct MoralisTokenMetadata {
    pub mint: Option<String>,
    pub standard: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub metaplex: Option<MoralisMetaplex>,
    pub decimals: Option<u8>,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub supply: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoralisMetaplex {
    pub metadata_uri: Option<String>,
    pub update_authority: Option<String>,
    pub seller_fee_basis_points: Option<u32>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoralisHolderStats {
    pub total: Option<u64>,
    pub supply_distribution: Option<MoralisSupplyDistribution>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoralisSupplyDistribution {
    pub top_10_holders_percentage: Option<f64>,
    pub top_20_holders_percentage: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoralisTopHoldersResponse {
    pub result: Option<Vec<MoralisTopHolder>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoralisTopHolder {
    pub owner_address: Option<String>,
    pub amount: Option<String>,
    pub amount_formatted: Option<String>,
    pub percentage_relative_to_total_supply: Option<f64>,
    pub usd_value: Option<f64>,
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

impl Scanner {
    pub fn new(
        moralis_key: Option<String>,
        solscan_key: Option<String>,
        dexscreener_key: Option<String>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("sol-memebot/0.1")
            .build()
            .unwrap();
        Scanner {
            client,
            moralis_key,
            solscan_key,
            dexscreener_key,
        }
    }

    /// Fetch recent new mints / token listings from Pump.fun using the Moralis API
    pub async fn fetch_pumpfun_listings(&self) -> Result<Vec<PumpFunListing>> {
        let url = "https://solana-gateway.moralis.io/token/mainnet/exchange/pumpfun/new?limit=100";
        let mut req = self.client.get(url);
        if let Some(k) = &self.moralis_key {
            req = req.header("X-API-Key", k);
        }
        let resp = req.send().await?.error_for_status()?;
        let body = resp.text().await?;

        //let status = resp.status();
        //println!("[fetch_pumpfun_listings] URL={} STATUS={} RESPONSE_BODY={}", url, status, body);

        let resp_wrapper: PumpFunResponse =
            serde_json::from_str(&body).unwrap_or(PumpFunResponse { result: None });
        Ok(resp_wrapper.result.unwrap_or_default())
    }

    /// Query Moralis to get token holder statistics
    pub async fn query_token_holder_stats(&self, mint: &str) -> Result<Option<MoralisHolderStats>> {
        let url = format!(
            "https://solana-gateway.moralis.io/token/mainnet/holders/{}",
            mint
        );
        let mut req = self.client.get(&url);
        if let Some(k) = &self.moralis_key {
            req = req.header("X-API-Key", k);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            let body = resp.text().await?;
            let stats: MoralisHolderStats = serde_json::from_str(&body)?;
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }

    /// Query Moralis to get top token holders
    pub async fn query_token_top_holders(
        &self,
        mint: &str,
    ) -> Result<Option<MoralisTopHoldersResponse>> {
        let url = format!(
            "https://solana-gateway.moralis.io/token/mainnet/{}/top-holders",
            mint
        );
        let mut req = self.client.get(&url);
        if let Some(k) = &self.moralis_key {
            req = req.header("X-API-Key", k);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            let body = resp.text().await?;
            let holders: MoralisTopHoldersResponse = serde_json::from_str(&body)?;
            Ok(Some(holders))
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
