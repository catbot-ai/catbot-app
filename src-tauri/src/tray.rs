use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIconBuilder, TrayIconId},
    App,
};

use crate::assets::fetch_and_set_icon;

pub fn setup_tray(app: &mut App) -> anyhow::Result<TrayIconId> {
    // Quit
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Setting
    let setting_i = MenuItem::with_id(app, "setting", "Setting", true, None::<&str>)?;

    // About
    let pkg_info = app.package_info();
    let config = app.config();
    let about_metadata = AboutMetadata {
        name: Some(pkg_info.name.clone()),
        version: Some(pkg_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        ..Default::default()
    };

    // Menu
    let menu = Menu::with_items(
        app,
        &[
            &Submenu::with_items(
                app,
                "Foo",
                true,
                &[
                    &PredefinedMenuItem::close_window(app, None)?,
                    &MenuItem::new(app, "Hello", true, None::<&str>)?,
                ],
            )?,
            &PredefinedMenuItem::separator(app)?,
            &setting_i,
            &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
            &PredefinedMenuItem::separator(app)?,
            &quit_i,
        ],
    )?;

    let tray_icon = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app)?;

    // Clone values needed in async task before moving them
    let tray_id = tray_icon.id().clone();

    // Icon
    tauri::async_runtime::spawn(async move {
        let _ = fetch_and_set_icon(
            "https://img-v1.raydium.io/icon/So11111111111111111111111111111111111111112.png",
            &tray_icon,
        )
        .await;
    });

    Ok(tray_id)
}
