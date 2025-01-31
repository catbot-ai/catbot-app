use tauri::{
    menu::{AboutMetadata, IconMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconId},
};

use crate::{
    assets::{fetch_and_set_icon, read_local_image},
    jup::{TokenId, TokenSymbol},
};

#[derive(Copy, Clone)]
pub struct TokenInfo {
    id: TokenId,
    symbol: TokenSymbol,
}

pub fn setup_tray(
    app_handle: &tauri::AppHandle,
    token_symbol: TokenSymbol,
) -> anyhow::Result<TrayIconId> {
    // Quit
    let quit_i = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;

    // Setting
    let setting_i = MenuItem::with_id(app_handle, "setting", "Setting", true, None::<&str>)?;

    // About
    let pkg_info = app_handle.package_info();
    let config = app_handle.config();
    let about_metadata = AboutMetadata {
        name: Some(pkg_info.name.clone()),
        version: Some(pkg_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        ..Default::default()
    };

    // TODO: load from json (we can't async load from url at the moment)
    // icons
    let tokens: Vec<TokenInfo> = vec![
        TokenInfo {
            id: TokenId::JLP,
            symbol: TokenSymbol::JLP,
        },
        TokenInfo {
            id: TokenId::SOL,
            symbol: TokenSymbol::SOL,
        },
    ];

    // Menu
    let token_menu_items: Vec<_> = tokens
        .iter()
        .map(|token| {
            let icon_path = format!("./icons/{}.png", token.symbol);
            let icon = read_local_image(&icon_path).expect("Image not found");

            IconMenuItem::with_id(
                app_handle,
                token.id,
                token.symbol,
                true,
                Some(icon),
                None::<&str>,
            )
        })
        .collect::<Result<_, _>>()?;

    let menu = Menu::with_items(
        app_handle,
        &[
            &token_menu_items[0],
            &token_menu_items[1],
            &PredefinedMenuItem::separator(app_handle)?,
            &setting_i,
            &PredefinedMenuItem::about(app_handle, None, Some(about_metadata))?,
            &PredefinedMenuItem::separator(app_handle)?,
            &quit_i,
        ],
    )?;

    let tray_icon = TrayIconBuilder::new()
        .icon(app_handle.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app_handle)?;

    // Clone values needed in async task before moving them
    let tray_id = tray_icon.id().clone();

    // Icon
    tauri::async_runtime::spawn(async move {
        let token_address = match token_symbol {
            TokenSymbol::SOL => TokenId::SOL,
            TokenSymbol::JLP => TokenId::JLP,
            TokenSymbol::USDC => TokenId::USDC,
        };

        let _ = fetch_and_set_icon(
            format!("https://img-v1.raydium.io/icon/{token_address}.png").as_str(),
            &tray_icon,
        )
        .await;
    });

    Ok(tray_id)
}
