pub mod feeder;
pub mod ray;
pub mod runner;

use runner::run_loop;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tokio::sync::watch;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Tray and menu setup
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .build(app)?;

            // Clone values needed in async task before moving them
            let tray_id = tray.id().clone();
            let app_handle = app.handle().clone();

            let (price_sender, price_receiver) = watch::channel(None);

            tauri::async_runtime::spawn(async move {
                let mut price_receiver = price_receiver.clone();

                loop {
                    // Wait for price changes
                    price_receiver.changed().await.unwrap();
                    let price = *price_receiver.borrow_and_update();

                    if let Some(price) = price {
                        let _ = app_handle
                            .tray_by_id(&tray_id)
                            .expect("Tray missing")
                            .set_title(Some(&format!("${:.2}", price)));
                    }
                }
            });

            // Start price fetch loop with sender
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_loop(price_sender).await {
                    eprintln!("Price fetch error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
