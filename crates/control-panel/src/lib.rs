pub mod binaries;
pub mod claude_config;
pub mod commands;
pub mod installer;
pub mod registry;

/// Build and run the Tauri application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_connectors,
            commands::list_installed,
            commands::install_connector,
            commands::test_connection,
            commands::uninstall_connector,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Tauri app");
}
