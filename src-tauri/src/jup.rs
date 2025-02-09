use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::AsRefStr;
use strum_macros::{Display, EnumString};

use crate::{
    fetcher::{Fetcher, RetrySettings},
    formatter::{format_price, format_price_result},
    token_registry::Token,
};

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

/// A dedicated struct for fetching prices.
pub struct PriceFetcher {
    fetcher: Fetcher,
}

impl Default for PriceFetcher {
    fn default() -> Self {
        Self {
            fetcher: Fetcher::new(),
        }
    }
}

impl PriceFetcher {
    /// Creates a new `PriceFetcher` with default settings.
    pub fn new() -> Self {
        Self {
            fetcher: Fetcher::new(),
        }
    }

    /// Creates a new `PriceFetcher` with custom settings.
    pub fn with_settings(settings: RetrySettings) -> Self {
        Self {
            fetcher: Fetcher::with_settings(settings),
        }
    }

    /// Fetches the price of a single token.
    pub async fn fetch_price(&self, address: &str) -> Result<f64> {
        let url = format!("{JUP_API}?ids={}", address);
        self.fetch_price_internal(&url).await.and_then(|mut map| {
            map.remove(address)
                .ok_or_else(|| anyhow!("Token {} not found", address))
        })
    }

    /// Fetches the price of a token pair.
    pub async fn fetch_pair_price(&self, base: &str, vs: &str) -> Result<f64> {
        let url = format!("{JUP_API}?ids={}&vsToken={}", base, vs);
        self.fetch_price_internal(&url).await.and_then(|mut map| {
            map.remove(base)
                .ok_or_else(|| anyhow!("Base token {} not found", base))
        })
    }

    /// Fetches prices for multiple tokens.
    pub async fn fetch_many_prices(&self, addresses: &[&str]) -> Result<HashMap<String, f64>> {
        let params = addresses.join(",");
        let url = format!("{JUP_API}?ids={}", params);
        self.fetch_price_internal(&url).await
    }

    /// Shared logic for fetching prices.
    async fn fetch_price_internal(&self, url: &str) -> Result<HashMap<String, f64>> {
        self.fetcher
            .fetch_with_retry(url, |response: PriceResponse| {
                response
                    .data
                    .iter()
                    .map(|(address, data)| {
                        data.price
                            .parse::<f64>()
                            .map(|price| (address.clone(), price))
                            .map_err(|e| anyhow!("Failed to parse price for {}: {}", address, e))
                    })
                    .collect()
            })
            .await
    }

    /// Fetches and formats the price for a single token or a token pair.
    pub async fn fetch_price_and_format(&self, tokens: Vec<Token>) -> Option<String> {
        let is_pair = tokens.len() == 2;
        if !is_pair {
            format_price_result(self.fetch_price(&tokens[0].address).await)
        } else {
            format_price_result(
                self.fetch_pair_price(&tokens[0].address, &tokens[1].address)
                    .await,
            )
        }
    }

    /// Fetches and formats prices for multiple tokens.
    pub async fn fetch_many_price_and_format(&self, tokens: Vec<Token>) -> Option<Vec<String>> {
        let addresses: Vec<&str> = tokens.iter().map(|token| token.address.as_str()).collect();
        let prices_result = self.fetch_many_prices(&addresses).await;
        prices_result
            .ok()
            .map(|prices| prices.into_values().map(format_price).collect::<Vec<_>>())
    }
}
