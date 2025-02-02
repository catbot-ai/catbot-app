pub mod assets;
pub mod commands;
pub mod feeder;
pub mod jup;
pub mod ray;
pub mod runner;
pub mod token_registry;
pub mod tray;

use commands::core::{greet, update_token_and_price};
use jup::TokenSymbol;
use runner::run_loop;
use tauri::{
    tray::TrayIconId, LogicalSize, Manager, RunEvent, Url, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_notification::NotificationExt;
use token_registry::{Token, TokenRegistry};
use tokio::sync::watch;
use tray::setup_tray;

use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    tray_id: Mutex<Option<TrayIconId>>,
    selected_token: Mutex<Token>,
    token_sender: Mutex<Option<watch::Sender<Vec<Token>>>>,
    token_registry: Mutex<TokenRegistry>,
    is_quit: Mutex<bool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .setup(|app| {
            let token_registry = TokenRegistry::new();
            *app.state::<AppState>().token_registry.lock().unwrap() = token_registry;

            let tray_id = setup_tray(app.handle()).expect("Expect tray_id");
            *app.state::<AppState>().tray_id.lock().unwrap() = Some(tray_id.clone());

            let (token_sender, token_receiver) = watch::channel(vec![TokenRegistry::new()
                .get_by_symbol(&TokenSymbol::SOL)
                .expect("Token ot exist")
                .clone()]);
            *app.state::<AppState>().token_sender.lock().unwrap() = Some(token_sender);

            let (price_sender, price_receiver) = watch::channel(None);
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let mut price_receiver = price_receiver.clone();
                loop {
                    let _ = price_receiver.changed().await;
                    let price = *price_receiver.borrow_and_update();
                    if let Some(price) = price {
                        let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
                        let _ = tray_icon.set_title(Some(&format!("${:.2}", price)));
                    }
                }
            });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_loop(price_sender, token_receiver).await {
                    eprintln!("Price fetch error: {}", e);
                }
            });

            app.notification()
                .builder()
                .title("Hello")
                .body("World! 😎")
                .show()
                .unwrap();

            Ok(())
        })
        .on_menu_event(|app_handle, event| {
            let id = event.id.as_ref();
            let state = app_handle.state::<AppState>();
            let registry = state.token_registry.lock().unwrap();

            match id {
                "quit" => {
                    *app_handle.state::<AppState>().is_quit.lock().unwrap() = true;
                    app_handle.exit(0);
                }
                "portfolio" => {
                    let window = app_handle.get_webview_window("main");

                    let window = match window {
                        Some(window) => window,
                        None => WebviewWindowBuilder::new(
                            app_handle,
                            "main",
                            WebviewUrl::External(
                                Url::parse("https://portfolio.jup.ag/").expect("Invalid url"),
                            ),
                        )
                        .always_on_top(true)
                        .build()
                        .unwrap(),
                    };

                    let _ = window.set_size(LogicalSize::new(360, 600));

                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                _ => {
                    if let Some(token) = registry.get_by_address(id) {
                        let token = token.clone();
                        let app_handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            // Spawn a new async task
                            if let Err(e) = update_token_and_price(app_handle, token).await {
                                eprintln!("Error updating token and price: {}", e);
                            }
                        });
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![greet, update_token_and_price])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |app_handle, e| {
        if let RunEvent::ExitRequested { api, .. } = &e {
            // Keep the event loop running even if all windows are closed
            // This allow us to catch system tray events when there is no window
            if !*app_handle.state::<AppState>().is_quit.lock().unwrap() {
                api.prevent_exit();
            }
        }
    });
}
