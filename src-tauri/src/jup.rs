use anyhow::{anyhow, Result};
use log::info;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use strum_macros::{Display, EnumString};

use crate::token_registry::Token;

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
    info!("fetch_price: {:#?}", address);

    let url = format!("{JUP_API}?ids={}", address);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    info!("response: {:#?}", response);

    response
        .data
        .get(address)
        .ok_or_else(|| anyhow!("Token {} not found", address))
        .and_then(|data| data.price.parse::<f64>().map_err(|e| anyhow!(e)))
}

pub async fn fetch_many_prices(addresses: &[&str]) -> Result<HashMap<String, f64>> {
    // Deduplicate addresses to avoid redundant API calls
    let unique_addresses: Vec<&str> = {
        let mut set = std::collections::HashSet::new();
        addresses
            .iter()
            .filter(|&&addr| set.insert(addr))
            .cloned()
            .collect()
    };

    // Construct the URL with comma-separated addresses
    let params = unique_addresses.join(",");
    let url = format!("{}?ids={}", JUP_API, params);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    let mut result = HashMap::new();
    for address in unique_addresses {
        let token_data = response
            .data
            .get(address)
            .ok_or_else(|| anyhow!("Token {} not found", address))?;
        let price = token_data
            .price
            .parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse price for {}: {}", address, e))?;
        result.insert(address.to_string(), price);
    }

    Ok(result)
}

pub async fn fetch_pair_price(base: &str, vs: &str) -> Result<f64> {
    let url = format!("{JUP_API}?ids={}&vsToken={}", base, vs);
    let response = reqwest::get(&url).await?.json::<PriceResponse>().await?;

    response
        .data
        .get(base)
        .ok_or_else(|| anyhow!("Base token {} not found", base))
        .and_then(|data| data.price.parse::<f64>().map_err(|e| anyhow!(e)))
}

pub fn format_price_result(result: Result<f64>) -> Option<String> {
    result
        .ok()
        .map(format_price)
        .or_else(|| Some("…".to_owned()))
}

pub fn format_price(price: f64) -> String {
    let price_str = price.to_string();
    format!("${}", &price_str[..7.min(price_str.len())])
}

pub async fn fetch_price_and_format(tokens: Vec<Token>) -> Option<String> {
    let is_pair = tokens.len() == 2;
    if !is_pair {
        format_price_result(fetch_price(&tokens[0].address).await)
    } else {
        format_price_result(fetch_pair_price(&tokens[0].address, &tokens[1].address).await)
    }
}

pub async fn fetch_many_price_and_format(tokens: Vec<Token>) -> Option<Vec<String>> {
    let addresses: Vec<&str> = tokens.iter().map(|token| token.address.as_str()).collect();
    let prices_result = fetch_many_prices(&addresses).await;
    prices_result
        .ok()
        .map(|prices| prices.into_values().map(format_price).collect::<Vec<_>>())
}
