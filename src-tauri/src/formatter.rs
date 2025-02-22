use crate::feeder::{PairPriceInfo, PerpValueInfo, TokenOrPairPriceInfo, TokenPriceInfo};

pub fn update_price_display(price_info: &TokenOrPairPriceInfo) -> (String, String) {
    match price_info {
        TokenOrPairPriceInfo::Pair(PairPriceInfo {
            token_a,
            token_b,
            price_info,
        }) => {
            let label = format!("{}/{}", token_a.symbol, token_b.symbol);
            let formatted_price = price_info
                .price
                .map(format_price)
                .unwrap_or("…".to_string());
            (label, formatted_price)
        }
        TokenOrPairPriceInfo::Token(TokenPriceInfo { token, price_info }) => {
            let label = token.symbol.to_string();
            let formatted_price = price_info
                .price
                .map(format_price_with_dollar)
                .unwrap_or("…".to_string());
            (label, formatted_price)
        }
        TokenOrPairPriceInfo::Perp(PerpValueInfo {
            id: _,
            token,
            pnl_after_fees_usd,
        }) => {
            let label = format!("{}🄿", token.symbol);
            let formatted_price = pnl_after_fees_usd
                .price
                .map(format_price_with_signed_dollar)
                .unwrap_or("…".to_string());
            (label, formatted_price)
        }
    }
}

/// Formats a price result into a user-friendly string.
pub fn format_price_result(result: anyhow::Result<f64>) -> Option<String> {
    result
        .ok()
        .map(format_price)
        .or_else(|| Some("…".to_owned()))
}

/// Formats a price value into a user-friendly string.
pub fn format_price(price: f64) -> String {
    let price_string = price.to_string();
    price_string[..7.min(price_string.len())].to_string()
}

pub fn format_price_with_dollar(price: f64) -> String {
    let price_string = format_price(price);
    if price_string.starts_with("-") {
        let abs_price_string = format_price(price.abs());
        format!("${}", abs_price_string)
    } else {
        format!("${}", price_string)
    }
}

pub fn format_price_with_signed_dollar(price: f64) -> String {
    let price_string = format_price(price);
    if price_string.starts_with("-") {
        let abs_price_string = format_price(price.abs());
        format!("-${}", abs_price_string)
    } else {
        format!("+${}", price_string)
    }
}
