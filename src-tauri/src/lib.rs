mod config;
mod fonts;
mod image;
mod llm;
mod scraper;
mod store;
mod types;

pub use types::*;

/// Entry point for the Tauri application.
///
/// Initializes the Tauri builder with default plugins, registers command handlers,
/// and starts the application event loop.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            config::load_config,
            config::save_config,
            config::get_api_key,
            config::set_api_key,
            config::get_config_path,
            store::get_last_url,
            store::save_last_url,
            image::fetch_image,
            scraper::scrape_url,
            llm::generate_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
