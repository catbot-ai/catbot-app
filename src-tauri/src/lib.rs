pub mod assets;
pub mod commands;
// pub mod config;
pub mod feeder;
pub mod fetcher;
pub mod formatter;
pub mod jup;
pub mod ray;
pub mod runner;
pub mod time;
pub mod token_registry;
pub mod tray;

use chrono::Local;
use commands::core::{greet, update_token_and_price};
use feeder::{TokenOrPairAddress, TokenOrPairPriceInfo};
use formatter::update_price_display;
use jup::prices::TokenSymbol;
use log::LevelFilter;
use runner::run_loop;
use std::io::Write;
use tauri_plugin_fs::FsExt;

use tauri::{
    menu::Menu, tray::TrayIconId, LogicalSize, Manager, RunEvent, Url, WebviewUrl,
    WebviewWindowBuilder,
};
use token_registry::{get_pair_ot_token_address_from_tokens, Token, TokenRegistry};
use tokio::sync::watch::{self};
use tray::setup_tray;

use std::{collections::HashMap, sync::Mutex};

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
    tray_id: Mutex<Option<TrayIconId>>,
    tray_menu: Mutex<Option<Menu<tauri::Wry>>>,
    selected_tokens: Mutex<Vec<Token>>,
    selected_token_or_pair_address: Mutex<SelectedTokenOrPair>,
    token_sender: Mutex<Option<watch::Sender<Vec<Token>>>>,
    token_registry: Mutex<TokenRegistry>,
    price_sender: Mutex<Option<watch::Sender<HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>>>>,
    is_quit: Mutex<bool>,
    price_targets: Mutex<Vec<PriceTarget>>,
    price_watches: Mutex<Vec<String>>,
    current_public_key: Mutex<Option<String>>,
}

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use tauri::AppHandle;

