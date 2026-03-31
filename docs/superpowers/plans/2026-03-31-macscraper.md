# macscraper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri 2 desktop app that scrapes blog post metadata, optionally generates AI summaries, and produces downloadable white and dark social media card images.

**Architecture:** Rust backend handles all I/O via Tauri commands (`scrape_url`, `fetch_image`, `generate_summary`, `load_config`, `save_config`, `get_last_url`, `save_last_url`, `get_api_key`, `set_api_key`). A Svelte frontend manages UI state through writable stores and renders social cards using HTML Canvas 2D. Template variable substitution happens in Svelte before calling the LLM command.

**Tech Stack:** Tauri 2, Rust (reqwest 0.13, scraper 0.26, async-openai 0.31, tokio 1, serde 1, toml 0.8, keyring 2, base64 0.22), tauri-plugin-store 2, Svelte 4 + Vite, TypeScript, HTML Canvas 2D, Vitest

**Spec:** `docs/superpowers/specs/2026-03-31-macscraper-design.md`

---

## Prerequisites (verify before starting)

```bash
rustup --version        # Rust stable toolchain required
node --version          # Node.js 18+ required
cargo tauri --version   # install: cargo install tauri-cli --locked
# Linux only:
sudo apt install libsecret-1-dev pkg-config libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev patchelf
```

---

## File Map

### Rust (`src-tauri/src/`)
| File | Responsibility |
|------|----------------|
| `main.rs` | Tauri builder, plugin registration, all command registrations |
| `types.rs` | `ScrapedData`, `ImageMeta`, `DetectedFonts` — shared data types |
| `config.rs` | `AppConfig` + TOML (de)serialisation + `load_config`/`save_config`/`get_api_key`/`set_api_key` commands |
| `scraper.rs` | `scrape_url` command: HTTP fetch, metadata waterfall, image collection, article text two-pass |
| `fonts.rs` | Font detection: stylesheets, Google Fonts URL, CSS parsing, inline `style=""` fallback |
| `image.rs` | `fetch_image` command: HTTP fetch → base64 data URL |
| `llm.rs` | `generate_summary` command: async-openai with custom base URL |
| `store.rs` | `get_last_url`/`save_last_url` commands: tauri-plugin-store KV |

### TypeScript/Svelte (`src/`)
| File | Responsibility |
|------|----------------|
| `lib/types.ts` | TypeScript mirrors of all Rust types |
| `lib/template.ts` | `substituteTemplate()` — `{{var}}` replacement |
| `lib/fonts.ts` | Google Fonts `<link>` injection + font picker list builder |
| `lib/canvas/renderer.ts` | Pure canvas functions: `wrapText`, `measureTextHeight`, `renderCard` |
| `lib/stores/scrape.ts` | `scraped` store: `ScrapedData | null` + `scraping: boolean` + `error: string | null` |
| `lib/stores/editor.ts` | `editor` store: title, description, articleText, selectedImage, fontOverrides, preset, autoHeight, showAttribution |
| `lib/stores/config.ts` | `appConfig` store, loaded from `load_config` on startup |
| `lib/components/UrlBar.svelte` | URL input + Scrape button + loading indicator |
| `lib/components/ImagePicker.svelte` | Scrollable thumbnail row + "+disk" tile |
| `lib/components/ContentSection.svelte` | Image picker + title + description + collapsible full-text |
| `lib/components/SummarySection.svelte` | Scraped/AI toggle, template picker, prompt editor, inline save-as-template |
| `lib/components/StyleSection.svelte` | Font pickers + size inputs + preset dropdown + auto-height + attribution |
| `lib/components/SettingsModal.svelte` | LLM config, API key, open-config-file |
| `lib/components/CardCanvas.svelte` | Single canvas, themed (light/dark), reactive render |
| `lib/components/PreviewPanel.svelte` | Hosts both `CardCanvas` + download buttons |
| `App.svelte` | Root: split-panel layout, settings gear, initialisation |
| `main.ts` | Svelte entry point |

---

## — PHASE 1: PROJECT SCAFFOLD —

### Task 1: Scaffold Tauri 2 + Svelte + Vite project

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`
- Create: `src/main.ts`, `src/App.svelte`, `src/app.css`, `index.html`
- Modify: `.gitignore`

- [ ] **Step 1: Scaffold project into current directory**

```bash
cd /home/emsi/git/macscraper
cargo tauri init --app-name macscraper --window-title macscraper \
  --dist-dir ../dist --dev-url http://localhost:1420 \
  --before-dev-command "npm run dev" --before-build-command "npm run build"
npm create vite@latest . -- --template svelte-ts --force
npm install
```

- [ ] **Step 2: Replace `src-tauri/Cargo.toml` with full dependency list**

```toml
[package]
name = "macscraper"
version = "0.1.0"
edition = "2021"

[lib]
name = "macscraper_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri              = { version = "2", features = [] }
tauri-plugin-store = "2"
serde              = { version = "1", features = ["derive"] }
serde_json         = "1"
tokio              = { version = "1", features = ["full"] }
reqwest            = { version = "0.13", features = ["json", "rustls-tls"], default-features = false }
scraper            = "0.26"
async-openai       = { version = "0.31", default-features = false, features = ["chat-completion"] }
toml               = "0.8"
base64             = { version = "0.22", features = ["std"] }
keyring            = "2"
```

- [ ] **Step 3: Add Vitest + testing library to package.json**

```bash
npm install --save-dev vitest jsdom @testing-library/svelte @testing-library/jest-dom
```

Add to `vite.config.ts`:
```typescript
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'jsdom',
    globals: true,
  },
})
```

- [ ] **Step 4: Add `tauri-plugin-store` to Tauri builder in `src-tauri/src/main.rs`**

```rust
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    macscraper_lib::run();
}
```

Create `src-tauri/src/lib.rs`:
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Update `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "store:allow-get",
    "store:allow-set",
    "store:allow-save",
    "store:allow-load"
  ]
}
```

- [ ] **Step 6: Verify the project builds**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
npm run build
```

Expected: both commands exit 0 with no errors.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2 + Svelte + Vite project"
```

---

## — PHASE 2: RUST BACKEND —

### Task 2: Shared Rust types

**Files:**
- Create: `src-tauri/src/types.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the types module**

```rust
// src-tauri/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedData {
    pub title: String,
    pub description: String,
    pub og_image: Option<String>,
    pub all_images: Vec<ImageMeta>,
    pub article_text: String,
    pub detected_fonts: DetectedFonts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMeta {
    pub src: String,
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectedFonts {
    pub heading_family: Option<String>,
    pub body_family: Option<String>,
    pub google_fonts_url: Option<String>,
}
```

- [ ] **Step 2: Declare module in `lib.rs`**

Add `mod types;` (and `pub use types::*;`) to `src-tauri/src/lib.rs`.

- [ ] **Step 3: Verify compilation**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "^error"
```

Expected: no output (no errors).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/types.rs src-tauri/src/lib.rs
git commit -m "feat: add shared Rust data types"
```

---

### Task 3: Config system

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing tests for config round-trip**

Add at bottom of `src-tauri/src/config.rs` (create the file with tests first):

```rust
// src-tauri/src/config.rs
use serde::{Deserialize, Serialize};

// ── structs (to be filled in Step 2) ──
// pub struct AppConfig { ... }

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
```

- [ ] **Step 2: Run tests — expect compile failure**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | head -20
```

Expected: `error[E0412]: cannot find type AppConfig`

- [ ] **Step 3: Implement all config structs**

Replace the stub comment in `config.rs` with:

```rust
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub title_family: String,
    pub title_size: u32,
    pub body_family: String,
    pub body_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    #[serde(default)]
    pub default: bool,
    pub prompt: String,
}
```

- [ ] **Step 4: Implement Tauri commands**

Add below the structs in `config.rs`:

```rust
use std::path::PathBuf;
use tauri::AppHandle;

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("config.toml");
    Ok(path)
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let path = config_path(&app)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_path(&app)?;
    let text = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new("macscraper", "api_key").map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), String> {
    let entry = keyring::Entry::new("macscraper", "api_key").map_err(|e| e.to_string())?;
    if key.is_empty() {
        let _ = entry.delete_credential();
        Ok(())
    } else {
        entry.set_password(&key).map_err(|e| e.to_string())
    }
}
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml config 2>&1 | tail -10
```

Expected:
```
test config::tests::test_config_default_values ... ok
test config::tests::test_config_toml_roundtrip ... ok
test config::tests::test_config_from_toml_string ... ok
test result: ok. 3 passed; 0 failed
```

- [ ] **Step 6: Register module and commands in `lib.rs`**

```rust
// src-tauri/src/lib.rs
mod config;
mod types;

pub use types::*;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            config::load_config,
            config::save_config,
            config::get_api_key,
            config::set_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: config system with TOML persistence and OS keychain for API key"
```

---

### Task 4: KV store (last URL persistence)

**Files:**
- Create: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write `store.rs`**

```rust
// src-tauri/src/store.rs
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "state.json";
const KEY_LAST_URL: &str = "last_url";

#[tauri::command]
pub fn get_last_url(app: AppHandle) -> Result<Option<String>, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    Ok(store.get(KEY_LAST_URL).and_then(|v| v.as_str().map(str::to_string)))
}

#[tauri::command]
pub fn save_last_url(app: AppHandle, url: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_LAST_URL, serde_json::Value::String(url));
    store.save().map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Register in `lib.rs`**

Add `mod store;` and add `store::get_last_url`, `store::save_last_url` to `generate_handler!`.

- [ ] **Step 3: Verify compilation**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error"
```

Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/store.rs src-tauri/src/lib.rs
git commit -m "feat: KV store for last-used URL persistence"
```

---

### Task 5: Image fetcher (CORS bypass)

