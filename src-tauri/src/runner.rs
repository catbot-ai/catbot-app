use anyhow::Result;
use log::warn;
use std::collections::HashMap;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

use crate::feeder::{TokenOrPairAddress, TokenOrPairPriceInfo};
use crate::jup::prices::PriceFetcher;
use crate::token_registry::TokenRegistry;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

pub async fn run_loop(
    price_sender: watch::Sender<HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>>,
    // token_receiver: watch::Receiver<Vec<Token>>,
    token_registry: &TokenRegistry,
) -> Result<()> {
    let mut retry_count = 0;
    let price_fetcher = PriceFetcher::new();

    // Single tokens
    let singles_tokens = token_registry.tokens.clone();

    // Pairs
    let pairs = token_registry.pairs.clone();

    loop {
        // Check for token changes
        // if token_receiver.has_changed()? {
        //     // Keep receiver a life
        //     let _selected_tokens = token_receiver.borrow().clone();
        //     retry_count = 0; // Reset retry counter on token change
        // }

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

        // Wait for next poll
        sleep(POLL_INTERVAL).await;
    }
}
