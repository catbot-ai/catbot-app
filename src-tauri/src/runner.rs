use std::time::Duration;
use tokio::sync::watch;

use crate::{
    jup::{fetch_price_from_jup_in_usdc, TokenId},
    ray,
};
use wasm_timer::Delay;

// TODO: When error it should send status to caller as ERROR_RESPONSE, NO_RESPONSE
pub async fn run_loop(price_sender: watch::Sender<Option<f64>>) -> anyhow::Result<()> {
    let pool_id = ray::PoolId::SOL_JLP;

    println!("🔥 Starting price fetch loop for token_id: {}", pool_id);
    loop {
        let price = fetch_price_from_jup_in_usdc(TokenId::SOL).await;
        match price {
            Ok(value) => {
                price_sender.send(Some(value))?; // Send new price
            }
            Err(error) => println!("{error:#?}"),
        }

        Delay::new(Duration::from_secs(5)).await?;
    }
}