// Define a struct to deserialize the YAML into
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    app_name: String,
    version: String,
    settings: Settings,
    wallets: Vec<Wallet>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Wallet {
    name: String,
    public_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Settings {
    theme: String,
    debug: bool,
}

// Tauri command to load the config
#[tauri::command]
fn load_config(app: AppHandle) -> Result<Config, String> {
    // Get the app's data directory
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // Construct the path to config.yaml
    let config_path = data_dir.join("config.yaml");

    dbg!(format!("🔥 config_path: {:?}", config_path.clone()));

    // Open and read the file
    let mut file = File::open(&config_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse the YAML content into the Config struct
    let config: Config =
        serde_yaml::from_str(&contents).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    dbg!("Config loaded: {:?}", config.clone());
    Ok(config)
}

fn initialize_config(app: AppHandle) {
    match load_config(app.clone()) {
        Ok(config) => {
            let app_state = app.state::<AppState>();
            *app_state.current_public_key.lock().unwrap() = config
                .wallets
                .first()
                .map(|wallet| wallet.public_key.clone());
        }
        Err(e) => {
            dbg!("Failed to load config: {}", e);
        }
    }
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
        .setup(|app| {
            // allowed the given directory
            let scope = app.try_fs_scope().expect("Invalid scope.");
            scope.allow_directory("./", false)?;
            dbg!(scope.is_allowed("./"));

            // Load config
            let app_handle = app.app_handle();
            initialize_config(app_handle.clone());

            let token_registry = TokenRegistry::new();
            let app_state = app.state::<AppState>();
            *app_state.token_registry.lock().unwrap() = token_registry.clone();

            let (tray_id, tray_menu) = setup_tray(app.handle()).expect("Expect tray_id");
            *app_state.tray_id.lock().unwrap() = Some(tray_id.clone());
            *app_state.tray_menu.lock().unwrap() = Some(tray_menu.clone());

            let (token_sender, mut token_receiver) = watch::channel(vec![TokenRegistry::new()
                .get_by_symbol(&TokenSymbol::SOL)
                .expect("Token not exist")
                .clone()]);
            *app_state.token_sender.lock().unwrap() = Some(token_sender);

            let (price_sender, mut price_receiver) = watch::channel::<
                HashMap<TokenOrPairAddress, TokenOrPairPriceInfo>,
            >(Default::default());
            *app_state.price_sender.lock().unwrap() = Some(price_sender.clone());

            let app_handle = app.handle().clone();

            // Default to SOL
            let selected_token = token_registry
                .get_by_symbol(&TokenSymbol::SOL)
                .expect("Invalid token")
                .clone();
            *app_state.selected_tokens.lock().unwrap() = vec![selected_token.clone()];

            let binding = selected_token.clone().address.clone();
            let address = binding.as_str();
            *app_state.selected_token_or_pair_address.lock().unwrap() = SelectedTokenOrPair {
                address: address.to_string(),
            };

            // Test
            let sol_symbol = TokenSymbol::SOL.to_string();
            let pair_symbol = format!("{}_{}", TokenSymbol::JLP, TokenSymbol::SOL);
            let price_targets = vec![
                PriceTarget {
                    token_or_pair_symbol: sol_symbol.clone(),
                    price: 200f64,
                },
                PriceTarget {
                    token_or_pair_symbol: pair_symbol.clone(),
                    price: 0.021f64,
                },
            ];
            *app_state.price_targets.lock().unwrap() = price_targets.clone();

            let price_watches = vec![sol_symbol, pair_symbol];
            *app_state.price_watches.lock().unwrap() = price_watches.clone();

            let tray_menu = app_state
                .tray_menu
                .lock()
                .unwrap()
                .clone()
                .expect("Tray not initialized");

            let tray_icon = app_handle.tray_by_id(&tray_id).expect("Tray missing");

            // Token effect
            tauri::async_runtime::spawn(async move {
                let _ = token_receiver.changed().await;
                let selected_tokens = token_receiver.borrow_and_update().clone();

                let selected_token_or_pair_address_string =
                    get_pair_ot_token_address_from_tokens(&selected_tokens)
                        .expect("Invalid token address");

                let app_state = app_handle.state::<AppState>();
                let mut selected_token_or_pair_address = app_state
                    .selected_token_or_pair_address
                    .lock()
                    .unwrap()
                    .clone();

                selected_token_or_pair_address.address =
                    selected_token_or_pair_address_string.clone();
            });

            let app_handle = app.handle();
            let cloned_app_handle = app_handle.clone();

            // Price effect
            tauri::async_runtime::spawn(async move {
                let tray_menu_clone = tray_menu.clone();

                loop {
                    let _ = price_receiver.changed().await;
                    let price_info_map = price_receiver.borrow().clone();

                    // Update tray
                    let app_state = cloned_app_handle.state::<AppState>();
                    let selected_token_or_pair_address = app_state
                        .selected_token_or_pair_address
                        .lock()
                        .unwrap()
                        .clone();

                    let maybe_price_info =
                        price_info_map.get(&selected_token_or_pair_address.address);

                    if let Some(price_info) = maybe_price_info {
                        let (_label, formatted_price) = update_price_display(price_info);
                        println!("_label:{:?}", _label);
                        let _ = tray_icon.set_title(Some(formatted_price));
                    }

                    // Update menu
                    let items = tray_menu_clone.items().unwrap();
                    price_info_map.iter().for_each(|(token_address, v)| {
                        match v {
                            TokenOrPairPriceInfo::Perp(perp_value_info) => {
                                if let Some(item) = items
                                    .iter()
                                    // TODO: Use id for single and pair
                                    .find(|menu_item| {
                                        menu_item.id().0.as_str() == perp_value_info.id
                                    })
                                {
                                    if let Some(item) = item.as_icon_menuitem() {
                                        let (_label, formatted_price) = update_price_display(v);
                                        let _ = item.set_text(formatted_price);
                                    }
                                }
                            }
                            _ => {
                                if let Some(item) = items
                                    .iter()
                                    .find(|menu_item| menu_item.id().0.as_str() == token_address)
                                {
                                    if let Some(item) = item.as_icon_menuitem() {
                                        let (_label, formatted_price) = update_price_display(v);
                                        let _ = item.set_text(formatted_price);
                                    }
                                }
                            }
                        }
                    });
                }
            });

            // // Notify
            // app.notification()
            // .builder()
            // .title(format!(
            //     "{}: ${}",
            //     price_target.token_or_pair_symbol,
            //     format_price(price)
            // ))
            // // .body(format!("${}", format_price(price)))
            // .show()
            // .unwrap();

            // TODO: Update when wallet_address changed.
            let maybe_wallet_address = app_state
                .current_public_key
                .lock()
                .unwrap()
                .clone()
                .unwrap();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_loop(
                    price_sender.clone(),
                    &token_registry,
                    Some(maybe_wallet_address.as_str()),
                )
                .await
                {
                    eprintln!("Price fetch error: {}", e);
                }
            });

            Ok(())
        })
        .on_menu_event(|app_handle, event| {
            let id = event.id.as_ref();
            let app_state = app_handle.state::<AppState>();
            let token_registry = app_state.token_registry.lock().unwrap();

            match id {
                "settings" => {
                    let window = app_handle.get_webview_window("main");

                    let window = match window {
                        Some(window) => window,
                        None => tauri::WebviewWindowBuilder::new(
                            app_handle,
                            "Settings",
                            WebviewUrl::App("index.html".into()),
                        )
                        .title("Settings")
                        .always_on_top(true)
                        .build()
                        .unwrap(),
                    };

                    let _ = window.set_size(LogicalSize::new(360, 600));

                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "quit" => {
                    *app_handle.state::<AppState>().is_quit.lock().unwrap() = true;
                    app_handle.exit(0);
                }
                "portfolio" => {
                    let app_state = app_handle.state::<AppState>();
                    let current_public_key = app_state.current_public_key.lock().unwrap();
                    let url_string = match current_public_key.as_ref() {
                        Some(public_key) => {
                            format!("https://portfolio.jup.ag/portfolio/{public_key}")
                        }
                        None => "https://portfolio.jup.ag/".to_owned(),
                    };

                    let window = app_handle.get_webview_window("portfolio");
                    let window = match window {
                        Some(window) => window,
                        None => WebviewWindowBuilder::new(
                            app_handle,
                            "portfolio",
                            WebviewUrl::External(
                                Url::parse(url_string.as_str()).expect("Invalid url"),
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
                    let app_handle = app_handle.clone();
                    let selected_tokens = token_registry
                        .get_tokens_from_pair_address(id)
                        .expect("Invalid id");

                    let price_sender = app_state.price_sender.lock().unwrap();
                    let price_sender = price_sender.as_ref().expect("Price sender not initialized");
                    let _ = update_token_and_price(app_handle, selected_tokens, price_sender);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![load_config, greet])
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