**Files:**
- Create: `src-tauri/src/image.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/image.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_data_url_png() {
        // Minimal valid PNG header bytes
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
```

- [ ] **Step 2: Run test — expect compile failure**

```bash
cargo test --manifest-path src-tauri/Cargo.toml image 2>&1 | head -10
```

Expected: `error[E0425]: cannot find function bytes_to_data_url`

- [ ] **Step 3: Implement**

```rust
// src-tauri/src/image.rs
use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Convert raw bytes to a base64 data URL for use in HTML Canvas drawImage().
pub fn bytes_to_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, STANDARD.encode(bytes))
}

/// Extract the MIME type from a Content-Type header value, stripping parameters.
pub fn mime_from_content_type(content_type: &str) -> &str {
    let mime = content_type.split(';').next().unwrap_or("").trim();
    if mime.is_empty() { "image/png" } else { mime }
}

/// Fetch an image from a URL and return it as a base64 data URL.
/// This bypasses CORS — the Rust backend fetches, Canvas receives a data URL
/// and stays untainted so toBlob() works.
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
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml image 2>&1 | tail -8
```

Expected:
```
test image::tests::test_bytes_to_data_url_png ... ok
test image::tests::test_bytes_to_data_url_jpeg ... ok
test image::tests::test_mime_from_content_type ... ok
test result: ok. 3 passed; 0 failed
```

- [ ] **Step 5: Register in `lib.rs`** — add `mod image;` and `image::fetch_image` to handler.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/image.rs src-tauri/src/lib.rs
git commit -m "feat: fetch_image command for CORS-safe canvas image loading"
```

---

### Task 6: Font detection

**Files:**
- Create: `src-tauri/src/fonts.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src-tauri/src/fonts.rs

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_google_fonts_url_detected() {
        let urls = vec![
            "https://fonts.googleapis.com/css2?family=Playfair+Display&display=swap".to_string(),
            "https://example.com/style.css".to_string(),
        ];
        assert_eq!(
            find_google_fonts_url(&urls),
            Some("https://fonts.googleapis.com/css2?family=Playfair+Display&display=swap".to_string())
        );
    }

    #[test]
    fn test_no_google_fonts() {
        let urls = vec!["https://example.com/style.css".to_string()];
        assert_eq!(find_google_fonts_url(&urls), None);
    }

    #[test]
    fn test_extract_heading_font_from_css() {
        let css = "h1 { font-family: 'Playfair Display', serif; color: red; }";
        assert_eq!(
            extract_font_for_selectors(css, &["h1", "h2"]),
            Some("Playfair Display".to_string())
        );
    }

    #[test]
    fn test_extract_body_font_from_css() {
        let css = "body { font-family: \"Source Sans Pro\", sans-serif; }";
        assert_eq!(
            extract_font_for_selectors(css, &["body", "p"]),
            Some("Source Sans Pro".to_string())
        );
    }

    #[test]
    fn test_no_font_in_css() {
        let css = "h1 { color: red; }";
        assert_eq!(extract_font_for_selectors(css, &["h1"]), None);
    }

    #[test]
    fn test_inline_style_font_detection() {
        let html = r#"<html><body><h1 style="font-family: 'Georgia', serif;">Title</h1></body></html>"#;
        let doc = Html::parse_document(html);
        assert_eq!(
            extract_inline_style_font(&doc, &["h1", "h2"]),
            Some("Georgia".to_string())
        );
    }

    #[test]
    fn test_collect_stylesheet_urls() {
        let html = r#"<html><head>
          <link rel="stylesheet" href="/style.css">
          <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter">
          <link rel="icon" href="/favicon.ico">
        </head></html>"#;
        let doc = Html::parse_document(html);
        let urls = collect_stylesheet_urls(&doc, "https://example.com");
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().any(|u| u.contains("googleapis.com")));
        assert!(urls.iter().any(|u| u.ends_with("/style.css")));
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test --manifest-path src-tauri/Cargo.toml fonts 2>&1 | head -10
```

- [ ] **Step 3: Implement**

```rust
// src-tauri/src/fonts.rs
use scraper::{Html, Selector};

/// Collect all stylesheet href values from <link rel="stylesheet"> tags,
/// resolved to absolute URLs using the page's base URL.
pub fn collect_stylesheet_urls(doc: &Html, base_url: &str) -> Vec<String> {
    let sel = Selector::parse(r#"link[rel="stylesheet"]"#).unwrap();
    let base = base_url.trim_end_matches('/');
    doc.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .map(|href| {
            if href.starts_with("http://") || href.starts_with("https://") {
                href.to_string()
            } else if href.starts_with('/') {
                // absolute path — prepend origin
                let origin = base.splitn(4, '/').take(3).collect::<Vec<_>>().join("/");
                format!("{}{}", origin, href)
            } else {
                format!("{}/{}", base, href)
            }
        })
        .collect()
}

/// Return the first Google Fonts URL found in a list of stylesheet URLs.
pub fn find_google_fonts_url(urls: &[String]) -> Option<String> {
    urls.iter().find(|u| u.contains("fonts.googleapis.com")).cloned()
}

/// Scan a CSS string for a font-family rule targeting any of the given selectors.
/// Returns the first font name found (strips quotes).
pub fn extract_font_for_selectors(css: &str, selectors: &[&str]) -> Option<String> {
    for selector in selectors {
        if let Some(font) = find_font_after_selector(css, selector) {
            return Some(font);
        }
    }
    None
}

fn find_font_after_selector(css: &str, selector: &str) -> Option<String> {
    let pos = css.find(selector)?;
    let after = &css[pos..];
    let brace_start = after.find('{')? + 1;
    let brace_end = after.find('}')?;
    if brace_end < brace_start {
        return None;
    }
    let body = &after[brace_start..brace_end];
    let fp = body.find("font-family")?;
    let colon = body[fp..].find(':')? + fp + 1;
    let value = body[colon..].trim();
    let end = value.find([';', '}']).unwrap_or(value.len());
    let raw = value[..end].trim();
    // Take first font in the stack, strip quotes
    let first = raw.split(',').next().unwrap_or(raw).trim();
    let clean = first.trim_matches('"').trim_matches('\'').trim().to_string();
    if clean.is_empty() { None } else { Some(clean) }
}

/// Check style="" attributes on heading/body elements directly.
pub fn extract_inline_style_font(doc: &Html, element_selectors: &[&str]) -> Option<String> {
    for sel_str in element_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                if let Some(style) = el.value().attr("style") {
                    if let Some(font) = extract_font_from_style_attr(style) {
                        return Some(font);
                    }
                }
            }
        }
    }
    None
}

fn extract_font_from_style_attr(style: &str) -> Option<String> {
    let fp = style.find("font-family")?;
    let colon = style[fp..].find(':')? + fp + 1;
    let value = style[colon..].trim();
    let end = value.find(';').unwrap_or(value.len());
    let first = value[..end].split(',').next().unwrap_or("").trim();
    let clean = first.trim_matches('"').trim_matches('\'').trim().to_string();
    if clean.is_empty() { None } else { Some(clean) }
}

/// Perform full font detection for a document. Fetches up to `max_stylesheets`
/// external (non-Google) stylesheets and inspects inline <style> blocks.
/// Falls back to inspecting inline style="" attributes on heading/body elements.
pub async fn detect_fonts(
    doc: &Html,
    base_url: &str,
    max_stylesheets: usize,
) -> crate::types::DetectedFonts {
    let urls = collect_stylesheet_urls(doc, base_url);
    let google_fonts_url = find_google_fonts_url(&urls);

    // Collect CSS text: inline <style> blocks + external non-Google sheets
    let inline_css: String = {
        let sel = Selector::parse("style").unwrap();
        doc.select(&sel).map(|el| el.text().collect::<String>()).collect::<Vec<_>>().join("\n")
    };

    let client = reqwest::Client::builder().user_agent("macscraper/0.1").build().ok();
    let mut external_css = String::new();
    if let Some(client) = client {
        let external_urls: Vec<_> = urls.iter()
            .filter(|u| !u.contains("fonts.googleapis.com"))
            .take(max_stylesheets)
            .collect();
        for url in external_urls {
            if let Ok(resp) = client.get(url).send().await {
                if let Ok(text) = resp.text().await {
                    external_css.push('\n');
                    external_css.push_str(&text);
                }
            }
        }
    }

    let all_css = format!("{}\n{}", inline_css, external_css);

    let heading_family = extract_font_for_selectors(&all_css, &["h1", "h2"])
        .or_else(|| extract_inline_style_font(doc, &["h1", "h2"]));

    let body_family = extract_font_for_selectors(&all_css, &["body", "p"])
        .or_else(|| extract_inline_style_font(doc, &["body"]));

    crate::types::DetectedFonts { heading_family, body_family, google_fonts_url }
}
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml fonts 2>&1 | tail -12
```

Expected: all 7 tests pass.

- [ ] **Step 5: Add `mod fonts;` to `lib.rs`. No command to register — fonts is called from `scraper.rs`.**

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/fonts.rs src-tauri/src/lib.rs
git commit -m "feat: font detection from stylesheets and inline styles"
```

---

### Task 7: Metadata scraper

