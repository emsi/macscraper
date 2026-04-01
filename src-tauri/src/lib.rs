/// Entry point for the Tauri application.
///
/// Initializes the Tauri builder with default plugins, registers command handlers,
/// and starts the application event loop.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
