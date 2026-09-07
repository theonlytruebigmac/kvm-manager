use crate::utils::error::{AppError, SafeFailure};
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

fn window_state_failure(detail: impl Into<String>) -> SafeFailure {
    AppError::WindowFailure(detail.into()).into()
}

fn get_window_states_path(app: &AppHandle) -> Result<PathBuf, Box<SafeFailure>> {
    let config_dir = app.path().app_config_dir().map_err(|_| {
        Box::new(window_state_failure(
            "window state directory is unavailable",
        ))
    })?;

    fs::create_dir_all(&config_dir).map_err(|_| {
        Box::new(window_state_failure(
            "window state directory cannot be created",
        ))
    })?;

    Ok(config_dir.join("window_states.json"))
}

fn parse_window_states(content: &str) -> Result<HashMap<String, WindowState>, Box<SafeFailure>> {
    serde_json::from_str(content)
        .map_err(|_| Box::new(window_state_failure("saved window state is malformed")))
}

#[tauri::command]
pub async fn save_window_state(
    app: AppHandle,
    label: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), SafeFailure> {
    let state = WindowState {
        x,
        y,
        width,
        height,
    };

    let state_file = get_window_states_path(&app).map_err(|failure| *failure)?;

    // Load existing states
    let mut states: HashMap<String, WindowState> = if state_file.exists() {
        let content = fs::read_to_string(&state_file)
            .map_err(|_| window_state_failure("saved window state cannot be read"))?;
        parse_window_states(&content).map_err(|failure| *failure)?
    } else {
        HashMap::new()
    };

    // Update state
    states.insert(label, state);

    // Save back
    let content = serde_json::to_string_pretty(&states)
        .map_err(|_| window_state_failure("window state cannot be serialized"))?;
    fs::write(&state_file, content)
        .map_err(|_| window_state_failure("window state cannot be saved"))?;

    Ok(())
}

#[tauri::command]
pub async fn load_window_state(
    app: AppHandle,
    label: String,
) -> Result<Option<WindowState>, SafeFailure> {
    let state_file = get_window_states_path(&app).map_err(|failure| *failure)?;

    if !state_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&state_file)
        .map_err(|_| window_state_failure("saved window state cannot be read"))?;
    let states = parse_window_states(&content).map_err(|failure| *failure)?;

    Ok(states.get(&label).cloned())
}

#[tauri::command]
pub async fn clear_window_states(app: AppHandle) -> Result<(), SafeFailure> {
    let state_file = get_window_states_path(&app).map_err(|failure| *failure)?;

    if state_file.exists() {
        fs::remove_file(&state_file)
            .map_err(|_| window_state_failure("saved window state cannot be cleared"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error::SafeFailureCode;

    #[test]
    fn malformed_saved_window_state_is_safe_and_recoverable() {
        let failure = parse_window_states("{invalid window state").unwrap_err();

        assert_eq!(failure.code, SafeFailureCode::Integration);
        assert_eq!(
            failure.summary,
            "The requested desktop window could not be opened. Retry the operation."
        );
        assert!(!failure.summary.contains("invalid window state"));
    }
}
