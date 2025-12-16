use tauri::{
    AppHandle, Manager, Wry, Emitter,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu, MenuEvent},
};

/// Creates the application menu bar
pub fn create_menu(app: &AppHandle) -> Result<Menu<Wry>, tauri::Error> {
    // File menu
    let new_vm = MenuItem::with_id(app, "new_vm", "New VM...", true, Some("Ctrl+N"))?;
    let import_vm = MenuItem::with_id(app, "import_vm", "Import VM...", true, Some("Ctrl+I"))?;
    let new_connection = MenuItem::with_id(app, "new_connection", "New Connection...", true, None::<&str>)?;
    let close_connection = MenuItem::with_id(app, "close_connection", "Close Connection", true, None::<&str>)?;
    let preferences = MenuItem::with_id(app, "preferences", "Preferences...", true, Some("Ctrl+,"))?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, Some("Ctrl+Q"))?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &new_vm,
            &import_vm,
            &PredefinedMenuItem::separator(app)?,
            &new_connection,
            &close_connection,
            &PredefinedMenuItem::separator(app)?,
            &preferences,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    // Edit menu
    let vm_details = MenuItem::with_id(app, "vm_details", "VM Details...", true, Some("Ctrl+D"))?;
    let clone_vm = MenuItem::with_id(app, "clone_vm", "Clone VM...", true, None::<&str>)?;
    let rename_vm = MenuItem::with_id(app, "rename_vm", "Rename...", true, None::<&str>)?;
    let delete_vm = MenuItem::with_id(app, "delete_vm", "Delete...", true, Some("Delete"))?;
    let take_snapshot = MenuItem::with_id(app, "take_snapshot", "Take Snapshot...", true, Some("Ctrl+S"))?;
    let manage_snapshots = MenuItem::with_id(app, "manage_snapshots", "Manage Snapshots...", true, None::<&str>)?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &vm_details,
            &clone_vm,
            &rename_vm,
            &delete_vm,
            &PredefinedMenuItem::separator(app)?,
            &take_snapshot,
            &manage_snapshots,
        ],
    )?;

    // View menu
    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, Some("F5"))?;
    let toggle_toolbar = MenuItem::with_id(app, "toggle_toolbar", "Toolbar", true, None::<&str>)?;
    let toggle_statusbar = MenuItem::with_id(app, "toggle_statusbar", "Status Bar", true, None::<&str>)?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &refresh,
            &PredefinedMenuItem::separator(app)?,
            &toggle_toolbar,
            &toggle_statusbar,
        ],
    )?;

    // Actions menu
    let start_vm = MenuItem::with_id(app, "start_vm", "Start", true, Some("Ctrl+Up"))?;
    let stop_vm = MenuItem::with_id(app, "stop_vm", "Stop", true, Some("Ctrl+Down"))?;
    let force_stop_vm = MenuItem::with_id(app, "force_stop_vm", "Force Stop", true, Some("Ctrl+Shift+Down"))?;
    let pause_vm = MenuItem::with_id(app, "pause_vm", "Pause", true, Some("Ctrl+P"))?;
    let resume_vm = MenuItem::with_id(app, "resume_vm", "Resume", true, None::<&str>)?;
    let reboot_vm = MenuItem::with_id(app, "reboot_vm", "Reboot", true, Some("Ctrl+R"))?;
    let open_console = MenuItem::with_id(app, "open_console", "Open Console", true, Some("Ctrl+O"))?;

    let actions_menu = Submenu::with_items(
        app,
        "Actions",
        true,
        &[
            &start_vm,
            &stop_vm,
            &force_stop_vm,
            &pause_vm,
            &resume_vm,
            &reboot_vm,
            &PredefinedMenuItem::separator(app)?,
            &open_console,
        ],
    )?;

    // Tools menu
    let storage_manager = MenuItem::with_id(app, "storage_manager", "Storage Manager...", true, None::<&str>)?;
    let network_manager = MenuItem::with_id(app, "network_manager", "Network Manager...", true, None::<&str>)?;
    let templates = MenuItem::with_id(app, "templates", "Templates...", true, None::<&str>)?;
    let schedules = MenuItem::with_id(app, "schedules", "Schedules...", true, None::<&str>)?;
    let alerts = MenuItem::with_id(app, "alerts", "Alerts...", true, None::<&str>)?;
    let backups = MenuItem::with_id(app, "backups", "Backups...", true, None::<&str>)?;
    let performance = MenuItem::with_id(app, "performance", "Performance Monitor", true, None::<&str>)?;
    let optimization = MenuItem::with_id(app, "optimization", "Optimization Suggestions", true, None::<&str>)?;

    let tools_menu = Submenu::with_items(
        app,
        "Tools",
        true,
        &[
            &storage_manager,
            &network_manager,
            &PredefinedMenuItem::separator(app)?,
            &templates,
            &schedules,
            &alerts,
            &backups,
            &PredefinedMenuItem::separator(app)?,
            &performance,
            &optimization,
        ],
    )?;

    // Help menu
    let documentation = MenuItem::with_id(app, "documentation", "Documentation", true, None::<&str>)?;
    let keyboard_shortcuts = MenuItem::with_id(app, "keyboard_shortcuts", "Keyboard Shortcuts", true, Some("Ctrl+?"))?;
    let check_updates = MenuItem::with_id(app, "check_updates", "Check for Updates...", true, None::<&str>)?;
    let report_issue = MenuItem::with_id(app, "report_issue", "Report Issue...", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About KVM Manager", true, None::<&str>)?;

    let help_menu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &documentation,
            &keyboard_shortcuts,
            &PredefinedMenuItem::separator(app)?,
            &check_updates,
            &report_issue,
            &PredefinedMenuItem::separator(app)?,
            &about,
        ],
    )?;

    // Create the menu bar
    let menu = Menu::with_items(
        app,
        &[
            &file_menu,
            &edit_menu,
            &view_menu,
            &actions_menu,
            &tools_menu,
            &help_menu,
        ],
    )?;

    Ok(menu)
}