**Files:**
- Create: `src-tauri/src/scraper.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src-tauri/src/scraper.rs (tests section)

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn parse(html: &str) -> Html { Html::parse_document(html) }

    #[test]
    fn test_title_og_priority() {
        let doc = parse(r#"<html><head>
          <meta property="og:title" content="OG Title">
          <title>Page Title</title>
        </head><body><h1>H1 Title</h1></body></html>"#);
        assert_eq!(extract_title(&doc), "OG Title");
    }

    #[test]
    fn test_title_falls_back_to_title_tag() {
        let doc = parse("<html><head><title>Page Title</title></head></html>");
        assert_eq!(extract_title(&doc), "Page Title");
    }

    #[test]
    fn test_title_falls_back_to_h1() {
        let doc = parse("<html><body><h1>H1 Title</h1></body></html>");
        assert_eq!(extract_title(&doc), "H1 Title");
    }

    #[test]
    fn test_description_og_priority() {
        let doc = parse(r#"<html><head>
          <meta property="og:description" content="OG Desc">
          <meta name="description" content="Meta Desc">
        </head></html>"#);
        assert_eq!(extract_description(&doc), "OG Desc");
    }

    #[test]
    fn test_description_fallback_to_meta() {
        let doc = parse(r#"<html><head><meta name="description" content="Meta Desc"></head></html>"#);
        assert_eq!(extract_description(&doc), "Meta Desc");
    }

    #[test]
    fn test_og_image_extracted() {
        let doc = parse(r#"<html><head><meta property="og:image" content="https://img.com/a.jpg"></head></html>"#);
        assert_eq!(extract_og_image(&doc), Some("https://img.com/a.jpg".to_string()));
    }

    #[test]
    fn test_all_images_collected_and_resolved() {
        let doc = parse(r#"<html><body>
          <img src="/img/a.jpg" alt="A" width="800" height="600">
          <img src="https://cdn.com/b.png" alt="B">
        </body></html>"#);
        let images = collect_all_images(&doc, "https://example.com", 300);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].src, "https://example.com/img/a.jpg");
        assert_eq!(images[0].width, Some(800));
        assert_eq!(images[1].src, "https://cdn.com/b.png");
    }

    #[test]
    fn test_article_text_semantic_pass() {
        let doc = parse(r#"<html><body>
          <nav>Nav stuff</nav>
          <article>This is the real article content with enough text to pass the threshold check here.</article>
        </body></html>"#);
        let text = extract_article_text(&doc, 50);
        assert!(text.contains("real article content"));
        assert!(!text.contains("Nav stuff"));
    }

    #[test]
    fn test_article_text_density_fallback() {
        // No semantic tags — should use density fallback
        let doc = parse(r#"<html><body>
          <div class="sidebar"><a>link</a><a>link</a><a>link</a></div>
          <div class="content"><p>This is a long paragraph with substantial text content that should win the density score because it has a high text to tag ratio and provides the actual article body.</p></div>
        </body></html>"#);
        let text = extract_article_text(&doc, 200);
        assert!(text.contains("high text to tag ratio"));
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test --manifest-path src-tauri/Cargo.toml scraper 2>&1 | head -10
```

- [ ] **Step 3: Implement extraction functions**

```rust
// src-tauri/src/scraper.rs
use scraper::{Html, Selector};
use crate::types::{ImageMeta, ScrapedData};

fn meta_property(doc: &Html, property: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[property='{}']", property)).ok()?;
    doc.select(&sel).next()?.value().attr("content").map(str::to_string)
}

fn meta_name(doc: &Html, name: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[name='{}']", name)).ok()?;
    doc.select(&sel).next()?.value().attr("content").map(str::to_string)
}

pub fn extract_title(doc: &Html) -> String {
    meta_property(doc, "og:title")
        .or_else(|| meta_name(doc, "twitter:title"))
        .or_else(|| {
            let sel = Selector::parse("title").ok()?;
            doc.select(&sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            let sel = Selector::parse("h1").ok()?;
            doc.select(&sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_default()
}

pub fn extract_description(doc: &Html) -> String {
    meta_property(doc, "og:description")
        .or_else(|| meta_name(doc, "twitter:description"))
        .or_else(|| meta_name(doc, "description"))
        .unwrap_or_default()
}

pub fn extract_og_image(doc: &Html) -> Option<String> {
    meta_property(doc, "og:image").or_else(|| meta_name(doc, "twitter:image"))
}

fn resolve_url(href: &str, base: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    let origin = base.splitn(4, '/').take(3).collect::<Vec<_>>().join("/");
    if href.starts_with('/') {
        format!("{}{}", origin, href)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), href)
    }
}

pub fn collect_all_images(doc: &Html, base_url: &str, min_width: u32) -> Vec<ImageMeta> {
    let sel = Selector::parse("img").unwrap();
    let mut images: Vec<ImageMeta> = doc.select(&sel)
        .filter_map(|el| {
            let src = el.value().attr("src")?;
            Some(ImageMeta {
                src: resolve_url(src, base_url),
                alt: el.value().attr("alt").unwrap_or("").to_string(),
                width: el.value().attr("width").and_then(|w| w.parse().ok()),
                height: el.value().attr("height").and_then(|h| h.parse().ok()),
            })
        })
        .collect();
    // Move OG image to front if present, and ensure large images appear early
    images.sort_by_key(|img| {
        let w = img.width.unwrap_or(0);
        if w >= min_width { 0u8 } else { 1u8 }
    });
    images
}

const SEMANTIC_SELECTORS: &[&str] = &[
    "article", "main", "[role=main]", ".post-content", ".entry-content",
];

pub fn extract_article_text(doc: &Html, min_chars: usize) -> String {
    // Pass 1: semantic selectors — take longest match
    let best_semantic = SEMANTIC_SELECTORS.iter()
        .filter_map(|sel_str| Selector::parse(sel_str).ok())
        .filter_map(|sel| doc.select(&sel).next())
        .map(|el| {
            el.text().collect::<Vec<_>>().join(" ")
                .split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .max_by_key(|s| s.len())
        .unwrap_or_default();

    if best_semantic.len() >= min_chars {
        return best_semantic;
    }

    // Pass 2: density fallback — score block elements by text/descendant ratio
    let block_sel = Selector::parse("p, div, section").unwrap();
    doc.select(&block_sel)
        .map(|el| {
            let text = el.text().collect::<Vec<_>>().join(" ")
                .split_whitespace().collect::<Vec<_>>().join(" ");
            let tag_count = el.descendants().count().max(1);
            let density = text.len() as f64 / tag_count as f64;
            (text, density)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(t, _)| t)
        .unwrap_or_default()
}
```

- [ ] **Step 4: Implement the `scrape_url` Tauri command**

```rust
/// Fetch a blog post URL and extract all metadata, images, article text, and fonts.
#[tauri::command]
pub async fn scrape_url(
    url: String,
    scraping_config: crate::config::ScrapingConfig,
) -> Result<ScrapedData, String> {
    let client = reqwest::Client::builder()
        .user_agent("macscraper/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let html = client.get(&url).send().await
        .map_err(|e| format!("Fetch failed: {}", e))?
        .text().await
        .map_err(|e| format!("Read failed: {}", e))?;

    let doc = Html::parse_document(&html);

    let title = extract_title(&doc);
    let description = extract_description(&doc);
    let og_image = extract_og_image(&doc);
    let all_images = collect_all_images(&doc, &url, scraping_config.image_min_width);
    let article_text = extract_article_text(&doc, scraping_config.article_min_chars);
    let detected_fonts = crate::fonts::detect_fonts(&doc, &url, scraping_config.max_stylesheets).await;

    Ok(ScrapedData { title, description, og_image, all_images, article_text, detected_fonts })
}
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml scraper 2>&1 | tail -15
```

Expected: all 9 tests pass.

- [ ] **Step 6: Register in `lib.rs`** — add `mod scraper;` and `scraper::scrape_url` to handler.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/scraper.rs src-tauri/src/lib.rs
git commit -m "feat: scrape_url command with metadata waterfall, image collection, article text extraction"
```

---

### Task 8: LLM integration

**Files:**
- Create: `src-tauri/src/llm.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test for prompt validation**

```rust
// src-tauri/src/llm.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default_base_url() {
        let cfg = LlmCallConfig {
            endpoint: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            api_key: None,
        };
        // Just verify the struct constructs correctly
        assert!(cfg.endpoint.contains("openai.com"));
    }

    #[test]
    fn test_llm_config_custom_endpoint() {
        let cfg = LlmCallConfig {
            endpoint: "http://localhost:11434/v1".into(),
            model: "llama3".into(),
            api_key: Some("ollama".into()),
        };
        assert!(cfg.endpoint.contains("11434"));
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test --manifest-path src-tauri/Cargo.toml llm 2>&1 | head -5
```

- [ ] **Step 3: Implement**

```rust
// src-tauri/src/llm.rs
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use serde::{Deserialize, Serialize};

/// LLM connection parameters passed from the frontend on each call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallConfig {
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

/// Call the LLM with a fully-resolved prompt (variables already substituted by Svelte).
/// Returns the assistant message text.
#[tauri::command]
pub async fn generate_summary(
    prompt: String,
    config: LlmCallConfig,
) -> Result<String, String> {
    let api_key = config.api_key.unwrap_or_else(|| "no-key".to_string());
    let openai_config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(config.endpoint);
    let client = Client::with_config(openai_config);

    let request = CreateChatCompletionRequestArgs::default()
        .model(config.model)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()
            .map_err(|e| e.to_string())?
            .into()])
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| e.to_string())?;

    response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| "LLM returned empty response".to_string())
}
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml llm 2>&1 | tail -6
```

Expected: 2 tests pass.

- [ ] **Step 5: Register in `lib.rs`** — add `mod llm;` and `llm::generate_summary` to handler.

- [ ] **Step 6: Full backend build check**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error"
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: 0 errors, all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/llm.rs src-tauri/src/lib.rs
git commit -m "feat: generate_summary command via async-openai with custom base URL"
```

---

## — PHASE 3: FRONTEND FOUNDATION —

### Task 9: TypeScript types + template substitution

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/template.ts`

- [ ] **Step 1: Write failing tests for template substitution**

Create `src/lib/template.test.ts`:

