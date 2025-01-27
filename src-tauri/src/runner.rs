use std::{sync::Arc, time::Duration};
use tauri::async_runtime::Mutex;

use crate::{feeder, ray};

use feeder::get_price_by_token_id;
use wasm_timer::Delay;

pub async fn run_loop(price_state: Arc<Mutex<Option<f64>>>) -> anyhow::Result<()> {
    let pool_id = ray::PoolId::SOL_JLP;

    println!("🔥 Starting price fetch loop for token_id: {}", pool_id);
    loop {
        let price = get_price_by_token_id(pool_id.clone()).await?;
        println!("Price of token_id: {} is {}", pool_id, price);

        // Update the shared state
        let mut price_guard = price_state.lock().await;
        *price_guard = Some(price);

        // Wait for the next iteration
        Delay::new(Duration::from_secs(5)).await.ok();
    }
}
