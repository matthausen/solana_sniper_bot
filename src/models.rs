use serde::Deserialize;

// Pump.fun API structures
#[derive(Debug, Deserialize)]
pub struct PumpFunResponse {
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

// Token metadata structures (formerly from Moralis, now generic)
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub mint: Option<String>,
    pub standard: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub metaplex: Option<MetaplexMetadata>,
    pub decimals: Option<u8>,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub supply: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaplexMetadata {
    pub metadata_uri: Option<String>,
    pub update_authority: Option<String>,
    pub seller_fee_basis_points: Option<u32>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
}

// Holder statistics structures
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderStats {
    pub total: Option<u64>,
    pub supply_distribution: Option<SupplyDistribution>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyDistribution {
    pub top_10_holders_percentage: Option<f64>,
    pub top_20_holders_percentage: Option<f64>,
}

// Top holders structures
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopHoldersResponse {
    pub result: Option<Vec<TopHolder>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopHolder {
    pub owner_address: Option<String>,
    pub amount: Option<String>,
    pub amount_formatted: Option<String>,
    pub percentage_relative_to_total_supply: Option<f64>,
    pub usd_value: Option<f64>,
}

// DEX Screener structures
#[derive(Debug, Clone, Deserialize)]
pub struct DexScreenerPair {
    pub pairs: Option<Vec<DexPairInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DexPairInfo {
    pub liquidity_usd: Option<f64>,
    pub price_usd: Option<f64>,
}