```typescript
import { describe, it, expect } from 'vitest'
import { substituteTemplate } from './template'

describe('substituteTemplate', () => {
  it('replaces known variables', () => {
    const result = substituteTemplate('Hello {{title}}!', { title: 'World' })
    expect(result).toBe('Hello World!')
  })

  it('replaces multiple variables', () => {
    const result = substituteTemplate(
      'Title: {{title}}\nText: {{article_text}}',
      { title: 'My Post', article_text: 'Body here' }
    )
    expect(result).toBe('Title: My Post\nText: Body here')
  })

  it('leaves unknown variables as-is', () => {
    const result = substituteTemplate('Hello {{unknown}}!', { title: 'World' })
    expect(result).toBe('Hello {{unknown}}!')
  })

  it('handles empty vars object', () => {
    const result = substituteTemplate('Hello {{title}}!', {})
    expect(result).toBe('Hello {{title}}!')
  })
})
```

- [ ] **Step 2: Run — expect failure**

```bash
npx vitest run src/lib/template.test.ts 2>&1 | tail -5
```

Expected: `Cannot find module './template'`

- [ ] **Step 3: Create `src/lib/types.ts`**

```typescript
// src/lib/types.ts
export interface ScrapedData {
  title: string
  description: string
  og_image: string | null
  all_images: ImageMeta[]
  article_text: string
  detected_fonts: DetectedFonts
}

export interface ImageMeta {
  src: string
  alt: string
  width: number | null
  height: number | null
}

export interface DetectedFonts {
  heading_family: string | null
  body_family: string | null
  google_fonts_url: string | null
}

export interface AppConfig {
  llm: LlmConfig
  scraping: ScrapingConfig
  style?: StyleConfig
  prompt_templates: PromptTemplate[]
}

export interface LlmConfig {
  endpoint: string
  model: string
}

export interface ScrapingConfig {
  article_min_chars: number
  image_min_width: number
  max_stylesheets: number
}

export interface StyleConfig {
  title_family: string
  title_size: number
  body_family: string
  body_size: number
}

export interface PromptTemplate {
  name: string
  default?: boolean
  prompt: string
}

export interface LlmCallConfig {
  endpoint: string
  model: string
  api_key: string | null
}

export const PLATFORM_PRESETS = [
  { name: 'Twitter / X', width: 1200, height: 628 },
  { name: 'Facebook / OG', width: 1200, height: 630 },
  { name: 'LinkedIn', width: 1200, height: 627 },
  { name: 'Instagram Square', width: 1080, height: 1080 },
  { name: 'Instagram Portrait', width: 1080, height: 1350 },
  { name: 'Custom', width: 1200, height: 0 }, // height 0 = auto
] as const

export type PresetName = typeof PLATFORM_PRESETS[number]['name']
```

- [ ] **Step 4: Create `src/lib/template.ts`**

```typescript
// src/lib/template.ts

/**
 * Replace {{varName}} placeholders in a template string.
 * Unknown variables are left as-is (no silent failure).
 */
export function substituteTemplate(
  template: string,
  vars: Record<string, string>
): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, key) => vars[key] ?? match)
}
```

- [ ] **Step 5: Run tests — expect pass**

```bash
npx vitest run src/lib/template.test.ts 2>&1 | tail -8
```

Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib/types.ts src/lib/template.ts src/lib/template.test.ts
git commit -m "feat: TypeScript types and template variable substitution"
```

---

### Task 10: Font utilities

**Files:**
- Create: `src/lib/fonts.ts`
- Create: `src/lib/fonts.test.ts`

- [ ] **Step 1: Write failing tests**

```typescript
// src/lib/fonts.test.ts
import { describe, it, expect } from 'vitest'
import { buildFontList, injectGoogleFonts } from './fonts'

describe('buildFontList', () => {
  it('puts detected fonts first with detected marker', () => {
    const list = buildFontList('Playfair Display', 'Source Sans Pro')
    expect(list[0].value).toBe('Playfair Display')
    expect(list[0].label).toContain('✦')
    expect(list[1].value).toBe('Source Sans Pro')
    expect(list[1].label).toContain('✦')
  })

  it('includes curated Google Fonts after detected', () => {
    const list = buildFontList(null, null)
    const values = list.map(f => f.value)
    expect(values).toContain('Inter')
    expect(values).toContain('Roboto')
    expect(values).toContain('Merriweather')
  })

  it('includes generic fallbacks at the end', () => {
    const list = buildFontList(null, null)
    const last = list[list.length - 1].value
    expect(['serif', 'sans-serif', 'monospace']).toContain(last)
  })

  it('deduplicates when detected font matches curated font', () => {
    const list = buildFontList('Inter', null)
    const interCount = list.filter(f => f.value === 'Inter').length
    expect(interCount).toBe(1)
  })
})
```

- [ ] **Step 2: Run — expect failure**

```bash
npx vitest run src/lib/fonts.test.ts 2>&1 | tail -5
```

- [ ] **Step 3: Implement `src/lib/fonts.ts`**

```typescript
// src/lib/fonts.ts
export interface FontOption {
  value: string
  label: string
}

const CURATED_GOOGLE_FONTS = [
  'Playfair Display', 'Merriweather', 'Lora', 'Roboto', 'Inter',
  'Open Sans', 'Source Sans Pro', 'Nunito', 'Raleway', 'Montserrat',
]

const GENERIC_FALLBACKS = ['serif', 'sans-serif', 'monospace']

/**
 * Build the font picker list. Detected fonts appear first marked "✦ detected",
 * followed by the curated Google Fonts list, then generic fallbacks.
 * Detected fonts that match a curated font are not duplicated.
 */
export function buildFontList(
  headingFamily: string | null,
  bodyFamily: string | null
): FontOption[] {
  const detected = [headingFamily, bodyFamily]
    .filter((f): f is string => f !== null)
    .filter((f, i, arr) => arr.indexOf(f) === i) // deduplicate

  const curated = CURATED_GOOGLE_FONTS.filter(f => !detected.includes(f))
  const generics = GENERIC_FALLBACKS.filter(f => !detected.includes(f))

  return [
    ...detected.map(f => ({ value: f, label: `${f} ✦` })),
    ...curated.map(f => ({ value: f, label: f })),
    ...generics.map(f => ({ value: f, label: f })),
  ]
}

/**
 * Inject a Google Fonts <link> into the document head so Canvas can use
 * the fonts immediately via CSS font-family name.
 */
export function injectGoogleFonts(url: string): void {
  if (document.querySelector(`link[href="${url}"]`)) return
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = url
  document.head.appendChild(link)
}
```

- [ ] **Step 4: Run tests — expect pass**

```bash
npx vitest run src/lib/fonts.test.ts 2>&1 | tail -8
```

Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/fonts.ts src/lib/fonts.test.ts
git commit -m "feat: font picker list builder and Google Fonts injection"
```

---

### Task 11: Canvas renderer

**Files:**
- Create: `src/lib/canvas/renderer.ts`
- Create: `src/lib/canvas/renderer.test.ts`

- [ ] **Step 1: Write failing tests**

```typescript
// src/lib/canvas/renderer.test.ts
import { describe, it, expect, beforeEach } from 'vitest'
import { wrapText, computeCardHeight, LIGHT_COLORS, DARK_COLORS } from './renderer'

// Minimal CanvasRenderingContext2D mock
function makeCtx(avgCharWidth = 10): CanvasRenderingContext2D {
  return {
    measureText: (text: string) => ({ width: text.length * avgCharWidth }),
    font: '',
  } as unknown as CanvasRenderingContext2D
}

describe('wrapText', () => {
  it('returns single line when text fits', () => {
    const ctx = makeCtx(10)
    expect(wrapText(ctx, 'Hello World', 200)).toEqual(['Hello World'])
  })

  it('wraps long text into multiple lines', () => {
    const ctx = makeCtx(10)
    // 'Hello World Test' = 16 chars * 10 = 160 > 100
    const lines = wrapText(ctx, 'Hello World Test', 100)
    expect(lines.length).toBeGreaterThan(1)
  })

  it('handles empty string', () => {
    const ctx = makeCtx(10)
    expect(wrapText(ctx, '', 200)).toEqual([])
  })
})

describe('computeCardHeight', () => {
  it('returns preset height when autoHeight is false and text fits', () => {
    const h = computeCardHeight({
      presetHeight: 628,
      imageHeight: 283,
      titleLines: 1,
      summaryLines: 2,
      titleLineHeight: 60,
      summaryLineHeight: 30,
      padding: 40,
      attributionHeight: 0,
      autoHeight: false,
    })
    expect(h).toBe(628)
  })

  it('returns computed height when autoHeight is true', () => {
    const h = computeCardHeight({
      presetHeight: 628,
      imageHeight: 283,
      titleLines: 2,
      summaryLines: 3,
      titleLineHeight: 60,
      summaryLineHeight: 30,
      padding: 40,
      attributionHeight: 0,
      autoHeight: true,
    })
    // imageHeight + titleLines*titleLH + summaryLines*summaryLH + 3*padding
    expect(h).toBe(283 + 2 * 60 + 3 * 30 + 3 * 40)
  })
})

describe('color themes', () => {
  it('light theme has white background', () => {
    expect(LIGHT_COLORS.background).toBe('#ffffff')
  })

  it('dark theme has dark background', () => {
    expect(DARK_COLORS.background).toBe('#1a1a2e')
  })
})
```

- [ ] **Step 2: Run — expect failure**

```bash
npx vitest run src/lib/canvas/renderer.test.ts 2>&1 | tail -5
```

- [ ] **Step 3: Implement `src/lib/canvas/renderer.ts`**

