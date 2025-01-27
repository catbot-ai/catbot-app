use std::time::Duration;
use tokio::sync::watch;

use crate::{feeder, ray};

use feeder::get_price_by_token_id;
use wasm_timer::Delay;

pub async fn run_loop(price_sender: watch::Sender<Option<f64>>) -> anyhow::Result<()> {
    let pool_id = ray::PoolId::SOL_JLP;

    println!("🔥 Starting price fetch loop for token_id: {}", pool_id);
    loop {
        let price = get_price_by_token_id(pool_id.clone()).await?;
        price_sender.send(Some(price))?; // Send new price
        Delay::new(Duration::from_secs(5)).await?;
    }
}
