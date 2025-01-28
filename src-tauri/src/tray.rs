use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIconBuilder, TrayIconId},
    App,
};

use crate::assets::fetch_and_set_icon;

pub fn setup_tray(app: &mut App) -> anyhow::Result<TrayIconId> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let setting_i = MenuItem::with_id(app, "setting", "Setting", true, None::<&str>)?;
    let about_i = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
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
            &about_i,
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
