use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "state.json";
const KEY_LAST_URL: &str = "last_url";
const KEY_SPLIT_RATIO: &str = "split_ratio";
const KEY_ACTIVE_THEME: &str = "active_theme";

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

/// Get the saved editor panel width percentage.
#[tauri::command]
pub fn get_split_ratio(app: AppHandle) -> Result<Option<f64>, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    Ok(store.get(KEY_SPLIT_RATIO).and_then(|v| v.as_f64()))
}

/// Save the editor panel width percentage and flush to disk.
#[tauri::command]
pub fn save_split_ratio(app: AppHandle, ratio: f64) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_SPLIT_RATIO, ratio);
    store.save().map_err(|e| e.to_string())
}

/// Get the last active preview theme ("light" or "dark").
#[tauri::command]
pub fn get_active_theme(app: AppHandle) -> Result<Option<String>, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    Ok(store.get(KEY_ACTIVE_THEME).and_then(|v| v.as_str().map(str::to_string)))
}

/// Save the active preview theme and flush to disk.
#[tauri::command]
pub fn save_active_theme(app: AppHandle, theme: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_ACTIVE_THEME, theme);
    store.save().map_err(|e| e.to_string())
}