/// Handles menu events
pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let window = app.get_webview_window("main").unwrap();

    match event.id.as_ref() {
        // File menu
        "new_vm" => {
            let _ = window.emit("menu-new-vm", ());
        }
        "import_vm" => {
            let _ = window.emit("menu-import-vm", ());
        }
        "new_connection" => {
            let _ = window.emit("menu-new-connection", ());
        }
        "close_connection" => {
            let _ = window.emit("menu-close-connection", ());
        }
        "preferences" => {
            let _ = window.emit("menu-preferences", ());
        }
        "quit" => {
            app.exit(0);
        }

        // Edit menu
        "vm_details" => {
            let _ = window.emit("menu-vm-details", ());
        }
        "clone_vm" => {
            let _ = window.emit("menu-clone-vm", ());
        }
        "rename_vm" => {
            let _ = window.emit("menu-rename-vm", ());
        }
        "delete_vm" => {
            let _ = window.emit("menu-delete-vm", ());
        }
        "take_snapshot" => {
            let _ = window.emit("menu-take-snapshot", ());
        }
        "manage_snapshots" => {
            let _ = window.emit("menu-manage-snapshots", ());
        }

        // View menu
        "refresh" => {
            let _ = window.emit("menu-refresh", ());
        }
        "toggle_toolbar" => {
            let _ = window.emit("menu-toggle-toolbar", ());
        }
        "toggle_statusbar" => {
            let _ = window.emit("menu-toggle-statusbar", ());
        }

        // Actions menu
        "start_vm" => {
            let _ = window.emit("menu-start-vm", ());
        }
        "stop_vm" => {
            let _ = window.emit("menu-stop-vm", ());
        }
        "force_stop_vm" => {
            let _ = window.emit("menu-force-stop-vm", ());
        }
        "pause_vm" => {
            let _ = window.emit("menu-pause-vm", ());
        }
        "resume_vm" => {
            let _ = window.emit("menu-resume-vm", ());
        }
        "reboot_vm" => {
            let _ = window.emit("menu-reboot-vm", ());
        }
        "open_console" => {
            let _ = window.emit("menu-open-console", ());
        }

        // Tools menu
        "storage_manager" => {
            let _ = window.emit("menu-storage-manager", ());
        }
        "network_manager" => {
            let _ = window.emit("menu-network-manager", ());
        }
        "templates" => {
            let _ = window.emit("menu-templates", ());
        }
        "schedules" => {
            let _ = window.emit("menu-schedules", ());
        }
        "alerts" => {
            let _ = window.emit("menu-alerts", ());
        }
        "backups" => {
            let _ = window.emit("menu-backups", ());
        }
        "performance" => {
            let _ = window.emit("menu-performance", ());
        }
        "optimization" => {
            let _ = window.emit("menu-optimization", ());
        }

        // Help menu
        "documentation" => {
            let _ = window.emit("menu-documentation", ());
        }
        "keyboard_shortcuts" => {
            let _ = window.emit("menu-keyboard-shortcuts", ());
        }
        "check_updates" => {
            let _ = window.emit("menu-check-updates", ());
        }
        "report_issue" => {
            let _ = window.emit("menu-report-issue", ());
        }
        "about" => {
            let _ = window.emit("menu-about", ());
        }

        _ => {}
    }
}