```typescript
// src/lib/canvas/renderer.ts

export interface CardColors {
  background: string
  title: string
  body: string
  attribution: string
  separator: string
}

export const LIGHT_COLORS: CardColors = {
  background: '#ffffff',
  title: '#111111',
  body: '#444444',
  attribution: '#aaaaaa',
  separator: '#eeeeee',
}

export const DARK_COLORS: CardColors = {
  background: '#1a1a2e',
  title: '#f0f0f0',
  body: '#cccccc',
  attribution: '#555555',
  separator: '#2a2a3e',
}

export interface CardSpec {
  image: HTMLImageElement | null
  title: string
  summary: string
  domain: string
  titleFont: string
  titleSize: number
  bodyFont: string
  bodySize: number
  width: number
  presetHeight: number  // 0 = auto
  autoHeight: boolean
  showAttribution: boolean
  theme: 'light' | 'dark'
}

/** Split text into lines that fit within maxWidth pixels. */
export function wrapText(
  ctx: CanvasRenderingContext2D,
  text: string,
  maxWidth: number
): string[] {
  if (!text) return []
  const words = text.split(' ')
  const lines: string[] = []
  let current = ''
  for (const word of words) {
    const candidate = current ? `${current} ${word}` : word
    if (ctx.measureText(candidate).width > maxWidth && current) {
      lines.push(current)
      current = word
    } else {
      current = candidate
    }
  }
  if (current) lines.push(current)
  return lines
}

export interface HeightParams {
  presetHeight: number
  imageHeight: number
  titleLines: number
  summaryLines: number
  titleLineHeight: number
  summaryLineHeight: number
  padding: number
  attributionHeight: number
  autoHeight: boolean
}

/** Calculate final card height. If autoHeight, compute from content. */
export function computeCardHeight(p: HeightParams): number {
  const contentHeight =
    p.imageHeight +
    p.titleLines * p.titleLineHeight +
    p.summaryLines * p.summaryLineHeight +
    3 * p.padding +
    p.attributionHeight
  if (p.autoHeight || p.presetHeight === 0) return contentHeight
  return p.presetHeight
}

/**
 * Render a social media card onto a canvas element.
 * Returns true if content fits within the preset height, false if it overflows.
 */
export function renderCard(canvas: HTMLCanvasElement, spec: CardSpec): boolean {
  const colors = spec.theme === 'dark' ? DARK_COLORS : LIGHT_COLORS
  const ctx = canvas.getContext('2d')!
  const PAD = 48
  const IMAGE_RATIO = 0.45
  const TITLE_LINE_HEIGHT = Math.round(spec.titleSize * 1.3)
  const BODY_LINE_HEIGHT = Math.round(spec.bodySize * 1.5)
  const ATTR_HEIGHT = spec.showAttribution ? 32 : 0
  const textWidth = spec.width - PAD * 2

  // Measure text
  ctx.font = `bold ${spec.titleSize}px "${spec.titleFont}", serif`
  const titleLines = wrapText(ctx, spec.title, textWidth)

  ctx.font = `${spec.bodySize}px "${spec.bodyFont}", sans-serif`
  const summaryLines = wrapText(ctx, spec.summary, textWidth)

  const imageHeight = Math.round(spec.width * IMAGE_RATIO)
  const contentHeight = computeCardHeight({
    presetHeight: spec.presetHeight,
    imageHeight,
    titleLines: titleLines.length,
    summaryLines: summaryLines.length,
    titleLineHeight: TITLE_LINE_HEIGHT,
    summaryLineHeight: BODY_LINE_HEIGHT,
    padding: PAD,
    attributionHeight: ATTR_HEIGHT,
    autoHeight: spec.autoHeight,
  })

  const fits = spec.presetHeight === 0 || spec.autoHeight ||
    contentHeight <= spec.presetHeight

  const finalHeight = spec.autoHeight || spec.presetHeight === 0
    ? contentHeight
    : spec.presetHeight

  canvas.width = spec.width
  canvas.height = finalHeight

  // Background
  ctx.fillStyle = colors.background
  ctx.fillRect(0, 0, spec.width, finalHeight)

  // Image zone
  if (spec.image) {
    const srcRatio = spec.image.naturalWidth / spec.image.naturalHeight
    const dstRatio = spec.width / imageHeight
    let sx = 0, sy = 0, sw = spec.image.naturalWidth, sh = spec.image.naturalHeight
    if (srcRatio > dstRatio) {
      sw = Math.round(sh * dstRatio)
      sx = Math.round((spec.image.naturalWidth - sw) / 2)
    } else {
      sh = Math.round(sw / dstRatio)
      sy = Math.round((spec.image.naturalHeight - sh) / 2)
    }
    ctx.drawImage(spec.image, sx, sy, sw, sh, 0, 0, spec.width, imageHeight)
  }

  // Gradient fade at image/text boundary
  const grad = ctx.createLinearGradient(0, imageHeight - 24, 0, imageHeight + 8)
  grad.addColorStop(0, 'rgba(0,0,0,0)')
  grad.addColorStop(1, colors.background)
  ctx.fillStyle = grad
  ctx.fillRect(0, imageHeight - 24, spec.width, 32)

  // Title
  let y = imageHeight + PAD
  ctx.font = `bold ${spec.titleSize}px "${spec.titleFont}", serif`
  ctx.fillStyle = colors.title
  for (const line of titleLines) {
    ctx.fillText(line, PAD, y)
    y += TITLE_LINE_HEIGHT
  }

  y += Math.round(PAD * 0.5)

  // Summary
  ctx.font = `${spec.bodySize}px "${spec.bodyFont}", sans-serif`
  ctx.fillStyle = colors.body
  for (const line of summaryLines) {
    ctx.fillText(line, PAD, y)
    y += BODY_LINE_HEIGHT
  }

  // Attribution
  if (spec.showAttribution && spec.domain) {
    const attrY = finalHeight - 16
    ctx.fillStyle = colors.separator
    ctx.fillRect(PAD, attrY - 20, spec.width - PAD * 2, 1)
    ctx.font = `13px sans-serif`
    ctx.fillStyle = colors.attribution
    ctx.fillText(spec.domain, PAD, attrY)
  }

  return fits
}
```

- [ ] **Step 4: Run tests — expect pass**

```bash
npx vitest run src/lib/canvas/renderer.test.ts 2>&1 | tail -10
```

Expected: 7 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/canvas/ 
git commit -m "feat: canvas renderer with text wrapping, zone layout, light/dark themes"
```

---

### Task 12: Svelte stores

**Files:**
- Create: `src/lib/stores/scrape.ts`
- Create: `src/lib/stores/editor.ts`
- Create: `src/lib/stores/config.ts`

- [ ] **Step 1: Create `src/lib/stores/scrape.ts`**

```typescript
// src/lib/stores/scrape.ts
import { writable } from 'svelte/store'
import type { ScrapedData } from '../types'

export const scraped = writable<ScrapedData | null>(null)
export const scraping = writable(false)
export const scrapeError = writable<string | null>(null)
```

- [ ] **Step 2: Create `src/lib/stores/editor.ts`**

```typescript
// src/lib/stores/editor.ts
import { writable } from 'svelte/store'
import type { PresetName } from '../types'
import { PLATFORM_PRESETS } from '../types'

export interface FontOverrides {
  titleFamily: string | null
  titleSize: number | null
  bodyFamily: string | null
  bodySize: number | null
}

export interface EditorState {
  title: string
  description: string
  articleText: string
  selectedImageSrc: string | null
  fontOverrides: FontOverrides
  preset: PresetName
  customWidth: number
  autoHeight: boolean
  showAttribution: boolean
  summarySource: 'scraped' | 'ai'
  activePrompt: string
  activeTemplateName: string
}

const DEFAULT_STATE: EditorState = {
  title: '',
  description: '',
  articleText: '',
  selectedImageSrc: null,
  fontOverrides: { titleFamily: null, titleSize: null, bodyFamily: null, bodySize: null },
  preset: 'Twitter / X',
  customWidth: 1200,
  autoHeight: false,
  showAttribution: false,
  summarySource: 'scraped',
  activePrompt: '',
  activeTemplateName: '',
}

export const editor = writable<EditorState>({ ...DEFAULT_STATE })

export function resetEditor(): void {
  editor.set({ ...DEFAULT_STATE })
}
```

- [ ] **Step 3: Create `src/lib/stores/config.ts`**

```typescript
// src/lib/stores/config.ts
import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types'

export const appConfig = writable<AppConfig | null>(null)
export const apiKey = writable<string>('')

export async function loadAppConfig(): Promise<void> {
  const config = await invoke<AppConfig>('load_config')
  appConfig.set(config)
  const key = await invoke<string | null>('get_api_key')
  apiKey.set(key ?? '')
}

export async function saveAppConfig(config: AppConfig): Promise<void> {
  await invoke('save_config', { config })
  appConfig.set(config)
}
```

- [ ] **Step 4: Verify TypeScript compilation**

```bash
npx tsc --noEmit 2>&1 | head -20
```

Expected: no errors (or only minor type warnings unrelated to our files).

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/
git commit -m "feat: Svelte stores for scrape state, editor state, and config"
```

---

## — PHASE 4: UI COMPONENTS —

### Task 13: App shell + URL bar

**Files:**
- Modify: `src/App.svelte`
- Create: `src/lib/components/UrlBar.svelte`

- [ ] **Step 1: Create `src/lib/components/UrlBar.svelte`**

