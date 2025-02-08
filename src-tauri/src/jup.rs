use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use strum_macros::{Display, EnumString};

use crate::{fetcher::fetch_with_retry, token_registry::Token};

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
pub struct PriceResponse {
    data: HashMap<String, TokenData>,
    time_taken: f64,
}

const JUP_API: &str = "https://api.jup.ag/price/v2";

// Single token price
pub async fn fetch_price(address: &str) -> Result<f64> {
    let url = format!("{JUP_API}?ids={}", address);
    fetch_with_retry(&url, |response: PriceResponse| {
        response
            .data
            .get(address)
            .ok_or_else(|| anyhow!("Token {} not found", address))
            .and_then(|data| {
                data.price
                    .parse()
                    .map_err(|e| anyhow!("Parse error: {}", e))
            })
    })
    .await
}

// Token pair price
pub async fn fetch_pair_price(base: &str, vs: &str) -> Result<f64> {
    let url = format!("{JUP_API}?ids={}&vsToken={}", base, vs);
    fetch_with_retry(&url, |response: PriceResponse| {
        response
            .data
            .get(base)
            .ok_or_else(|| anyhow!("Base token {} not found", base))
            .and_then(|data| {
                data.price
                    .parse()
                    .map_err(|e| anyhow!("Parse error: {}", e))
            })
    })
    .await
}

// Multiple token prices
pub async fn fetch_many_prices(addresses: &[&str]) -> Result<HashMap<String, f64>> {
    let unique_addresses: Vec<&str> = addresses.to_vec();
    let params = unique_addresses.join(",");
    let url = format!("{JUP_API}?ids={}", params);

    fetch_with_retry(&url, |response: PriceResponse| {
        unique_addresses
            .iter()
            .map(|&addr| {
                let price = response
                    .data
                    .get(addr)
                    .ok_or_else(|| anyhow!("Token {} not found", addr))?
                    .price
                    .parse()
                    .map_err(|e| anyhow!("Parse error for {}: {}", addr, e))?;
                Ok((addr.to_string(), price))
            })
            .collect()
    })
    .await
}

/// Formats a price result into a user-friendly string.
pub fn format_price_result(result: Result<f64>) -> Option<String> {
    result
        .ok()
        .map(format_price)
        .or_else(|| Some("…".to_owned()))
}

/// Formats a price value into a user-friendly string.
pub fn format_price(price: f64) -> String {
    let price_str = price.to_string();
    format!("${}", &price_str[..7.min(price_str.len())])
}

/// Fetches and formats the price for a single token or a token pair.
pub async fn fetch_price_and_format(tokens: Vec<Token>) -> Option<String> {
    let is_pair = tokens.len() == 2;
    if !is_pair {
        format_price_result(fetch_price(&tokens[0].address).await)
    } else {
        format_price_result(fetch_pair_price(&tokens[0].address, &tokens[1].address).await)
    }
}

/// Fetches and formats prices for multiple tokens.
pub async fn fetch_many_price_and_format(tokens: Vec<Token>) -> Option<Vec<String>> {
    let addresses: Vec<&str> = tokens.iter().map(|token| token.address.as_str()).collect();
    let prices_result = fetch_many_prices(&addresses).await;
    prices_result
        .ok()
        .map(|prices| prices.into_values().map(format_price).collect::<Vec<_>>())
}
