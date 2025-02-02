use crate::assets::read_local_image;
use crate::jup::fetch_price;
use crate::token_registry::Token;
use crate::AppState;
use tauri::Manager;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub async fn update_token_and_price(
    app_handle: tauri::AppHandle,
    token: Token,
) -> Result<(), String> {
    // Update selected token
    let state = app_handle.state::<AppState>();

    // Prevent `update_token_and_price` is not `Send`
    {
        let mut selected_token = state.selected_token.lock().unwrap();
        if *selected_token == token.clone() {
            return Ok(());
        }
        *selected_token = token.clone();
    }

    // Update tray icon and title
    let icon_path = format!("./tokens/{}.png", token.symbol);
    let icon = read_local_image(&icon_path).map_err(|e| e.to_string())?;

    if let Some(sender) = state.token_sender.lock().unwrap().as_ref() {
        sender
            .send(vec![token.clone()])
            .map_err(|e| e.to_string())?;
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
