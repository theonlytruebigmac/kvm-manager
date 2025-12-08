// Module declarations
mod commands;
mod models;
mod services;
mod state;
mod utils;

use state::app_state::AppState;
use tracing_subscriber;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting KVM Manager");

    // Initialize application state
    let app_state = match AppState::new() {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Failed to initialize AppState: {}", e);
            eprintln!("Failed to connect to libvirt: {}", e);
            eprintln!("Please ensure libvirtd is running and you have proper permissions.");
            eprintln!("Try: sudo systemctl start libvirtd");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // VM Commands
            commands::vm::get_vms,
            commands::vm::get_vm,
            commands::vm::start_vm,
            commands::vm::stop_vm,
            commands::vm::force_stop_vm,
            commands::vm::pause_vm,
            commands::vm::resume_vm,
            commands::vm::reboot_vm,
            commands::vm::delete_vm,
            commands::vm::clone_vm,
            commands::vm::create_vm,
            commands::vm::get_vm_stats,
            commands::vm::add_vm_tags,
            commands::vm::remove_vm_tags,
            // Network Commands
            commands::network::get_networks,
            commands::network::get_network,
            commands::network::create_network,
            commands::network::delete_network,
            commands::network::start_network,
            commands::network::stop_network,
            // Storage Commands
            commands::storage::get_storage_pools,
            commands::storage::get_volumes,
            commands::storage::create_volume,
            commands::storage::delete_volume,
            // Snapshot Commands
            commands::snapshot::get_snapshots,
            commands::snapshot::create_snapshot,
            commands::snapshot::delete_snapshot,
            commands::snapshot::revert_snapshot,
            // System Commands
            commands::system::get_host_info,
            commands::system::get_connection_status,
            commands::system::get_vnc_info,
            commands::system::open_vnc_console,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
