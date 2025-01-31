use anyhow::anyhow;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use strum_macros::{Display, EnumString};

// Define the TokenId enum with strum conversions
#[derive(EnumString, Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum TokenId {
    #[strum(to_string = "So11111111111111111111111111111111111111112")]
    #[allow(non_camel_case_types)]
    SOL,
    #[strum(to_string = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4")]
    #[allow(non_camel_case_types)]
    JLP,
    #[strum(to_string = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")]
    #[allow(non_camel_case_types)]
    USDC,
}

#[derive(Default, AsRefStr, EnumString, Display, Debug, Copy, Clone, Eq, PartialEq)]
#[strum(serialize_all = "UPPERCASE")]
pub enum TokenSymbol {
    #[strum(serialize = "SOL")]
    #[allow(non_camel_case_types)]
    #[default]
    SOL,
    #[strum(serialize = "JLP")]
    #[allow(non_camel_case_types)]
    JLP,
    #[strum(serialize = "USDC")]
    #[allow(non_camel_case_types)]
    USDC,
}

// Response structures
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    id: String,
    #[serde(rename = "type")]
    price_type: String,
    price: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    data: HashMap<String, TokenData>,
    time_taken: f64,
}

const JUP_API: &str = "https://api.jup.ag/price/v2";

// Fetch price in USDC
#[allow(dead_code)]
pub async fn fetch_price_from_jup_in_usdc(token: TokenId) -> anyhow::Result<f64> {
    let url = format!("{JUP_API}?ids={}", token);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    let token_str = token.to_string();
    response
        .data
        .get(&token_str)
        .ok_or_else(|| anyhow!("Token {} not found in response", token))
        .and_then(|data| data.price.parse::<f64>().map_err(|e| anyhow!(e)))
}

// Fetch vs price between two tokens
#[allow(dead_code)]
pub async fn fetch_vs_price_from_jup(
    base_token: TokenId,
    vs_token: TokenId,
) -> anyhow::Result<f64> {
    let url = format!("{JUP_API}?ids={}&vsToken={}", base_token, vs_token);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    let token_str = base_token.to_string();
    response
        .data
        .get(&token_str)
        .ok_or_else(|| anyhow!("Base token {} not found in response", base_token))
        .and_then(|data| data.price.parse::<f64>().map_err(|e| anyhow!(e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_sol_price() {
        let price = fetch_price_from_jup_in_usdc(TokenId::SOL).await.unwrap();
        assert!(price > 0.0);
    }

    #[tokio::test]
    async fn test_fetch_jlp_price() {
        let price = fetch_price_from_jup_in_usdc(TokenId::JLP).await.unwrap();
        assert!(price > 0.0);
    }

    #[tokio::test]
    async fn test_sol_vs_usdc() {
        let price1 = fetch_price_from_jup_in_usdc(TokenId::SOL).await.unwrap();
        let price2 = fetch_vs_price_from_jup(TokenId::SOL, TokenId::USDC)
            .await
            .unwrap();
        assert!(price1 > 0.0);
        assert!(price2 > 0.0);
    }

    #[tokio::test]
    async fn test_usdc_self_price() {
        let price = fetch_vs_price_from_jup(TokenId::USDC, TokenId::USDC)
            .await
            .unwrap();
        assert!((price - 1.0).abs() < f64::EPSILON);
    }
}
