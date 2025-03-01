use crate::{
    commands::core::{get_suggestion, update_token_and_price, UserCommand},
    settings::{load_settings, update_settings},
    AppState,
};
use jup_sdk::{prices::TokenSymbol, token_registry::get_symbol_pair_from_tokens};
use log::info;
use tauri::{AppHandle, LogicalSize, Manager, Url, WebviewUrl, WebviewWindowBuilder};

pub fn handle_menu_event(app_handle: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id.as_ref();
    let app_state = app_handle.state::<AppState>();
    let token_registry = app_state.token_registry.lock().unwrap();
    info!("id:{:?}", id);
    match id {
        "suggest" => {
            info!("suggest");
            // Get current wallet address.
            let current_public_key = app_state
                .current_public_key
                .lock()
                .unwrap()
                .clone()
                .expect("Expect wallet address");

            info!("current_public_key:{}", current_public_key);

            // Get signals
            let command_sender = app_state.command_sender.lock().unwrap();
            let command_sender = command_sender
                .as_ref()
                .expect("Command sender not initialized");
            *app_handle.state::<AppState>().user_command.lock().unwrap() =
                Some(UserCommand::Suggest);

            let app_handle_clone = app_handle.clone();

            // TODO: support other symbol
            let symbol_pair_string = TokenSymbol::SOL.to_string();

            tauri::async_runtime::spawn(async move {
                let _ = get_suggestion(
                    app_handle_clone,
                    &current_public_key.clone(),
                    &symbol_pair_string,
                )
                .await;
            });
        }
        "settings" => {
            let maybe_window = app_handle.get_webview_window("main");
            let window = match maybe_window {
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
            let _ = window.show();
            let _ = window.set_focus();
        }
        "quit" => {
            *app_handle.state::<AppState>().is_quit.lock().unwrap() = true;
            app_handle.exit(0);
        }
        "portfolio" => {
            let app_state = app_handle.state::<AppState>();
            let current_public_key = app_state.current_public_key.lock().unwrap();
            let url_string = match current_public_key.as_ref() {
                Some(public_key) => format!("https://portfolio.jup.ag/portfolio/{public_key}"),
                None => "https://portfolio.jup.ag/".to_owned(),
            };

            let maybe_window = app_handle.get_webview_window("portfolio");
            let window = match maybe_window {
                Some(window) => window,
                None => WebviewWindowBuilder::new(
                    app_handle,
                    "portfolio",
                    WebviewUrl::External(Url::parse(url_string.as_str()).expect("Invalid url")),
                )
                .always_on_top(true)
                .build()
                .unwrap(),
            };

            let _ = window.set_size(LogicalSize::new(640, 480));
            let _ = window.show();
            let _ = window.set_focus();
        }
        _ => {
            let app_handle_clone = app_handle.clone();
            let selected_tokens = token_registry.get_tokens_from_pair_address(id);

            if let Ok(mut settings) = load_settings(app_handle.clone()) {
                settings.recent_token_id = Some(id.to_string());
                let _ = update_settings(app_handle, settings);
            }

            let price_sender = app_state.price_sender.lock().unwrap();
            let price_sender = price_sender.as_ref().expect("Price sender not initialized");
            let _ = update_token_and_price(app_handle_clone, selected_tokens, price_sender);
        }
    }
}
