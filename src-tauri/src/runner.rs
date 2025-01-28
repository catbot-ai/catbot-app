use std::time::Duration;
use tokio::sync::watch;

use crate::{
    jup::{fetch_price_from_jup_in_usdc, TokenId},
    ray,
};
use wasm_timer::Delay;

pub async fn run_loop(price_sender: watch::Sender<Option<f64>>) -> anyhow::Result<()> {
    let pool_id = ray::PoolId::SOL_JLP;

    println!("🔥 Starting price fetch loop for token_id: {}", pool_id);
    loop {
        let price = fetch_price_from_jup_in_usdc(TokenId::SOL).await?;
        price_sender.send(Some(price))?; // Send new price
        Delay::new(Duration::from_secs(5)).await?;
    }
}