```svelte
<!-- src/lib/components/UrlBar.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { scraped, scraping, scrapeError } from '../stores/scrape'
  import { editor, resetEditor } from '../stores/editor'
  import { appConfig } from '../stores/config'
  import { injectGoogleFonts } from '../fonts'
  import type { ScrapedData } from '../types'

  let url = ''

  // Pre-fill with last used URL on mount
  import { onMount } from 'svelte'
  onMount(async () => {
    const last = await invoke<string | null>('get_last_url')
    if (last) url = last
  })

  async function handleScrape() {
    if (!url.trim()) return
    scraping.set(true)
    scrapeError.set(null)
    try {
      await invoke('save_last_url', { url })
      const cfg = $appConfig
      const scrapingConfig = cfg?.scraping ?? {
        article_min_chars: 200, image_min_width: 300, max_stylesheets: 5,
      }
      const data = await invoke<ScrapedData>('scrape_url', { url, scrapingConfig })
      scraped.set(data)

      // Inject Google Fonts if detected
      if (data.detected_fonts.google_fonts_url) {
        injectGoogleFonts(data.detected_fonts.google_fonts_url)
      }

      // Pre-populate editor
      const defaultTemplate = cfg?.prompt_templates.find(t => t.default)
        ?? cfg?.prompt_templates[0]
      editor.update(e => ({
        ...e,
        title: data.title,
        description: data.description,
        articleText: data.article_text,
        selectedImageSrc: data.og_image
          ?? data.all_images.find(i => (i.width ?? 0) >= scrapingConfig.image_min_width)?.src
          ?? data.all_images[0]?.src
          ?? null,
        fontOverrides: {
          titleFamily: data.detected_fonts.heading_family,
          titleSize: null,
          bodyFamily: data.detected_fonts.body_family,
          bodySize: null,
        },
        activePrompt: defaultTemplate?.prompt ?? '',
        activeTemplateName: defaultTemplate?.name ?? '',
      }))
    } catch (err) {
      scrapeError.set(String(err))
    } finally {
      scraping.set(false)
    }
  }
</script>

<div class="url-bar">
  <input
    type="url"
    bind:value={url}
    placeholder="https://myblog.com/my-article"
    disabled={$scraping}
    on:keydown={e => e.key === 'Enter' && handleScrape()}
  />
  <button on:click={handleScrape} disabled={$scraping || !url.trim()}>
    {$scraping ? 'Scraping…' : 'Scrape'}
  </button>
</div>

{#if $scrapeError}
  <p class="error">{$scrapeError}</p>
{/if}

<style>
  .url-bar { display: flex; gap: 0.5rem; padding: 0.6rem 0.75rem; }
  input { flex: 1; }
  .error { color: var(--color-error, #e07b39); font-size: 0.8rem; padding: 0 0.75rem; }
</style>
```

- [ ] **Step 2: Replace `src/App.svelte` with split-panel layout**

```svelte
<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from 'svelte'
  import { loadAppConfig } from './lib/stores/config'
  import UrlBar from './lib/components/UrlBar.svelte'
  import ContentSection from './lib/components/ContentSection.svelte'
  import SummarySection from './lib/components/SummarySection.svelte'
  import StyleSection from './lib/components/StyleSection.svelte'
  import PreviewPanel from './lib/components/PreviewPanel.svelte'
  import SettingsModal from './lib/components/SettingsModal.svelte'

  let showSettings = false

  onMount(() => loadAppConfig())
</script>

<div class="app">
  <header class="titlebar">
    <span class="app-name">macscraper</span>
    <button class="settings-btn" on:click={() => showSettings = true}>⚙</button>
  </header>

  <div class="workspace">
    <!-- Left: editor panel -->
    <div class="editor-panel">
      <UrlBar />
      <div class="accordion">
        <ContentSection />
        <SummarySection />
        <StyleSection />
      </div>
    </div>

    <!-- Right: live preview -->
    <div class="preview-panel">
      <PreviewPanel />
    </div>
  </div>
</div>

{#if showSettings}
  <SettingsModal on:close={() => showSettings = false} />
{/if}

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) { margin: 0; font-family: system-ui, sans-serif; background: #0f0f1a; color: #e0e0e0; }
  :global(input, textarea, select, button) {
    font-family: inherit; font-size: inherit;
    background: #1e1e2e; color: #e0e0e0; border: 1px solid #333; border-radius: 4px;
  }
  :global(button) { cursor: pointer; padding: 0.3rem 0.75rem; }
  :global(button:disabled) { opacity: 0.5; cursor: default; }

  .app { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
  .titlebar {
    display: flex; justify-content: space-between; align-items: center;
    padding: 0.4rem 0.75rem; background: #16162a; border-bottom: 1px solid #333;
  }
  .app-name { font-weight: 600; }
  .settings-btn { background: none; border: none; font-size: 1.1rem; }
  .workspace { display: flex; flex: 1; overflow: hidden; }
  .editor-panel { width: 42%; border-right: 1px solid #333; display: flex; flex-direction: column; overflow: hidden; }
  .accordion { flex: 1; overflow-y: auto; }
  .preview-panel { flex: 1; overflow-y: auto; }
</style>
```

- [ ] **Step 3: Verify app runs**

```bash
npm run dev &
sleep 3 && curl -s http://localhost:1420 | grep -c "macscraper" || echo "check browser"
```

Expected: app loads in browser at `http://localhost:1420`.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte src/lib/components/UrlBar.svelte
git commit -m "feat: app shell split-panel layout and URL bar with scrape trigger"
```

---

### Task 14: Content section + image picker

**Files:**
- Create: `src/lib/components/ImagePicker.svelte`
- Create: `src/lib/components/ContentSection.svelte`

- [ ] **Step 1: Create `src/lib/components/ImagePicker.svelte`**

```svelte
<!-- src/lib/components/ImagePicker.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import { editor } from '../stores/editor'
  import { scraped } from '../stores/scrape'
  import type { ImageMeta } from '../types'

  $: images = $scraped?.all_images ?? []
  $: selected = $editor.selectedImageSrc

  function selectImage(src: string) {
    editor.update(e => ({ ...e, selectedImageSrc: src }))
  }

  async function pickFromDisk() {
    const file = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['png','jpg','jpeg','webp','gif'] }] })
    if (typeof file === 'string') {
      // Convert local path to a data URL via fetch_image
      const dataUrl = await invoke<string>('fetch_image', { url: `file://${file}` })
      editor.update(e => ({ ...e, selectedImageSrc: dataUrl }))
    }
  }
</script>

