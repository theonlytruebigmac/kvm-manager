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

    // Start background tasks
    app_state.start_background_tasks();

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
            commands::vm::export_vm,
            commands::vm::import_vm,
            commands::vm::attach_disk,
            commands::vm::detach_disk,
            commands::vm::batch_start_vms,
            commands::vm::batch_stop_vms,
            commands::vm::batch_reboot_vms,
            // Network Commands
            commands::network::get_networks,
            commands::network::get_network,
            commands::network::create_network,
            commands::network::delete_network,
            commands::network::start_network,
            commands::network::stop_network,
            commands::network::add_port_forward,
            commands::network::remove_port_forward,
            // Storage Commands
            commands::storage::get_storage_pools,
            commands::storage::get_volumes,
            commands::storage::create_volume,
            commands::storage::delete_volume,
            commands::storage::create_storage_pool,
            commands::storage::resize_volume,
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
            // Metrics Commands
            commands::metrics::store_vm_metrics,
            commands::metrics::get_historical_metrics,
            commands::metrics::cleanup_old_metrics,
            commands::metrics::get_metrics_count,
            // Template Commands
            commands::template::create_template,
            commands::template::list_templates,
            commands::template::get_template,
            commands::template::update_template,
            commands::template::delete_template,
            // Scheduler Commands
            commands::scheduler::create_schedule,
            commands::scheduler::list_schedules,
            commands::scheduler::get_schedule,
            commands::scheduler::update_schedule_status,
            commands::scheduler::delete_schedule,
            commands::scheduler::get_vm_schedules,
            // Alert Commands
            commands::alert::create_alert,
            commands::alert::list_alerts,
            commands::alert::get_alert,
            commands::alert::update_alert_status,
            commands::alert::delete_alert,
            commands::alert::get_vm_alerts,
            commands::alert::check_alert_threshold,
            // Backup Commands
            commands::backup::create_backup_config,
            commands::backup::list_backup_configs,
            commands::backup::get_backup_config,
            commands::backup::update_backup_status,
            commands::backup::delete_backup_config,
            commands::backup::get_vm_backup_configs,
            commands::backup::record_backup,
            // Optimization Commands
            commands::optimization::analyze_vm_performance,
            commands::optimization::analyze_all_vms,
            // Retention Commands
            commands::retention::get_retention_policy,
            commands::retention::update_retention_policy,
            commands::retention::execute_retention_cleanup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
