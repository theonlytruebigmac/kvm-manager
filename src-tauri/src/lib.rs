// Module declarations
mod commands;
mod models;
mod services;
mod state;
mod utils;
// mod menu; // Native menu disabled - functionality available via toolbar
mod window_state;

use state::app_state::AppState;
use tauri::Manager;
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
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .setup(|app| {
            // Note: Native menu bar disabled for cleaner look
            // Menu functionality available via toolbar and keyboard shortcuts

            // Start background tasks after Tauri runtime is initialized
            let app_state = app.state::<AppState>();
            app_state.start_background_tasks();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // VM Commands
            commands::vm::get_vms,
            commands::vm::get_vm,
            commands::vm::start_vm,
            commands::vm::stop_vm,
            commands::vm::force_stop_vm,
            commands::vm::pause_vm,
            commands::vm::resume_vm,
            commands::vm::hibernate_vm,
            commands::vm::has_managed_save,
            commands::vm::remove_managed_save,
            commands::vm::reboot_vm,
            commands::vm::delete_vm,
            commands::vm::clone_vm,
            commands::vm::clone_vm_with_options,
            commands::vm::create_vm,
            commands::vm::get_vm_stats,
            commands::vm::add_vm_tags,
            commands::vm::remove_vm_tags,
            commands::vm::export_vm,
            commands::vm::import_vm,
            commands::vm::attach_disk,
            commands::vm::detach_disk,
            commands::vm::update_disk_settings,
            commands::vm::get_kernel_boot_settings,
            commands::vm::set_kernel_boot_settings,
            commands::vm::get_hugepages_settings,
            commands::vm::set_hugepages,
            commands::vm::get_host_hugepage_info,
            commands::vm::update_interface_bandwidth,
            commands::vm::set_interface_link_state,
            commands::vm::get_interface_link_state,
            commands::vm::batch_start_vms,
            commands::vm::batch_stop_vms,
            commands::vm::batch_reboot_vms,
            commands::vm::mount_iso,
            commands::vm::mount_guest_agent_iso,
            commands::vm::eject_cdrom,
            commands::vm::update_vm_boot_order,
            commands::vm::rename_vm,
            commands::vm::get_vm_autostart,
            commands::vm::set_vm_autostart,
            commands::vm::attach_interface,
            commands::vm::attach_interface_advanced,
            commands::vm::list_host_interfaces,
            commands::vm::detach_interface,
            commands::vm::attach_sound,
            commands::vm::detach_sound,
            commands::vm::attach_input,
            commands::vm::attach_rng,
            commands::vm::attach_watchdog,
            commands::vm::attach_channel,
            commands::vm::attach_filesystem,
            commands::vm::attach_graphics,
            commands::vm::attach_video,
            commands::vm::set_vm_vcpus,
            commands::vm::set_vm_memory,
            commands::vm::set_vm_cpu_topology,
            commands::vm::get_cpu_model,
            commands::vm::set_cpu_model,
            commands::vm::get_available_cpu_models,
            commands::vm::get_cpu_pinning,
            commands::vm::set_cpu_pin,
            commands::vm::clear_cpu_pin,
            commands::vm::attach_serial,
            commands::vm::attach_console,
            commands::vm::attach_tpm,
            commands::vm::attach_usb_controller,
            commands::vm::get_usb_redirection,
            commands::vm::attach_usb_redirection,
            commands::vm::remove_usb_redirection,
            commands::vm::attach_scsi_controller,
            commands::vm::attach_panic_notifier,
            commands::vm::attach_vsock,
            commands::vm::attach_parallel,
            commands::vm::attach_smartcard,
            commands::vm::get_host_numa_topology,
            commands::vm::get_vm_numa_config,
            commands::vm::set_vm_numa_config,
            commands::vm::clear_vm_numa_config,
            commands::vm::migrate_vm,
            commands::vm::get_migration_info,
            commands::vm::check_migration_compatibility,
            commands::vm::get_migration_targets,
            commands::vm::list_evdev_devices,
            commands::vm::attach_evdev,
            commands::vm::get_vm_evdev_devices,
            commands::vm::detach_evdev,
            // Network Commands
            commands::network::get_networks,
            commands::network::get_network,
            commands::network::create_network,
            commands::network::delete_network,
            commands::network::start_network,
            commands::network::stop_network,
            commands::network::add_port_forward,
            commands::network::remove_port_forward,
            commands::network::set_network_autostart,
            commands::network::get_dhcp_leases,
            commands::network::get_network_details,
            // NwFilter Commands
            commands::nwfilter::get_nwfilters,
            commands::nwfilter::get_nwfilter,
            commands::nwfilter::get_nwfilter_xml,
            commands::nwfilter::create_nwfilter,
            commands::nwfilter::create_nwfilter_from_xml,
            commands::nwfilter::delete_nwfilter,
            // Storage Commands
            commands::storage::get_storage_pools,
            commands::storage::get_volumes,
            commands::storage::create_volume,
            commands::storage::delete_volume,
            commands::storage::create_storage_pool,
            commands::storage::resize_volume,
            commands::storage::upload_volume,
            commands::storage::download_volume,
            commands::storage::get_volume_path,
            commands::storage::get_volume_encryption_info,
            commands::storage::get_ova_metadata,
            commands::storage::import_ova,
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
            commands::system::stop_vnc_proxy,
            // Connection Commands
            commands::connection::get_saved_connections,
            commands::connection::get_active_connection,
            commands::connection::connect_to,
            commands::connection::disconnect_from,
            commands::connection::add_connection,
            commands::connection::update_connection,
            commands::connection::remove_connection,
            commands::connection::test_connection,
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
            // Guest Agent Commands
            commands::guest_agent::check_guest_agent_status,
            commands::guest_agent::get_guest_system_info,
            commands::guest_agent::get_guest_network_info,
            commands::guest_agent::get_guest_disk_usage,
            commands::guest_agent::execute_guest_command,
            commands::guest_agent::read_guest_file,
            commands::guest_agent::write_guest_file,
            commands::guest_agent::guest_agent_shutdown,
            commands::guest_agent::guest_agent_reboot,
            commands::guest_agent::get_guest_cpu_stats,
            commands::guest_agent::get_guest_disk_stats,
            commands::guest_agent::get_guest_users,
            commands::guest_agent::get_guest_timezone,
            commands::guest_agent::get_guest_full_info,
            // PCI Device Commands
            commands::pci::list_pci_devices,
            commands::pci::check_iommu_status,
            commands::pci::get_iommu_groups,
            commands::pci::attach_pci_device,
            commands::pci::detach_pci_device,
            commands::pci::get_vfio_status,
            commands::pci::bind_to_vfio,
            commands::pci::unbind_from_vfio,
            // USB Device Commands
            commands::usb::list_usb_devices,
            commands::usb::attach_usb_device,
            commands::usb::detach_usb_device,
            commands::usb::get_vm_usb_devices,
            // MDEV (vGPU) Commands
            commands::mdev::check_mdev_status,
            commands::mdev::list_mdev_types,
            commands::mdev::list_mdev_devices,
            commands::mdev::attach_mdev,
            commands::mdev::detach_mdev,
            commands::mdev::create_mdev,
            commands::mdev::remove_mdev,
            // Window Management Commands
            commands::window::open_vm_details_window,
            commands::window::open_console_window,
            commands::window::close_vm_windows,
            commands::window::get_open_windows,
            commands::window::close_window,
            // Serial Console Commands
            commands::serial_console::get_serial_console_info,
            commands::serial_console::open_serial_console,
            commands::serial_console::close_serial_console,
            commands::serial_console::read_serial_console,
            commands::serial_console::write_serial_console,
            commands::serial_console::is_serial_console_connected,
            // SR-IOV Commands
            commands::sriov::list_sriov_pfs,
            commands::sriov::list_sriov_vfs,
            commands::sriov::enable_sriov_vfs,
            commands::sriov::configure_sriov_vf,
            commands::sriov::attach_sriov_vf,
            commands::sriov::detach_sriov_vf,
            // Window State Persistence Commands
            window_state::save_window_state,
            window_state::load_window_state,
            window_state::clear_window_states,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
