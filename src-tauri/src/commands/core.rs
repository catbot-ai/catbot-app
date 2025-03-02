use std::collections::HashMap;

use crate::assets::read_local_image;
use crate::{AppState, SelectedTokenOrPair};
use common::RefinedPredictionOutput;
use jup_sdk::feeder::{TokenOrPairAddress, TokenOrPairPriceInfo};
use jup_sdk::prices::PriceFetcher;
use jup_sdk::token_registry::{get_pair_or_token_address_from_tokens, Token};
use log::{info, warn};
use std::env;
use strum_macros::{Display, EnumString};
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::watch;

use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;

#[derive(Debug, Eq, PartialEq, EnumString, Display)]
pub enum UserCommand {
    #[strum(serialize = "suggest")]
    Suggest,
}

fn build_client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(Client::new())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

async fn fetch_suggestion(
    symbol: &str,
    wallet_address: &str,
) -> anyhow::Result<RefinedPredictionOutput> {
    dotenvy::from_filename(".env").ok();
    let suggest_api_url = env::var("SUGGEST_API_URL").expect("Missing .env SUGGEST_API_URL");

    let binance_pair_symbol = format!("{symbol}USDT");
    let client = build_client();
    let url = format!("{suggest_api_url}/{binance_pair_symbol}?wallet_address={wallet_address}");
    let response = client.get(url).send().await?;
    let suggestion = serde_json::from_value::<RefinedPredictionOutput>(response.json().await?)?;

    Ok(suggestion)
}

pub async fn get_suggestion(
    app_handle: tauri::AppHandle,
    wallet_address: &str,
    token_or_pair_symbol: &str,
) -> anyhow::Result<()> {
    info!("⬆️ fetch_suggestion:{:#?}", token_or_pair_symbol);
    let suggestion = fetch_suggestion(token_or_pair_symbol, wallet_address).await?;
    info!("⬇️ suggestion:{:#?}", suggestion);

    // Notify
    app_handle
        .notification()
        .builder()
        .title(suggestion.summary.title)
        .body(suggestion.summary.suggestion)
        .show()
        .unwrap();

    Ok(())
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn update_token_and_price(
    app_handle: tauri::AppHandle,
    selected_tokens: Vec<Token>,
    price_sender: &watch::Sender<HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>>,
) -> anyhow::Result<()> {
    // Update tray icon and title
    let is_pair = selected_tokens.len() == 2;
    let icon_path = if !is_pair {
        format!("./tokens/{}.png", selected_tokens[0].symbol)
    } else {
        let pair_symbol = format!(
            "{}_{}",
            selected_tokens[0].symbol, selected_tokens[1].symbol
        );
        format!("./tokens/{}.png", pair_symbol)
    };

    let selected_token_or_pair_address = get_pair_or_token_address_from_tokens(&selected_tokens);
    let state = app_handle.state::<AppState>();
    *state.selected_token_or_pair_address.lock().unwrap() = SelectedTokenOrPair {
        address: selected_token_or_pair_address.clone(),
    };

    let icon = read_local_image(&icon_path)?;

    let tray_id = {
        state
            .tray_id
            .lock()
            .unwrap()
            .clone()
            .expect("Missing tray_id")
    };

    let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
    tray_icon.set_icon(Some(icon))?;

    // Loading
    tray_icon.set_title(Some("…"))?;

    // Instant fetch price
    let price_sender_clone = price_sender.clone();
    let is_pair: bool = selected_tokens.len() == 2;
    let single_tokens = if is_pair {
        vec![]
    } else {
        selected_tokens.clone()
    };
    let pair_tokens = if is_pair {
        vec![[selected_tokens[0].clone(), selected_tokens[1].clone()]]
    } else {
        vec![]
    };

    tauri::async_runtime::spawn(async move {
        // TODO: perps is more complex, should we wait?
        let price_fetcher = PriceFetcher::new();
        match price_fetcher
            .fetch_many_price_and_format(single_tokens, pair_tokens)
            .await
        {
            Some(prices_map) => {
                let _ = price_sender_clone.send(prices_map);
            }
            None => {
                warn!("Price fetch failed.");
            }
        };
    });

    // let tray_menu = state
    //     .tray_menu
    //     .lock()
    //     .unwrap()
    //     .clone()
    //     .ok_or("Tray not initialized".to_string())?;

    // let items = tray_menu.items().unwrap();
    // let foo = items.first().unwrap();
    // println!("foo:{:?}", foo.id());

    Ok(())
}
