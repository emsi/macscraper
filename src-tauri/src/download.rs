use base64::Engine;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use image::GenericImageView;

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

/// Copy a PNG image to the system clipboard via the native clipboard API.
///
/// WebKitGTK blocks navigator.clipboard.write() for images, so this
/// command decodes the PNG, extracts RGBA pixels, and writes them via arboard.
///
/// :param data: Base64-encoded PNG bytes.
/// :return: Ok(()) on success.
#[tauri::command]
pub async fn copy_png_to_clipboard(data: String) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("base64 decode: {e}"))?;

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .map_err(|e| format!("PNG decode: {e}"))?
            .into_rgba8();

        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard
            .set_image(arboard::ImageData {
                width: width as usize,
                height: height as usize,
                bytes: std::borrow::Cow::Owned(pixels),
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
