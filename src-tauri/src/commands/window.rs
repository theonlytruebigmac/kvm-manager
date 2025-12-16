use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::window_state;

// Track open windows to prevent duplicates
lazy_static::lazy_static! {
    static ref OPEN_WINDOWS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// Opens a VM details window for the specified VM
/// If a window is already open for this VM, it will be focused instead
#[tauri::command]
pub async fn open_vm_details_window(
    app: AppHandle,
    vm_id: String,
    vm_name: String,
) -> Result<(), String> {
    let window_label = format!("vm-details-{}", vm_id);

    // Check if window is already open
    {
        let windows = OPEN_WINDOWS.lock().unwrap();
        if windows.contains_key(&window_label) {
            // Window already exists, just focus it
            if let Some(window) = app.get_webview_window(&window_label) {
                window.set_focus().map_err(|e| e.to_string())?;
                return Ok(());
            } else {
                // Window was closed, remove from tracking
                drop(windows);
                OPEN_WINDOWS.lock().unwrap().remove(&window_label);
            }
        }
    }

    // Create new window
    let window_title = format!("{} - VM Details", vm_name);
    let url = format!("/vms/{}", vm_id);

    let mut builder = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into())
    )
    .title(&window_title)
    .min_inner_size(900.0, 600.0);

    // Try to restore saved window state, otherwise use defaults
    if let Ok(Some(state)) = window_state::load_window_state(app.clone(), window_label.clone()).await {
        builder = builder
            .position(state.x as f64, state.y as f64)
            .inner_size(state.width as f64, state.height as f64);
    } else {
        builder = builder.inner_size(1200.0, 800.0);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    // Track the window
    OPEN_WINDOWS.lock().unwrap().insert(window_label.clone(), vm_id.clone());

    // Listen for window close to remove from tracking
    let window_label_clone = window_label.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            OPEN_WINDOWS.lock().unwrap().remove(&window_label_clone);
        }
    });

    Ok(())
}

/// Opens a console window for the specified VM
/// If a window is already open for this VM console, it will be focused instead
#[tauri::command]
pub async fn open_console_window(
    app: AppHandle,
    vm_id: String,
    vm_name: String,
) -> Result<(), String> {
    let window_label = format!("console-{}", vm_id);

    // Check if window is already open
    {
        let windows = OPEN_WINDOWS.lock().unwrap();
        if windows.contains_key(&window_label) {
            // Window already exists, just focus it
            if let Some(window) = app.get_webview_window(&window_label) {
                window.set_focus().map_err(|e| e.to_string())?;
                return Ok(());
            } else {
                // Window was closed, remove from tracking
                drop(windows);
                OPEN_WINDOWS.lock().unwrap().remove(&window_label);
            }
        }
    }

    // Create new window
    let window_title = format!("{} - Console", vm_name);
    let url = format!("/console/{}", vm_id);

    let mut builder = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into())
    )
    .title(&window_title)
    .min_inner_size(800.0, 600.0);

    // Try to restore saved window state, otherwise use defaults
    if let Ok(Some(state)) = window_state::load_window_state(app.clone(), window_label.clone()).await {
        builder = builder
            .position(state.x as f64, state.y as f64)
            .inner_size(state.width as f64, state.height as f64);
    } else {
        builder = builder.inner_size(1024.0, 768.0);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    // Track the window
    OPEN_WINDOWS.lock().unwrap().insert(window_label.clone(), vm_id.clone());

    // Listen for window close to remove from tracking
    let window_label_clone = window_label.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            OPEN_WINDOWS.lock().unwrap().remove(&window_label_clone);
        }
    });

    Ok(())
}

/// Closes all windows associated with a specific VM
/// Useful when a VM is deleted
#[tauri::command]
pub async fn close_vm_windows(
    app: AppHandle,
    vm_id: String,
) -> Result<(), String> {
    let windows_to_close: Vec<String> = {
        let windows = OPEN_WINDOWS.lock().unwrap();
        windows.iter()
            .filter(|(_, id)| **id == vm_id)
            .map(|(label, _)| label.clone())
            .collect()
    };

    for window_label in windows_to_close {
        if let Some(window) = app.get_webview_window(&window_label) {
            window.close().map_err(|e| e.to_string())?;
        }
        OPEN_WINDOWS.lock().unwrap().remove(&window_label);
    }

    Ok(())
}

/// Gets a list of all currently open window labels
#[tauri::command]
pub async fn get_open_windows() -> Result<Vec<String>, String> {
    let windows = OPEN_WINDOWS.lock().unwrap();
    Ok(windows.keys().cloned().collect())
}

/// Closes a specific window by label
#[tauri::command]
pub async fn close_window(
    app: AppHandle,
    window_label: String,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_label) {
        window.close().map_err(|e| e.to_string())?;
    }
    OPEN_WINDOWS.lock().unwrap().remove(&window_label);
    Ok(())
}
