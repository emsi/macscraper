use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

/// Top-level application configuration persisted as TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub scraping: ScrapingConfig,
    pub style: Option<StyleConfig>,
    pub prompt_templates: Vec<PromptTemplate>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            scraping: ScrapingConfig::default(),
            style: None,
            prompt_templates: vec![
                PromptTemplate {
                    name: "Concise teaser".into(),
                    default: true,
                    prompt: "Write a 2-sentence social media teaser for the following article.\nBe engaging and avoid clickbait.\n\nTitle: {{title}}\nArticle: {{article_text}}".into(),
                },
                PromptTemplate {
                    name: "LinkedIn post".into(),
                    default: false,
                    prompt: "Write a professional 3-sentence LinkedIn post summary.\nFocus on the key insight or takeaway.\n\nTitle: {{title}}\nArticle: {{article_text}}".into(),
                },
                PromptTemplate {
                    name: "Twitter / X opener".into(),
                    default: false,
                    prompt: "Write a punchy opener tweet (max 280 characters) for this article.\nNo hashtags.\n\nTitle: {{title}}".into(),
                },
            ],
        }
    }
}

/// LLM endpoint and model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub endpoint: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
        }
    }
}

/// Web scraping thresholds and limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingConfig {
    /// Minimum character count from semantic selectors before falling back to
    /// density-scoring heuristic.
    pub article_min_chars: usize,
    /// Minimum image width (px) to qualify as the "first large image" OG fallback.
    pub image_min_width: u32,
    /// Max number of external CSS files fetched per page for font detection.
    pub max_stylesheets: usize,
}

impl Default for ScrapingConfig {
    fn default() -> Self {
        Self { article_min_chars: 200, image_min_width: 300, max_stylesheets: 5 }
    }
}

/// Optional style overrides for card rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub title_family: String,
    pub title_size: u32,
    pub body_family: String,
    pub body_size: u32,
}

/// A named LLM prompt template with a mustache-style placeholder convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    #[serde(default)]
    pub default: bool,
    pub prompt: String,
}

/// Returns the path to `config.toml` inside the app config directory,
/// creating the directory if needed.
///
/// :param app: Tauri app handle used to resolve the platform config dir.
/// :return: Absolute path to the config file, or an error string.
fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("config.toml");
    Ok(path)
}

/// Loads the application config from disk, returning defaults if no file exists.
///
/// :param app: Tauri app handle.
/// :return: Parsed `AppConfig` or an error string.
#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let path = config_path(&app)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&text).map_err(|e| e.to_string())
}

/// Persists the application config to disk as pretty-printed TOML.
///
/// :param app: Tauri app handle.
/// :param config: Config to serialize and write.
/// :return: Ok on success, or an error string.
#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_path(&app)?;
    let text = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

/// Retrieves the stored API key from the OS keychain.
///
/// :return: `Some(key)` if set, `None` if no entry exists, or an error string.
#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new("macscraper", "api_key").map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Stores or clears the API key in the OS keychain.
///
/// Passing an empty string deletes the existing credential.
///
/// :param key: API key to store, or empty string to remove it.
/// :return: Ok on success, or an error string.
#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), String> {
    let entry = keyring::Entry::new("macscraper", "api_key").map_err(|e| e.to_string())?;
    if key.is_empty() {
        let _ = entry.delete_password();
        Ok(())
    } else {
        entry.set_password(&key).map_err(|e| e.to_string())
    }
}

/// Returns the resolved absolute path to the config file.
///
/// :param app: Tauri app handle.
/// :return: Path string, or an error.
#[tauri::command]
pub fn get_config_path(app: AppHandle) -> Result<String, String> {
    config_path(&app).map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.llm.endpoint, "https://api.openai.com/v1");
        assert_eq!(cfg.llm.model, "gpt-4o-mini");
        assert_eq!(cfg.scraping.article_min_chars, 200);
        assert_eq!(cfg.scraping.image_min_width, 300);
        assert_eq!(cfg.scraping.max_stylesheets, 5);
        assert!(cfg.prompt_templates.len() >= 1);
        assert!(cfg.prompt_templates.iter().any(|t| t.default));
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let original = AppConfig::default();
        let toml_str = toml::to_string(&original).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.llm.endpoint, original.llm.endpoint);
        assert_eq!(parsed.scraping.article_min_chars, original.scraping.article_min_chars);
        assert_eq!(parsed.prompt_templates.len(), original.prompt_templates.len());
    }

    #[test]
    fn test_config_from_toml_string() {
        let toml_str = r#"
[llm]
endpoint = "http://localhost:11434/v1"
model = "llama3"

[scraping]
article_min_chars = 300
image_min_width = 400
max_stylesheets = 3

[[prompt_templates]]
name = "My template"
default = true
prompt = "Summarise: {{article_text}}"
"#;
        let cfg: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.llm.model, "llama3");
        assert_eq!(cfg.scraping.article_min_chars, 300);
        assert_eq!(cfg.prompt_templates[0].name, "My template");
    }
}
