use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

use tauri::AppHandle;
use tauri::Manager;

use crate::AppState;

// Define a struct to deserialize the YAML into
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub wallets: Vec<Wallet>,
    pub recent_token_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Wallet {
    pub name: String,
    pub public_key: String,
}

// Tauri command to load the settings
#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<Settings, String> {
    // Get the app's data directory
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // Construct the path to settings.yaml
    let settings_path = data_dir.join("settings.yaml");

    // e.g. /Users/katopz/Library/Application Support/com.catbot.app/settings.yaml
    println!("🔥 settings_path: {:?}", settings_path.clone());

    // Open and read the file
    let mut file = File::open(&settings_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse the YAML content into the Settings struct
    let settings: Settings =
        serde_yaml::from_str(&contents).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    dbg!("Settings loaded: {:?}", settings.clone());
    Ok(settings)
}

pub fn initialize_settings(app: AppHandle) {
    match load_settings(app.clone()) {
        Ok(settings) => {
            let app_state = app.state::<AppState>();
            *app_state.current_public_key.lock().unwrap() = settings
                .wallets
                .first()
                .map(|wallet| wallet.public_key.clone());
        }
        Err(e) => {
            dbg!("Failed to load settings: {}", e);
        }
    }
}

pub fn update_settings(app: &AppHandle, settings: Settings) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings_path = data_dir.join("settings.yaml");
    let mut file =
        File::create(&settings_path).map_err(|e| format!("Failed to create file: {}", e))?;
    serde_yaml::to_writer(&mut file, &settings)
        .map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}
