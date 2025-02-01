use anyhow::Result;

use tokio::sync::watch;
use tokio::time::{sleep, Duration};

use crate::jup::{fetch_price, TokenSymbol};
use crate::token_registry::TokenRegistry;

const POLL_INTERVAL: Duration = Duration::from_secs(30);

pub async fn run_loop(
    price_sender: watch::Sender<Option<f64>>,
    token_receiver: watch::Receiver<TokenSymbol>,
) -> Result<()> {
    let mut current_token = *token_receiver.borrow();
    let mut retry_count = 0;

    let token_registry = TokenRegistry::new();

    loop {
        // Check for token changes
        if token_receiver.has_changed()? {
            current_token = *token_receiver.borrow();
            retry_count = 0; // Reset retry counter on token change
        }

        // Get token address from symbol map
        let token_address_string = &token_registry
            .get_by_symbol(&current_token)
            .expect("Not found data for symbol:{current_token}")
            .address;

        // Fetch price with retry logic
        match fetch_price(token_address_string).await {
            Ok(price) => {
                price_sender.send(Some(price))?;
                retry_count = 0; // Reset retry counter on success
            }
            Err(e) => {
                retry_count += 1;
                println!("Price fetch failed (attempt {}): {}", retry_count, e);

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
