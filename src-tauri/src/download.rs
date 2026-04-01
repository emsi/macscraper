use base64::Engine;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Save a PNG image to a user-chosen file via native save dialog.
///
/// Decodes base64-encoded PNG bytes, presents a native file-save dialog,
/// and writes the bytes to the chosen path. Returns `true` if saved,
/// `false` if the user cancelled.
///
/// :param app: Tauri application handle.
/// :param data: Base64-encoded PNG bytes (e.g. from canvas.toDataURL()).
/// :param suggested_name: Default filename shown in the save dialog.
/// :return: true if the file was saved, false if cancelled.
#[tauri::command]
pub async fn save_png(
    app: AppHandle,
    data: String,
    suggested_name: String,
) -> Result<bool, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("Failed to decode image data: {e}"))?;

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<std::path::PathBuf>>();

    app.dialog()
        .file()
        .set_file_name(&suggested_name)
        .add_filter("PNG Image", &["png"])
        .save_file(move |path| {
            let converted = path.and_then(|fp| fp.into_path().ok());
            let _ = tx.send(converted);
        });

    match rx.await.map_err(|e| e.to_string())? {
        Some(path) => {
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false),
    }
}
