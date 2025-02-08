use anyhow::{anyhow, Result};
use log::{info, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use strum_macros::{Display, EnumString};
use tokio::time::{timeout, Duration};

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
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10); // Timeout for API requests
const MAX_RETRIES: usize = 3; // Maximum number of retries for failed requests

/// Helper function to calculate exponential backoff delay.
fn exponential_backoff(retries: u32) -> Duration {
    Duration::from_secs(2u64.pow(retries))
}

/// Fetches the price of a single token with retry logic and timeout.
pub async fn fetch_price(address: &str) -> Result<f64> {
    info!("fetch_price: {:#?}", address);

    let url = format!("{JUP_API}?ids={}", address);
    let mut retries = 0;

    loop {
        match timeout(REQUEST_TIMEOUT, reqwest::get(&url)).await {
            Ok(response) => {
                let response = response?;
                let price_response = response.json::<PriceResponse>().await?;

                if let Some(token_data) = price_response.data.get(address) {
                    return token_data
                        .price
                        .parse::<f64>()
                        .map_err(|e| anyhow!("Failed to parse price: {}", e));
                } else {
                    return Err(anyhow!("Token {} not found", address));
                }
            }
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(anyhow!(
                        "Request timed out after {} retries: {}",
                        retries,
                        e
                    ));
                }
                warn!("Request failed (attempt {}): {}", retries, e);

                // Use exponential backoff
                let delay = exponential_backoff(retries as u32);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Fetches prices for multiple tokens with retry logic and timeout.
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
    let mut retries = 0;

    loop {
        match timeout(REQUEST_TIMEOUT, reqwest::get(&url)).await {
            Ok(response) => {
                let response = response?;
                let price_response = response.json::<PriceResponse>().await?;

                let mut result = HashMap::new();
                for address in unique_addresses.iter() {
                    if let Some(token_data) = price_response.data.get(*address) {
                        let price = token_data
                            .price
                            .parse::<f64>()
                            .map_err(|e| anyhow!("Failed to parse price for {}: {}", address, e))?;
                        result.insert((*address).to_string(), price);
                    } else {
                        return Err(anyhow!("Token {} not found", address));
                    }
                }
                return Ok(result);
            }
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(anyhow!(
                        "Request timed out after {} retries: {}",
                        retries,
                        e
                    ));
                }
                warn!("Request failed (attempt {}): {}", retries, e);

                // Use exponential backoff
                let delay = exponential_backoff(retries as u32);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Fetches the price of a token pair with retry logic and timeout.
pub async fn fetch_pair_price(base: &str, vs: &str) -> Result<f64> {
    let url = format!("{JUP_API}?ids={}&vsToken={}", base, vs);
    let mut retries = 0;

    loop {
        match timeout(REQUEST_TIMEOUT, reqwest::get(&url)).await {
            Ok(response) => {
                let response = response?;
                let price_response = response.json::<PriceResponse>().await?;

                if let Some(token_data) = price_response.data.get(base) {
                    return token_data
                        .price
                        .parse::<f64>()
                        .map_err(|e| anyhow!("Failed to parse price: {}", e));
                } else {
                    return Err(anyhow!("Base token {} not found", base));
                }
            }
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(anyhow!(
                        "Request timed out after {} retries: {}",
                        retries,
                        e
                    ));
                }
                warn!("Request failed (attempt {}): {}", retries, e);

                // Use exponential backoff
                let delay = exponential_backoff(retries as u32);
                tokio::time::sleep(delay).await;
            }
        }
    }
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
