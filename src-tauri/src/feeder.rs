use crate::ray::{fetch_pool_info_by_id, PoolId};

#[derive(Default, Debug, Copy, Clone)]
pub struct PriceInfo {
    pub price: Option<f64>,
    pub retry_count: i32,
}

pub async fn get_price_by_token_id(pool_id: PoolId) -> anyhow::Result<f64> {
    let pool_info = fetch_pool_info_by_id(pool_id).await?;

    // Get price from pool that match id
    let price = pool_info.price;

    Ok(price)
}
