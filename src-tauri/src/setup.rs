use crate::{
    formatter::get_label_and_ui_price,
    runner::run_loop,
    settings::{initialize_settings, load_settings},
    token_registry::{get_pair_or_token_address_from_tokens, TokenRegistry},
    tray::setup_tray,
    AppState, SelectedTokenOrPair, TokenOrPairAddress, TokenOrPairPriceInfo, TokenSymbol,
};

use std::collections::HashMap;
use tauri::{App, Manager};
use tauri_plugin_fs::FsExt;

use tokio::sync::watch;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let scope = app.try_fs_scope().expect("Invalid scope.");
    scope.allow_directory("./", false)?;
    dbg!(scope.is_allowed("./"));

    let app_handle = app.app_handle();
    initialize_settings(app_handle.clone());

    let token_registry = TokenRegistry::new();
    let app_state = app.state::<AppState>();
    *app_state.token_registry.lock().unwrap() = token_registry.clone();

    // Load settings once and reuse
    let settings_result = load_settings(app_handle.clone());
    let default_token = token_registry
        .get_by_symbol(&TokenSymbol::SOL)
        .cloned()
        .unwrap_or_default();

    let (settings, tokens) = match &settings_result {
        Ok(settings) => {
            let tokens = token_registry
                .get_tokens_from_pair_address(
                    &settings
                        .recent_token_id
                        .clone()
                        .unwrap_or(TokenSymbol::SOL.to_string()),
                )
                .unwrap_or(vec![default_token.clone()]);

            (Some(settings), tokens)
        }
        Err(_) => (None, vec![default_token.clone()]),
    };

    let recent_token_id = if let Some(settings) = settings {
        settings
            .recent_token_id
            .clone()
            .unwrap_or(default_token.address.clone())
    } else {
        default_token.address.clone()
    };

    // Setup tray
    let (tray_id, tray_menu) = setup_tray(app.handle(), &recent_token_id).expect("Expect tray_id");
    *app_state.tray_id.lock().unwrap() = Some(tray_id.clone());
    *app_state.tray_menu.lock().unwrap() = Some(tray_menu.clone());

    // Define sender
    let (token_sender, mut token_receiver) = watch::channel(tokens.clone());
    *app_state.token_sender.lock().unwrap() = Some(token_sender);

    let (price_sender, mut price_receiver) =
        watch::channel::<HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>>(Default::default());
    *app_state.price_sender.lock().unwrap() = Some(price_sender.clone());

    let cloned_app_handle = app.handle().clone();
    *app_state.selected_token_or_pair_address.lock().unwrap() = SelectedTokenOrPair {
        address: tokens[0].address.clone(),
    };

    let tray_menu = app_state
        .tray_menu
        .lock()
        .unwrap()
        .clone()
        .expect("Tray not initialized");

    let tray_icon = cloned_app_handle
        .tray_by_id(&tray_id)
        .expect("Tray missing");

    tauri::async_runtime::spawn(async move {
        let _ = token_receiver.changed().await;
        let selected_tokens = token_receiver.borrow_and_update().clone();

        let selected_token_or_pair_address_string =
            get_pair_or_token_address_from_tokens(&selected_tokens).expect("Invalid token address");

        let app_state = cloned_app_handle.state::<AppState>();
        let mut selected_token_or_pair_address = app_state
            .selected_token_or_pair_address
            .lock()
            .unwrap()
            .clone();

        selected_token_or_pair_address.address = selected_token_or_pair_address_string.clone();
    });

    let cloned_app_handle = app.handle().clone();

    // Price effect
    tauri::async_runtime::spawn(async move {
        let tray_menu_clone = tray_menu.clone();

        loop {
            let _ = price_receiver.changed().await;
            let price_info_map = price_receiver.borrow().clone();

            // Update tray
            let app_state = cloned_app_handle.state::<AppState>();
            let selected_token_or_pair_address = app_state
                .selected_token_or_pair_address
                .lock()
                .unwrap()
                .clone();

            let maybe_price_info = price_info_map.get(&selected_token_or_pair_address.address);

            if let Some(price_info) = maybe_price_info {
                let (_label, ui_price) = get_label_and_ui_price(price_info);
                let _ = tray_icon.set_title(Some(ui_price));
            }

            // Update menu
            let items = tray_menu_clone.items().unwrap();
            price_info_map
                .iter()
                .for_each(|(token_address, v)| match v {
                    TokenOrPairPriceInfo::Perp(perp_value_info) => {
                        if let Some(item) = items
                            .iter()
                            .find(|menu_item| menu_item.id().0.as_str() == perp_value_info.id)
                        {
                            if let Some(item) = item.as_icon_menuitem() {
                                let (_label, ui_price) = get_label_and_ui_price(v);
                                let _ = item.set_text(ui_price);
                            }
                        }
                    }
                    _ => {
                        if let Some(item) = items
                            .iter()
                            .find(|menu_item| menu_item.id().0.as_str() == token_address)
                        {
                            if let Some(item) = item.as_icon_menuitem() {
                                let (_label, ui_price) = get_label_and_ui_price(v);
                                let _ = item.set_text(ui_price);
                            }
                        }
                    }
                });
        }
    });

    let maybe_wallet_address = app_state.current_public_key.lock().unwrap().clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_loop(
            price_sender.clone(),
            &token_registry,
            maybe_wallet_address.as_deref(),
        )
        .await
        {
            eprintln!("Price fetch error: {}", e);
        }
    });

    Ok(())
}
