use anyhow::Result;
use chrono::Utc;
use log::{info, warn};
use std::collections::HashMap;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

use crate::feeder::{PerpValueInfo, PriceInfo, TokenOrPairAddress, TokenOrPairPriceInfo};
use crate::formatter::format_price;
use crate::jup::perps::PerpsFetcher;
use crate::jup::prices::{PriceFetcher, TokenSymbol};
use crate::token_registry::TokenRegistry;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

pub async fn run_loop(
    price_sender: watch::Sender<HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>>,
    token_registry: &TokenRegistry,
    maybe_wallet_address: Option<&str>,
) -> Result<()> {
    let mut retry_count = 0;
    let price_fetcher = PriceFetcher::new();

    // Single tokens
    let singles_tokens = token_registry.tokens.clone();

    // Pairs
    let pairs = token_registry.pairs.clone();

    // Preps
    let perps_fetcher = PerpsFetcher::default();

    // POC SOL Perps
    let sol_token = token_registry
        .get_by_symbol(&TokenSymbol::SOL)
        .expect("Invalid symbol");

    loop {
        // Token Prices
        match price_fetcher
            .fetch_many_price_and_format(singles_tokens.clone(), pairs.clone())
            .await
        {
            Some(prices_map) => {
                retry_count = 0;
                // info!("{:#?}", prices_map);
                price_sender.send(prices_map)?;
            }
            None => {
                retry_count += 1;
                warn!("Price fetch failed (attempt {})", retry_count);

                // Exponential backoff up to 5 minutes
                let backoff = Duration::from_secs(30).mul_f32(2f32.powi(retry_count - 1));
                sleep(backoff.min(Duration::from_secs(300))).await;
                continue;
            }
        }

        // JUP Perps
        let wallet_address = match maybe_wallet_address {
            Some(wallet_address) => wallet_address,
            None => return Ok(()),
        };

        println!("Fetching positions for wallet: {:?}", wallet_address);
        match perps_fetcher
            .fetch_positions_pnl_and_format(wallet_address)
            .await
        {
            Ok(positions_result) => {
                retry_count = 0;
                let mut prices_map: HashMap<TokenOrPairAddress, TokenOrPairPriceInfo> =
                    HashMap::new();
                let perps_key: TokenOrPairAddress = format!("{}_PERPS", sol_token.address.clone());
                let id = format!("{}_PERPS", sol_token.symbol.clone());
                let price = positions_result.total_pnl_usd;
                let value_in_usd_info = TokenOrPairPriceInfo::Perp(PerpValueInfo {
                    id,
                    token: sol_token.clone(),
                    pnl_after_fees_usd: PriceInfo {
                        price: Some(price),
                        formatted_price: format_price(price),
                        updated_at: Utc::now().timestamp_millis() as u64,
                    },
                });
                prices_map.insert(perps_key, value_in_usd_info);
                info!("{:#?}", prices_map);
                price_sender.send(prices_map)?;
            }
            Err(_) => {
                retry_count += 1;
                warn!("Preps fetch failed (attempt {})", retry_count);

                // Exponential backoff up to 5 minutes
                let backoff = Duration::from_secs(30).mul_f32(2f32.powi(retry_count - 1));
                sleep(backoff.min(Duration::from_secs(300))).await;
                continue;
            }
        }

        // Wait for next poll
        sleep(POLL_INTERVAL).await;
    }
}
