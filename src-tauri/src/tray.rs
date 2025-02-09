use tauri::{
    menu::{AboutMetadata, IconMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconId},
    AppHandle,
};

use crate::{assets::read_local_image, jup::TokenSymbol, token_registry::TokenRegistry};

fn get_menu_pair_item(
    app_handle: &AppHandle,
    token_registry: &TokenRegistry,
    token_a_symbol: &TokenSymbol,
    token_b_symbol: &TokenSymbol,
) -> anyhow::Result<IconMenuItem<tauri::Wry>> {
    let pair = [
        token_registry
            .get_by_symbol(token_a_symbol)
            .expect("Not exist")
            .clone(),
        token_registry
            .get_by_symbol(token_b_symbol)
            .expect("Not exist")
            .clone(),
    ];
    let pair_symbol = format!("{}_{}", pair[0].symbol, pair[1].symbol);
    let pair_address = format!("{}_{}", pair[0].address, pair[1].address);
    let pair_label = format!("{}/{}", pair[0].symbol, pair[1].symbol);
    let icon_path = format!("./tokens/{}.png", pair_symbol);
    let pair_icon = read_local_image(&icon_path).ok();
    let icon_menu_item = IconMenuItem::with_id(
        app_handle,
        pair_address,
        pair_label,
        true,
        pair_icon,
        None::<&str>,
    )?;

    Ok(icon_menu_item)
}

pub fn setup_tray(app_handle: &tauri::AppHandle) -> anyhow::Result<(TrayIconId, Menu<tauri::Wry>)> {
    // Portfolio
    let icon = read_local_image("../assets/jup-portfolio.png").ok();
    let portfolio_i = IconMenuItem::with_id(
        app_handle,
        "portfolio",
        "JUP Portfolio",
        true,
        icon,
        None::<&str>,
    )?;

    // Quit
    let quit_i = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;

    // Settings
    let settings_i = MenuItem::with_id(app_handle, "settings", "Setting", true, None::<&str>)?;

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

    // Default tokens
    let token_registry = TokenRegistry::new();
    let tokens = token_registry.tokens.clone();

    // Menu
    let token_menu_items: Vec<_> = tokens
        .iter()
        .map(|token| {
            let icon_path = format!("./tokens/{}.png", token.symbol);
            let icon = read_local_image(&icon_path).ok();

            IconMenuItem::with_id(
                app_handle,
                token.address.clone(),
                token.symbol,
                true,
                icon,
                None::<&str>,
            )
        })
        .collect::<Result<_, _>>()?;

    let menu = Menu::with_items(
        app_handle,
        &[
            &PredefinedMenuItem::separator(app_handle)?,
            &portfolio_i,
            &PredefinedMenuItem::separator(app_handle)?,
            &settings_i,
            &PredefinedMenuItem::about(app_handle, None, Some(about_metadata))?,
            &PredefinedMenuItem::separator(app_handle)?,
            &quit_i,
        ],
    )?;

    // Convert each IconMenuItem to a &dyn IsMenuItem
    let token_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = token_menu_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<tauri::Wry>)
        .collect();
    let _ = menu.insert_items(&token_refs, 0);

    // Pair JLP_SOL
    #[allow(non_snake_case)]
    let JLP_SOL = get_menu_pair_item(
        app_handle,
        &token_registry,
        &TokenSymbol::JLP,
        &TokenSymbol::SOL,
    )?;
    #[allow(non_snake_case)]
    let laineSOL_SOL = get_menu_pair_item(
        app_handle,
        &token_registry,
        &TokenSymbol::laineSOL,
        &TokenSymbol::SOL,
    )?;
    let _ = menu.insert_items(
        &[
            &JLP_SOL as &dyn IsMenuItem<tauri::Wry>,
            &laineSOL_SOL as &dyn IsMenuItem<tauri::Wry>,
            &PredefinedMenuItem::separator(app_handle)?,
        ],
        0,
    );

    let tray_icon = TrayIconBuilder::new()
        .icon(app_handle.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app_handle)?;

    // Clone values needed in async task before moving them
    let tray_id = tray_icon.id().clone();

    // Default Icon
    tauri::async_runtime::spawn(async move {
        let icon_path = format!("./tokens/{}.png", TokenSymbol::SOL);
        let icon = read_local_image(&icon_path).expect("Image not found");
        tray_icon.set_icon(Some(icon)).expect("Expect tray_icon");
    });

    Ok((tray_id, menu))
}
