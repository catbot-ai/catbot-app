pub mod assets;
pub mod commands;
pub mod menu_handler;
pub mod runner;
pub mod settings;
pub mod setup;
pub mod time;
pub mod tray;
pub mod tray_handler;

use chrono::Local;
use commands::core::{greet, UserCommand};
use jup_sdk::{
    feeder::{TokenOrPairAddress, TokenOrPairPriceInfo},
    prices::TokenSymbol,
    token_registry::{self, Token},
};
use log::LevelFilter;
use settings::load_settings;
use std::io::Write;

use std::{collections::HashMap, sync::Mutex};
use tauri::{Manager, RunEvent};

#[allow(dead_code)]
#[derive(Clone)]
pub struct PriceTarget {
    token_or_pair_symbol: String,
    price: f64,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct SelectedTokenOrPair {
    address: String,
}

#[derive(Default)]
pub struct AppState {
    tray_id: Mutex<Option<tauri::tray::TrayIconId>>,
    tray_menu: Mutex<Option<tauri::menu::Menu<tauri::Wry>>>,
    selected_token_or_pair_address: Mutex<SelectedTokenOrPair>,
    token_sender: Mutex<Option<tokio::sync::watch::Sender<Vec<Token>>>>,
    token_registry: Mutex<token_registry::TokenRegistry>,
    price_sender: Mutex<
        Option<tokio::sync::watch::Sender<HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>>>,
    >,
    is_quit: Mutex<bool>,
    current_public_key: Mutex<Option<String>>,
    user_command: Mutex<Option<UserCommand>>,
    command_sender: Mutex<Option<tokio::sync::watch::Sender<String>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}: {}",
                record.level(),
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .setup(setup::setup)
        .on_menu_event(menu_handler::handle_menu_event)
        .on_tray_icon_event(tray_handler::handle_tray_event)
        .invoke_handler(tauri::generate_handler![load_settings, greet])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |app_handle, e| {
        if let RunEvent::ExitRequested { api, .. } = &e {
            if !*app_handle.state::<AppState>().is_quit.lock().unwrap() {
                api.prevent_exit();
            }
        }
    });
}
