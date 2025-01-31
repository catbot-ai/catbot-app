use std::time::Duration;
use tokio::sync::watch;

use crate::jup::{fetch_price_from_jup_in_usdc, TokenId, TokenSymbol};
use wasm_timer::Delay;

// TODO: When error it should send status to caller as ERROR_RESPONSE, NO_RESPONSE

pub async fn run_loop(
    price_sender: watch::Sender<Option<f64>>,
    mut token_receiver: watch::Receiver<TokenSymbol>,
) -> anyhow::Result<()> {
    let mut current_token = *token_receiver.borrow(); // Initial token
    println!(
        "🔥 Starting price fetch loop for token: {:?}",
        current_token
    );

    loop {
        let token_address = match current_token {
            TokenSymbol::SOL => TokenId::SOL,
            TokenSymbol::JLP => TokenId::JLP,
            TokenSymbol::USDC => TokenId::USDC,
        };
        // Fetch the price for the current token
        let price = fetch_price_from_jup_in_usdc(token_address).await;
        match price {
            Ok(value) => {
                price_sender.send(Some(value))?; // Send new price
            }
            Err(error) => println!("{error:#?}"),
        }

        // Check if the token has changed
        if token_receiver.has_changed()? {
            current_token = *token_receiver.borrow_and_update(); // Update to the new token
            println!("Switching to new token: {:?}", current_token);
        }

        Delay::new(Duration::from_secs(5)).await?;
    }
}
