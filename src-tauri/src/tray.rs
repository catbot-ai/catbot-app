use tauri::{
    menu::{AboutMetadata, IconMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconId},
};

use crate::{assets::read_local_image, jup::TokenSymbol, token_registry::TokenRegistry};

pub fn setup_tray(app_handle: &tauri::AppHandle) -> anyhow::Result<TrayIconId> {
    // Portfolio
    let portfolio_i =
        MenuItem::with_id(app_handle, "portfolio", "JUP Portfolio", true, None::<&str>)?;

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

    // Default tokens
    let tokens = TokenRegistry::get_tokens();

    // Menu
    let token_menu_items: Vec<_> = tokens
        .iter()
        .map(|token| {
            let icon_path = format!("./tokens/{}.png", token.symbol);
            let icon = read_local_image(&icon_path).expect("Image not found");

            IconMenuItem::with_id(
                app_handle,
                token.address.clone(),
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
            &PredefinedMenuItem::separator(app_handle)?,
            &portfolio_i,
            &PredefinedMenuItem::separator(app_handle)?,
            &setting_i,
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

    Ok(tray_id)
}
