use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn get_window_states_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;

    fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

    Ok(config_dir.join("window_states.json"))
}

#[tauri::command]
pub async fn save_window_state(
    app: AppHandle,
    label: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let state = WindowState {
        x,
        y,
        width,
        height,
    };

    let state_file = get_window_states_path(&app)?;

    // Load existing states
    let mut states: HashMap<String, WindowState> = if state_file.exists() {
        let content = fs::read_to_string(&state_file).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Update state
    states.insert(label, state);

    // Save back
    let content = serde_json::to_string_pretty(&states).map_err(|e| e.to_string())?;
    fs::write(&state_file, content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn load_window_state(
    app: AppHandle,
    label: String,
) -> Result<Option<WindowState>, String> {
    let state_file = get_window_states_path(&app)?;

    if !state_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&state_file).map_err(|e| e.to_string())?;
    let states: HashMap<String, WindowState> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(states.get(&label).cloned())
}

#[tauri::command]
pub async fn clear_window_states(app: AppHandle) -> Result<(), String> {
    let state_file = get_window_states_path(&app)?;

    if state_file.exists() {
        fs::remove_file(&state_file).map_err(|e| e.to_string())?;
    }

    Ok(())
}
