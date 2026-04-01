use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "state.json";
const KEY_LAST_URL: &str = "last_url";

/// Get the last used URL from persistent KV store.
///
/// Returns `Ok(Some(url))` if a URL has been saved, `Ok(None)` otherwise.
#[tauri::command]
pub fn get_last_url(app: AppHandle) -> Result<Option<String>, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    Ok(store.get(KEY_LAST_URL).and_then(|v| v.as_str().map(str::to_string)))
}

/// Save the last used URL to persistent KV store.
///
/// Persists `url` under the `last_url` key and flushes to disk immediately.
#[tauri::command]
pub fn save_last_url(app: AppHandle, url: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_LAST_URL, url);
    store.save().map_err(|e| e.to_string())
}
