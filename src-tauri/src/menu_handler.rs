use crate::{
    commands::core::update_token_and_price,
    settings::{load_settings, update_settings},
    AppState,
};
use tauri::{AppHandle, LogicalSize, Manager, Url, WebviewUrl, WebviewWindowBuilder};

pub fn handle_menu_event(app_handle: &AppHandle, event: tauri::menu::MenuEvent) {
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
                Some(public_key) => format!("https://portfolio.jup.ag/portfolio/{public_key}"),
                None => "https://portfolio.jup.ag/".to_owned(),
            };

            let window = app_handle.get_webview_window("portfolio");
            let window = match window {
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

            let _ = window.set_size(LogicalSize::new(360, 600));

            window.show().unwrap();
            window.set_focus().unwrap();
        }
        _ => {
            let app_handle_clone = app_handle.clone();
            let selected_tokens = token_registry
                .get_tokens_from_pair_address(id)
                .expect("Invalid id");

            if let Ok(mut settings) = load_settings(app_handle.clone()) {
                settings.recent_token_id = Some(id.to_string());
                let _ = update_settings(&app_handle, settings);
            }

            let price_sender = app_state.price_sender.lock().unwrap();
            let price_sender = price_sender.as_ref().expect("Price sender not initialized");
            let _ = update_token_and_price(app_handle_clone, selected_tokens, price_sender);
        }
    }
}
