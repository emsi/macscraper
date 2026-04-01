use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Convert raw bytes to a base64 data URL for use in HTML Canvas drawImage().
///
/// :param bytes: Raw image bytes to encode.
/// :param mime: MIME type string (e.g. "image/png").
/// :return: A `data:<mime>;base64,<encoded>` string.
pub fn bytes_to_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, STANDARD.encode(bytes))
}

/// Extract the MIME type from a Content-Type header value, stripping parameters.
///
/// :param content_type: Raw Content-Type header string (e.g. "image/png; charset=utf-8").
/// :return: The bare MIME type, defaulting to "image/png" if empty.
pub fn mime_from_content_type(content_type: &str) -> &str {
    let mime = content_type.split(';').next().unwrap_or("").trim();
    if mime.is_empty() { "image/png" } else { mime }
}

/// Fetch an image from a URL and return it as a base64 data URL.
///
/// Bypasses CORS — the Rust backend fetches the image, Canvas receives a data URL
/// and stays untainted so toBlob() works.
///
/// :param url: The image URL to fetch.
/// :return: A `data:<mime>;base64,<encoded>` string, or an error message.
#[tauri::command]
pub async fn fetch_image(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("macscraper/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let mime = mime_from_content_type(&content_type).to_string();
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes_to_data_url(&bytes, &mime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_data_url_png() {
        let png_bytes = vec![0x89, 0x50, 0x4E, 0x47];
        let result = bytes_to_data_url(&png_bytes, "image/png");
        assert!(result.starts_with("data:image/png;base64,"));
        assert!(result.len() > 22);
    }

    #[test]
    fn test_bytes_to_data_url_jpeg() {
        let jpeg_bytes = vec![0xFF, 0xD8, 0xFF];
        let result = bytes_to_data_url(&jpeg_bytes, "image/jpeg");
        assert!(result.starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn test_mime_from_content_type() {
        assert_eq!(mime_from_content_type("image/png; charset=utf-8"), "image/png");
        assert_eq!(mime_from_content_type("image/jpeg"), "image/jpeg");
        assert_eq!(mime_from_content_type(""), "image/png");
    }
}
