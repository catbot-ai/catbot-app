use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use strum_macros::{Display, EnumString};

#[derive(AsRefStr, Display, EnumString, Debug, Clone, Eq, PartialEq, Hash)]
pub enum TokenAddress {
    Address(String),
}

#[derive(
    Default,
    AsRefStr,
    Display,
    EnumString,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[strum(serialize_all = "UPPERCASE")]
pub enum TokenSymbol {
    #[default]
    SOL,
    JLP,
    JUP,
    USDC,
    #[allow(non_camel_case_types)]
    laineSOL,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenData {
    price: String,
    #[serde(rename = "type")]
    price_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PriceResponse {
    data: HashMap<String, TokenData>,
    time_taken: f64,
}

const JUP_API: &str = "https://api.jup.ag/price/v2";

pub async fn fetch_price(address: &str) -> Result<f64> {
    let url = format!("{JUP_API}?ids={}", address);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    response
        .data
        .get(address)
        .ok_or_else(|| anyhow!("Token {} not found", address))
        .and_then(|data| data.price.parse::<f64>().map_err(|e| anyhow!(e)))
}

pub async fn fetch_vs_price(base: &str, vs: &str) -> Result<f64> {
    let url = format!("{JUP_API}?ids={}&vsToken={}", base, vs);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    response
        .data
        .get(base)
        .ok_or_else(|| anyhow!("Base token {} not found", base))
        .and_then(|data| data.price.parse::<f64>().map_err(|e| anyhow!(e)))
}
