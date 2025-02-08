use anyhow::{anyhow, Result};
use log::warn;
use reqwest;
use serde::de::DeserializeOwned;
use tokio::time::{timeout, Duration};

/// Helper function to calculate exponential backoff delay.
fn exponential_backoff(retries: u32) -> Duration {
    Duration::from_secs(2u64.pow(retries))
}

pub async fn fetch_with_retry<F, R, T>(url: &str, processor: F) -> Result<T>
where
    R: DeserializeOwned,
    F: Fn(R) -> Result<T>,
{
    const MAX_RETRIES: usize = 3;
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

    let mut retries = 0;

    loop {
        match timeout(REQUEST_TIMEOUT, reqwest::get(url)).await {
            Ok(response) => {
                let response = response?;
                let api_response = response.json::<R>().await?;
                return processor(api_response);
            }
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(anyhow!("Request failed after {} retries: {}", retries, e));
                }
                warn!("Request failed (attempt {}): {}", retries, e);
                tokio::time::sleep(exponential_backoff(retries as u32)).await;
            }
        }
    }
}
