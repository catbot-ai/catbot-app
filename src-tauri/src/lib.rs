pub mod assets;
pub mod feeder;
pub mod jup;
pub mod ray;
pub mod runner;
pub mod token_registry;
pub mod tray;

use assets::read_local_image;
use jup::{fetch_price, TokenSymbol};
use runner::run_loop;
use tauri::{tray::TrayIconId, Manager};
use tauri_plugin_notification::NotificationExt;
use token_registry::{Token, TokenRegistry};
use tokio::sync::watch;
use tray::setup_tray;

use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    tray_id: Mutex<Option<TrayIconId>>,
    selected_token: Mutex<TokenSymbol>,
    token_sender: Mutex<Option<watch::Sender<TokenSymbol>>>,
    token_registry: Mutex<TokenRegistry>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn update_token_and_price(app_handle: tauri::AppHandle, token: Token) -> Result<(), String> {
    // Update selected token
    let state = app_handle.state::<AppState>();

    // Prevent `update_token_and_price` is not `Send`
    {
        let mut selected_token = state.selected_token.lock().unwrap();
        if *selected_token == token.symbol {
            return Ok(());
        }
        *selected_token = token.symbol;
    }

    // Update tray icon and title
    let icon_path = format!("./tokens/{}.png", token.symbol);
    let icon = read_local_image(&icon_path).map_err(|e| e.to_string())?;

    if let Some(sender) = state.token_sender.lock().unwrap().as_ref() {
        sender.send(token.symbol).map_err(|e| e.to_string())?;
    }

    let tray_id = {
        state
            .tray_id
            .lock()
            .unwrap()
            .clone()
            .ok_or("Tray not initialized".to_string())?
    };

    let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
    tray_icon.set_icon(Some(icon)).map_err(|e| e.to_string())?;

    // Loading
    tray_icon.set_title(Some("…")).map_err(|e| e.to_string())?;

    // Fetch price
    match fetch_price(&token.address).await {
        Ok(price) => {
            tray_icon
                .set_title(Some(&format!("${:.2}", price)))
                .map_err(|e| e.to_string())?;
        }
        Err(e) => {
            tray_icon
                .set_title(Some("Error"))
                .map_err(|e| e.to_string())?;
            eprintln!("Price fetch failed: {}", e);
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .on_menu_event(|app_handle, event| {
            let id = event.id.as_ref();
            let state = app_handle.state::<AppState>();
            let registry = state.token_registry.lock().unwrap();
            if let Some(token) = registry.get_by_address(id) {
                let token = token.clone();
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    // Spawn a new async task
                    if let Err(e) = update_token_and_price(app_handle, token).await {
                        eprintln!("Error updating token and price: {}", e);
                    }
                });
            } else {
                // TODO?
            }
        })
        .setup(|app| {
            let token_registry = TokenRegistry::new();
            *app.state::<AppState>().token_registry.lock().unwrap() = token_registry;

            let tray_id = setup_tray(app.handle()).expect("Expect tray_id");
            *app.state::<AppState>().tray_id.lock().unwrap() = Some(tray_id.clone());

            let (token_sender, token_receiver) = watch::channel(TokenSymbol::SOL);
            *app.state::<AppState>().token_sender.lock().unwrap() = Some(token_sender);
            *app.state::<AppState>().selected_token.lock().unwrap() = TokenSymbol::SOL;

            let (price_sender, price_receiver) = watch::channel(None);
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let mut price_receiver = price_receiver.clone();
                loop {
                    let _ = price_receiver.changed().await;
                    let price = *price_receiver.borrow_and_update();
                    if let Some(price) = price {
                        // let state = app_handle.state::<AppState>();
                        // if let Some(tray_id) = state.tray_id.lock().unwrap().as_ref() {
                        let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");
                        let _ = tray_icon.set_title(Some(&format!("${:.2}", price)));
                        // }
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
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![greet, update_token_and_price]) // Add the new command
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
