pub mod assets;
pub mod feeder;
pub mod jup;
pub mod ray;
pub mod runner;
pub mod tray;

use jup::TokenSymbol;
use runner::run_loop;
use tauri::{include_image, tray::TrayIconId};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::watch;
use tray::setup_tray;

use std::sync::Mutex;
use tauri::Manager;

#[derive(Default)]
pub struct AppState {
    tray_id: Mutex<Option<TrayIconId>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default()) // Add the state here
        .on_menu_event(move |app_handle, event| {
            match event.id.as_ref() {
                "quit" => {
                    app_handle.exit(0);
                }
                "about" => {
                    // TODO
                }
                "setting" => {
                    // TODO
                }
                "So11111111111111111111111111111111111111112" => {
                    let icon = include_image!("./icons/SOL.png");

                    // Retrieve the tray ID from the state
                    let state = app_handle.state::<AppState>();
                    let tray_id = state.tray_id.lock().unwrap().as_ref().unwrap().clone();

                    // Use tray_by_id to set the icon
                    let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
                    tray_icon
                        .set_icon(Some(icon))
                        .expect("Failed to set tray icon");
                }
                "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4" => {
                    let icon = include_image!("./icons/JLP.png");

                    // Retrieve the tray ID from the state
                    let state = app_handle.state::<AppState>();
                    let tray_id = state.tray_id.lock().unwrap().as_ref().unwrap().clone();

                    // Use tray_by_id to set the icon
                    let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
                    tray_icon
                        .set_icon(Some(icon))
                        .expect("Failed to set tray icon");
                }
                _ => {}
            }
        })
        .setup(move |app| {
            // Create the tray icon and store its ID in the state
            let tray_id = setup_tray(app.handle(), TokenSymbol::SOL).expect("expect tray_id");

            // Store the tray ID in the state
            let state = app.state::<AppState>();
            *state.tray_id.lock().unwrap() = Some(tray_id.clone());

            // Feed
            let app_handle = app.handle().clone();
            let (price_sender, price_receiver) = watch::channel(None);
            tauri::async_runtime::spawn(async move {
                let mut price_receiver = price_receiver.clone();
                loop {
                    // Wait for price changes
                    let _ = price_receiver.changed().await;
                    let price = *price_receiver.borrow_and_update();
                    if let Some(price) = price {
                        let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
                        let _ = tray_icon.set_title(Some(&format!("${:.2}", price)));
                    }
                }
            });

            // Start price fetch loop with sender
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_loop(price_sender).await {
                    eprintln!("Price fetch error: {}", e);
                }
            });

            let _ = app
                .notification()
                .builder()
                .title("Hello")
                .body("World! 😎")
                .show();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