<div class="image-row">
  {#each images as img}
    <button
      class="thumb"
      class:selected={img.src === selected}
      on:click={() => selectImage(img.src)}
      title={img.alt}
    >
      <img src={img.src} alt={img.alt} loading="lazy" />
    </button>
  {/each}
  <button class="thumb disk-tile" on:click={pickFromDisk} title="Upload from disk">
    +disk
  </button>
</div>

<style>
  .image-row { display: flex; gap: 0.3rem; flex-wrap: wrap; padding: 0.25rem 0; }
  .thumb { width: 56px; height: 40px; padding: 0; border: 2px solid transparent; border-radius: 3px; overflow: hidden; }
  .thumb.selected { border-color: #4a9eff; }
  .thumb img { width: 100%; height: 100%; object-fit: cover; }
  .disk-tile { background: #252535; border-style: dashed; font-size: 0.65rem; color: #888; }
</style>
```

- [ ] **Step 2: Add `tauri-plugin-dialog` for file picker**

```bash
cargo add tauri-plugin-dialog --manifest-path src-tauri/Cargo.toml
npm install @tauri-apps/plugin-dialog
```

Register in `lib.rs`:
```rust
.plugin(tauri_plugin_dialog::init())
```

Add to `capabilities/default.json` permissions: `"dialog:allow-open"`.

- [ ] **Step 3: Create `src/lib/components/ContentSection.svelte`**

```svelte
<!-- src/lib/components/ContentSection.svelte -->
<script lang="ts">
  import { scraped } from '../stores/scrape'
  import { editor } from '../stores/editor'
  import ImagePicker from './ImagePicker.svelte'
</script>

<details open>
  <summary>🖼 Content</summary>
  <div class="section-body">
    {#if $scraped}
      <label class="field-label">IMAGE</label>
      <ImagePicker />

      <label class="field-label" for="title-input">TITLE</label>
      <input
        id="title-input"
        type="text"
        value={$editor.title}
        on:input={e => editor.update(s => ({ ...s, title: e.currentTarget.value }))}
      />

      <label class="field-label" for="desc-input">DESCRIPTION</label>
      <textarea
        id="desc-input"
        rows="3"
        value={$editor.description}
        on:input={e => editor.update(s => ({ ...s, description: e.currentTarget.value }))}
      ></textarea>

      <details>
        <summary class="field-label">FULL ARTICLE TEXT (for AI context) ▸</summary>
        <textarea
          rows="5"
          value={$editor.articleText}
          on:input={e => editor.update(s => ({ ...s, articleText: e.currentTarget.value }))}
        ></textarea>
      </details>
    {:else}
      <p class="hint">Scrape a URL to see content.</p>
    {/if}
  </div>
</details>

<style>
  details { border-bottom: 1px solid #333; }
  summary { padding: 0.5rem 0.75rem; cursor: pointer; font-weight: 600; background: #1a1a2e; font-size: 0.85rem; }
  .section-body { padding: 0.6rem 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .field-label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  input, textarea { width: 100%; resize: vertical; }
  .hint { font-size: 0.8rem; color: #666; }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ImagePicker.svelte src/lib/components/ContentSection.svelte \
        src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/capabilities/default.json
git commit -m "feat: content section with image picker (page thumbnails + disk upload)"
```

---

### Task 15: Summary section

**Files:**
- Create: `src/lib/components/SummarySection.svelte`

- [ ] **Step 1: Create the component**

```svelte
<!-- src/lib/components/SummarySection.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { appConfig, apiKey } from '../stores/config'
  import { substituteTemplate } from '../template'

  let generating = false
  let genError = ''
  let showSaveInput = false
  let newTemplateName = ''

  $: templates = $appConfig?.prompt_templates ?? []

  function selectTemplate(name: string) {
    const tpl = templates.find(t => t.name === name)
    if (tpl) editor.update(e => ({ ...e, activeTemplateName: name, activePrompt: tpl.prompt }))
  }

  async function generate() {
    const state = $editor
    const cfg = $appConfig
    if (!cfg) return
    generating = true
    genError = ''
    try {
      const vars = {
        title: state.title,
        description: state.description,
        article_text: state.articleText,
      }
      const resolvedPrompt = substituteTemplate(state.activePrompt, vars)
      const summary = await invoke<string>('generate_summary', {
        prompt: resolvedPrompt,
        config: { endpoint: cfg.llm.endpoint, model: cfg.llm.model, api_key: $apiKey || null },
      })
      editor.update(e => ({ ...e, description: summary }))
    } catch (err) {
      genError = String(err)
    } finally {
      generating = false
    }
  }

  async function saveAsTemplate() {
    if (!newTemplateName.trim() || !$appConfig) return
    const updated = {
      ...$appConfig,
      prompt_templates: [
        ...$appConfig.prompt_templates,
        { name: newTemplateName.trim(), default: false, prompt: $editor.activePrompt },
      ],
    }
    await invoke('save_config', { config: updated })
    appConfig.set(updated)
    showSaveInput = false
    newTemplateName = ''
  }
</script>

<details open>
  <summary>🤖 Summary</summary>
  <div class="section-body">
    <div class="toggle-row">
      <button
        class:active={$editor.summarySource === 'scraped'}
        on:click={() => editor.update(e => ({ ...e, summarySource: 'scraped' }))}
      >Scraped</button>
      <button
        class:active={$editor.summarySource === 'ai'}
        on:click={() => editor.update(e => ({ ...e, summarySource: 'ai' }))}
      >AI Generate</button>
    </div>

    {#if $editor.summarySource === 'ai'}
      <label class="field-label">PROMPT TEMPLATE</label>
      <select value={$editor.activeTemplateName} on:change={e => selectTemplate(e.currentTarget.value)}>
        {#each templates as tpl}
          <option value={tpl.name}>{tpl.name}</option>
        {/each}
      </select>

      <textarea
        rows="4"
        value={$editor.activePrompt}
        on:input={e => editor.update(s => ({ ...s, activePrompt: e.currentTarget.value }))}
      ></textarea>

      <div class="row">
        <button on:click={generate} disabled={generating}>
          {generating ? 'Generating…' : 'Generate ✨'}
        </button>
        <button class="secondary" on:click={() => showSaveInput = !showSaveInput}>
          Save as template…
        </button>
      </div>

      {#if showSaveInput}
        <div class="save-row">
          <input type="text" placeholder="Template name" bind:value={newTemplateName} />
          <button on:click={saveAsTemplate} disabled={!newTemplateName.trim()}>Save</button>
        </div>
      {/if}

      {#if genError}
        <p class="error">{genError}</p>
      {/if}
    {/if}
  </div>
</details>

<style>
  details { border-bottom: 1px solid #333; }
  summary { padding: 0.5rem 0.75rem; cursor: pointer; font-weight: 600; background: #1a1a2e; font-size: 0.85rem; }
  .section-body { padding: 0.6rem 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .field-label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  .toggle-row, .row { display: flex; gap: 0.4rem; }
  .save-row { display: flex; gap: 0.4rem; }
  .save-row input { flex: 1; }
  button.active { background: #4a9eff; color: white; border-color: #4a9eff; }
  button.secondary { background: transparent; }
  textarea, select { width: 100%; resize: vertical; }
  .error { color: #e07b39; font-size: 0.8rem; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/SummarySection.svelte
git commit -m "feat: summary section with scraped/AI toggle, template picker, inline save-as-template"
```

---

### Task 16: Style section

**Files:**
- Create: `src/lib/components/StyleSection.svelte`

- [ ] **Step 1: Create the component**

```svelte
<!-- src/lib/components/StyleSection.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { appConfig } from '../stores/config'
  import { scraped } from '../stores/scrape'
  import { buildFontList } from '../fonts'
  import { PLATFORM_PRESETS, type PresetName } from '../types'

  $: detectedHeading = $scraped?.detected_fonts.heading_family ?? null
  $: detectedBody = $scraped?.detected_fonts.body_family ?? null
  $: fontList = buildFontList(detectedHeading, detectedBody)

  $: titleFont = $editor.fontOverrides.titleFamily ?? detectedHeading ?? 'Georgia'
  $: bodyFont = $editor.fontOverrides.bodyFamily ?? detectedBody ?? 'Inter'
  $: titleSize = $editor.fontOverrides.titleSize ?? 48
  $: bodySize = $editor.fontOverrides.bodySize ?? 22

  async function saveStyleDefault() {
    if (!$appConfig) return
    const updated = {
      ...$appConfig,
      style: { title_family: titleFont, title_size: titleSize, body_family: bodyFont, body_size: bodySize },
    }
    await invoke('save_config', { config: updated })
    appConfig.set(updated)
  }
</script>

<details>
  <summary>🎨 Style</summary>
  <div class="section-body">
    <div class="font-grid">
      <span class="field-label">TITLE FONT</span>
      <span class="field-label">SIZE</span>
      <select
        value={titleFont}
        on:change={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, titleFamily: e.currentTarget.value } }))}
      >
        {#each fontList as f}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
      <input type="number" min="12" max="120" value={titleSize}
        on:input={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, titleSize: +e.currentTarget.value } }))}
      />

      <span class="field-label">BODY FONT</span>
      <span class="field-label">SIZE</span>
      <select
        value={bodyFont}
        on:change={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, bodyFamily: e.currentTarget.value } }))}
      >
        {#each fontList as f}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
      <input type="number" min="10" max="80" value={bodySize}
        on:input={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, bodySize: +e.currentTarget.value } }))}
      />
    </div>

    <label class="field-label">PLATFORM PRESET</label>
    <select
      value={$editor.preset}
      on:change={e => editor.update(s => ({ ...s, preset: e.currentTarget.value as PresetName }))}
    >
      {#each PLATFORM_PRESETS as p}
        <option value={p.name}>{p.name} ({p.width}×{p.height || 'auto'})</option>
      {/each}
    </select>

    {#if $editor.preset === 'Custom'}
      <label class="field-label">CUSTOM WIDTH (px)</label>
      <input type="number" min="400" max="3000" value={$editor.customWidth}
        on:input={e => editor.update(s => ({ ...s, customWidth: +e.currentTarget.value }))}
      />
    {/if}

    <label class="checkbox-label">
      <input type="checkbox"
        checked={$editor.autoHeight}
        on:change={e => editor.update(s => ({ ...s, autoHeight: e.currentTarget.checked }))}
      />
      ↕ Auto height
    </label>

    <label class="checkbox-label">
      <input type="checkbox"
        checked={$editor.showAttribution}
        on:change={e => editor.update(s => ({ ...s, showAttribution: e.currentTarget.checked }))}
      />
      Show source attribution
    </label>

    <button on:click={saveStyleDefault}>Save style as default</button>
  </div>
</details>

<style>
  details { border-bottom: 1px solid #333; }
  summary { padding: 0.5rem 0.75rem; cursor: pointer; font-weight: 600; background: #1a1a2e; font-size: 0.85rem; }
  .section-body { padding: 0.6rem 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .field-label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  .font-grid { display: grid; grid-template-columns: 1fr 56px; gap: 0.3rem; align-items: center; }
  select { width: 100%; }
  .checkbox-label { display: flex; align-items: center; gap: 0.4rem; font-size: 0.85rem; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/StyleSection.svelte
git commit -m "feat: style section with font pickers, platform presets, auto-height, attribution"
```

---

### Task 17: Settings modal

**Files:**
- Create: `src/lib/components/SettingsModal.svelte`

- [ ] **Step 1: Create the component**

```svelte
<!-- src/lib/components/SettingsModal.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { open as openPath } from '@tauri-apps/plugin-shell'
  import { appConfig, apiKey, saveAppConfig } from '../stores/config'

  const dispatch = createEventDispatcher()

  let endpoint = $appConfig?.llm.endpoint ?? 'https://api.openai.com/v1'
  let model = $appConfig?.llm.model ?? 'gpt-4o-mini'
  let key = $apiKey

  async function save() {
    if (!$appConfig) return
    await saveAppConfig({ ...$appConfig, llm: { endpoint, model } })
    await invoke('set_api_key', { key })
    apiKey.set(key)
    dispatch('close')
  }

  async function openConfigFile() {
    const path = await invoke<string>('get_config_path')
    await openPath(path)
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="overlay" on:click|self={() => dispatch('close')}>
  <div class="modal">
    <h2>Settings</h2>

    <label>LLM Endpoint
      <input type="url" bind:value={endpoint} />
    </label>
    <label>Model
      <input type="text" bind:value={model} />
    </label>
    <label>API Key
      <input type="password" bind:value={key} placeholder="sk-… (stored in OS keychain)" />
    </label>

    <p class="note">
      To change the default prompt template or scraping thresholds,
      edit <code>config.toml</code> directly.
    </p>

    <div class="actions">
      <button class="secondary" on:click={openConfigFile}>Open config file</button>
      <button on:click={save}>Save</button>
    </div>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .modal { background: #16162a; border: 1px solid #333; border-radius: 8px; padding: 1.5rem; min-width: 360px; display: flex; flex-direction: column; gap: 0.75rem; }
  h2 { margin: 0; font-size: 1.1rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; }
  input { width: 100%; }
  .note { font-size: 0.78rem; color: #888; }
  .actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  button.secondary { background: transparent; }
</style>
```

- [ ] **Step 2: Add `get_config_path` command to `config.rs`**

```rust
#[tauri::command]
pub fn get_config_path(app: AppHandle) -> Result<String, String> {
    config_path(&app).map(|p| p.to_string_lossy().into_owned())
}
```

Register `config::get_config_path` in `lib.rs` handler.

- [ ] **Step 3: Add `tauri-plugin-shell` for `open`**

```bash
cargo add tauri-plugin-shell --manifest-path src-tauri/Cargo.toml
npm install @tauri-apps/plugin-shell
```

Register `.plugin(tauri_plugin_shell::init())` in `lib.rs`. Add `"shell:allow-open"` to `capabilities/default.json`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/SettingsModal.svelte src-tauri/src/config.rs src-tauri/src/lib.rs \
        src-tauri/Cargo.toml src-tauri/capabilities/default.json
git commit -m "feat: settings modal for LLM config and API key"
```

---

### Task 18: CardCanvas + PreviewPanel

**Files:**
- Create: `src/lib/components/CardCanvas.svelte`
- Create: `src/lib/components/PreviewPanel.svelte`

- [ ] **Step 1: Create `src/lib/components/CardCanvas.svelte`**

```svelte
<!-- src/lib/components/CardCanvas.svelte -->
<script lang="ts">
  import { onMount, afterUpdate } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { appConfig } from '../stores/config'
  import { scraped } from '../stores/scrape'
  import { PLATFORM_PRESETS } from '../types'
  import { renderCard, type CardSpec } from '../canvas/renderer'

  export let theme: 'light' | 'dark'

  let canvas: HTMLCanvasElement
  let overflows = false
  let imageEl: HTMLImageElement | null = null
  let lastImageSrc = ''

  $: preset = PLATFORM_PRESETS.find(p => p.name === $editor.preset) ?? PLATFORM_PRESETS[0]
  $: titleFont = $editor.fontOverrides.titleFamily
    ?? $scraped?.detected_fonts.heading_family
    ?? $appConfig?.style?.title_family
    ?? 'Georgia'
  $: bodyFont = $editor.fontOverrides.bodyFamily
    ?? $scraped?.detected_fonts.body_family
    ?? $appConfig?.style?.body_family
    ?? 'Inter'
  $: titleSize = $editor.fontOverrides.titleSize ?? $appConfig?.style?.title_size ?? 48
  $: bodySize = $editor.fontOverrides.bodySize ?? $appConfig?.style?.body_size ?? 22
  $: domain = (() => { try { return new URL($scraped ? 'https://x.com' : '').hostname } catch { return '' } })()

  async function loadImage(src: string) {
    if (!src || src === lastImageSrc) return
    lastImageSrc = src
    // Use data URL if already one, otherwise fetch via Rust to avoid CORS taint
    const dataUrl = src.startsWith('data:') ? src : await invoke<string>('fetch_image', { url: src })
    const img = new Image()
    img.src = dataUrl
    await new Promise<void>(res => { img.onload = () => res() })
    imageEl = img
    draw()
  }

  function draw() {
    if (!canvas) return
    const spec: CardSpec = {
      image: imageEl,
      title: $editor.title,
      summary: $editor.description,
      domain,
      titleFont,
      titleSize,
      bodyFont,
      bodySize,
      width: $editor.preset === 'Custom' ? $editor.customWidth : preset.width,
      presetHeight: preset.height,
      autoHeight: $editor.autoHeight || preset.height === 0,
      showAttribution: $editor.showAttribution,
      theme,
    }
    overflows = !renderCard(canvas, spec)
  }

  // Reactive draw on any editor change
  $: { $editor; $appConfig; draw() }

  // Load image when selected image changes
  $: if ($editor.selectedImageSrc) loadImage($editor.selectedImageSrc)

  onMount(draw)
</script>

<div class="canvas-wrapper">
  {#if overflows && !$editor.autoHeight}
    <span class="overflow-badge">⚠ Text overflows — enable ↕ Auto height</span>
  {/if}
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .canvas-wrapper { position: relative; width: 100%; }
  canvas { width: 100%; height: auto; display: block; border-radius: 4px; }
  .overflow-badge {
    position: absolute; top: 6px; right: 6px;
    background: #e07b39; color: white; font-size: 0.72rem;
    padding: 0.2rem 0.5rem; border-radius: 4px; z-index: 1;
  }
</style>
```

- [ ] **Step 2: Create `src/lib/components/PreviewPanel.svelte`**

```svelte
<!-- src/lib/components/PreviewPanel.svelte -->
<script lang="ts">
  import { editor } from '../stores/editor'
  import { PLATFORM_PRESETS } from '../types'
  import CardCanvas from './CardCanvas.svelte'

  $: preset = PLATFORM_PRESETS.find(p => p.name === $editor.preset) ?? PLATFORM_PRESETS[0]
  $: dims = $editor.preset === 'Custom'
    ? `${$editor.customWidth}×auto`
    : `${preset.width}×${$editor.autoHeight ? 'auto' : preset.height}`

  let lightCanvas: CardCanvas
  let darkCanvas: CardCanvas

  function download(theme: 'light' | 'dark') {
    const ref = theme === 'light' ? lightCanvas : darkCanvas
    const canvas: HTMLCanvasElement = (ref as any)?.canvas
    if (!canvas) return
    canvas.toBlob(blob => {
      if (!blob) return
      const a = document.createElement('a')
      a.href = URL.createObjectURL(blob)
      a.download = `card-${theme}-${Date.now()}.png`
      a.click()
      URL.revokeObjectURL(a.href)
    }, 'image/png')
  }
</script>

<div class="preview">
  <div class="preview-header">
    Live preview · {$editor.preset} ({dims}) · <span class="dot">auto-updating</span>
  </div>
  <div class="cards">
    <div class="card-block">
      <span class="label">☀️ White</span>
      <CardCanvas bind:this={lightCanvas} theme="light" />
      <button on:click={() => download('light')}>⬇ Download PNG</button>
    </div>
    <div class="card-block">
      <span class="label">🌑 Dark</span>
      <CardCanvas bind:this={darkCanvas} theme="dark" />
      <button on:click={() => download('dark')}>⬇ Download PNG</button>
    </div>
  </div>
</div>

<style>
  .preview { display: flex; flex-direction: column; height: 100%; }
  .preview-header { padding: 0.5rem 0.75rem; font-size: 0.75rem; color: #888; background: #1a1a2e; border-bottom: 1px solid #333; }
  .dot { color: #4a9eff; }
  .cards { flex: 1; overflow-y: auto; padding: 0.75rem; display: flex; flex-direction: column; gap: 1rem; }
  .card-block { display: flex; flex-direction: column; gap: 0.3rem; }
  .label { font-size: 0.75rem; color: #888; }
  button { align-self: flex-start; font-size: 0.75rem; margin-top: 0.2rem; }
</style>
```

- [ ] **Step 3: Full integration check — run the app**

```bash
npm run tauri dev
```

Expected: app opens, paste a URL and click Scrape. Both preview cards should render and update live as you edit.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/CardCanvas.svelte src/lib/components/PreviewPanel.svelte
git commit -m "feat: card canvas with live rendering, overflow detection, and PNG download"
```

---

## — PHASE 5: WIRING & POLISH —

### Task 19: Final integration + CLAUDE.md update

**Files:**
- Modify: `CLAUDE.md`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Set window size in `tauri.conf.json`**

Under `windows[0]`, set:
```json
{
  "title": "macscraper",
  "width": 1280,
  "height": 800,
  "minWidth": 900,
  "minHeight": 600,
  "resizable": true
}
```

- [ ] **Step 2: Update `CLAUDE.md` to reflect the Rust/Svelte stack**

Replace the Python-specific tooling section with:

```markdown
### Development Workflow

**Rust backend:**
```bash
cargo test --manifest-path src-tauri/Cargo.toml   # run all Rust tests
cargo build --manifest-path src-tauri/Cargo.toml  # check compilation
```

**Frontend:**
```bash
npm run test        # Vitest unit tests
npm run dev         # Vite dev server only
npm run tauri dev   # full Tauri dev mode (opens window)
npm run tauri build # production build
```

# CODING CONVENTIONS
... (keep existing conventions, applicable to Rust and TypeScript)
```

- [ ] **Step 3: Run full test suite**

```bash
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
npx vitest run 2>&1 | tail -5
```

Expected: all Rust and TypeScript tests pass.

- [ ] **Step 4: Production build check**

```bash
npm run tauri build 2>&1 | tail -10
```

Expected: build succeeds and produces a binary in `src-tauri/target/release/`.

- [ ] **Step 5: Final commit**

```bash
git add CLAUDE.md src-tauri/tauri.conf.json
git commit -m "chore: set window dimensions and update CLAUDE.md for Rust/Svelte stack"
```

---

## Self-Review Checklist

After implementation, verify against spec:

- [ ] Metadata waterfall: og:title → twitter:title → `<title>` → h1
- [ ] Description: og:description → twitter:description → meta description
- [ ] Image: og:image → twitter:image → first large img (configurable min_width)
- [ ] Image picker: OG pre-selected, page images shown, "+disk" tile works
- [ ] Article text: two-pass (semantic → density), `article_min_chars` configurable
- [ ] Font detection: stylesheets + inline `<style>` + inline `style=""` fallback
- [ ] Google Fonts URL injected as `<link>` in WebView
- [ ] Canvas: image zone (~45%), gradient fade, title, summary, attribution zones
- [ ] White + dark variants with correct colours per spec
- [ ] Overflow warning badge when text exceeds fixed preset height
- [ ] Auto-height checkbox overrides preset height
- [ ] Template substitution: `{{title}}`, `{{description}}`, `{{article_text}}`
- [ ] Unknown template vars left as-is
- [ ] Save-as-template: inline input, appends to config.toml
- [ ] API key in OS keychain (never in TOML)
- [ ] Last URL in Tauri KV store (not config.toml)
- [ ] config.toml via `app_config_dir()` — no hardcoded paths
- [ ] All scraping thresholds configurable in `[scraping]` block
- [ ] Font picker: detected ✦ → curated Google Fonts → generics
- [ ] Download: `canvas.toBlob()` → browser-style save
