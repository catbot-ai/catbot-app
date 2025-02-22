use std::collections::HashMap;

use crate::assets::read_local_image;
use crate::feeder::{TokenOrPairAddress, TokenOrPairPriceInfo};
use crate::jup::prices::PriceFetcher;
use crate::token_registry::{get_pair_ot_token_address_from_tokens, Token};
use crate::{AppState, SelectedTokenOrPair};
use log::warn;
use tauri::Manager;
use tokio::sync::watch;

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
    // Update selected token
    let state = app_handle.state::<AppState>();
    *state.selected_tokens.lock().unwrap() = selected_tokens.clone();

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

    let selected_token_or_pair_address = get_pair_ot_token_address_from_tokens(&selected_tokens)?;
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
        match PriceFetcher::new()
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
