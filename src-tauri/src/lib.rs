pub mod assets;
pub mod feeder;
pub mod jup;
pub mod ray;
pub mod runner;
pub mod tray;

use runner::run_loop;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::watch;
use tray::setup_tray;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "about" => {
                // TODO
            }
            "setting" => {
                // TODO
            }
            _ => {}
        })
        .setup(|app| {
            // Tray and menu setup
            let tray_id = setup_tray(app).expect("Tray setup failed");

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

                        // let icon = include_image!("./icons/32x32-notification.png");
                        // let _ = tray.set_icon(Some(icon));
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
