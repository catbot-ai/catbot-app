pub mod feeder;
pub mod ray;
pub mod runner;

use runner::run_loop;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use wasm_timer::Delay;

use std::{sync::Arc, time::Duration};
use tauri::async_runtime::Mutex;

struct AppState {
    price: Arc<Mutex<Option<f64>>>, // Store the price here
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            price: Arc::new(Mutex::new(None)), // Initialize with no price
        })
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
            let price_state = app.try_state::<AppState>().unwrap().price.clone();

            // Spawn async task with owned values
            tauri::async_runtime::spawn(async move {
                // Split into two independent clones
                let run_loop_state = price_state.clone();
                let tray_update_state = price_state;

                // Price fetching task
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = run_loop(run_loop_state).await {
                        eprintln!("Price fetch error: {}", e);
                    }
                });

                println!("Tray icon initialized");

                // Tray update loop
                loop {
                    let price = {
                        let mut guard = tray_update_state.lock().await;
                        guard.take()
                    };

                    if let Some(price) = price {
                        let _ = app_handle
                            .tray_by_id(&tray_id)
                            .expect("Tray icon missing")
                            .set_title(Some(&format!("${:.2}", price)));
                    }

                    Delay::new(Duration::from_secs(5)).await.ok();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
