# macscraper — Design Spec
**Date:** 2026-03-31  
**Status:** Approved

---

## Overview

A Tauri 2 desktop application that scrapes a blog post URL, extracts social-sharing metadata, optionally generates an AI summary, and produces social media card images (white and dark variants) ready for download. All logic runs locally — no cloud service required.

---

## Tech Stack

| Layer | Technology | Version |
|---|---|---|
| App shell | Tauri | 2.10.3 (stable) |
| Backend language | Rust | stable |
| HTTP client | reqwest | 0.13 (rustls-tls, no default features) |
| HTML parser | scraper | 0.26 |
| LLM API client | async-openai | 0.31 (custom base URL) |
| Async runtime | tokio | 1 |
| Serialisation | serde + serde_json | 1 |
| Frontend framework | Svelte + Vite | latest |
| Image composition | HTML Canvas 2D | (browser built-in) |
| Config format | TOML | (toml crate) |
| Secret storage | Tauri secure store | (OS keychain) |

---

## UI Architecture — Split-Panel Workbench

The application is a single window split into two panels:

### Left Panel — Editor (42% width)
Organised into collapsible accordion sections. Contains:

1. **URL Bar** (always visible, above accordion)
   - Text input pre-filled with last used URL (persisted in Tauri key-value store between sessions; not stored in `config.toml`)
   - "Scrape" button — triggers `scrape_url` Tauri command
   - Shows loading state during scrape

2. **🖼 Content Section** (open by default)
   - **Image picker** — horizontal scrollable row of thumbnails:
     - All `<img>` elements from the scraped page (resolved to absolute URLs)
     - Pre-selection priority: OG image if present → first image meeting `image_min_width` → first image regardless of size → "+disk" tile only (if no images found at all)
     - Pre-selected thumbnail shown with accent border
     - Final thumbnail is a "+ disk" tile that opens a native Tauri file picker
     - Clicking any thumbnail selects it as the card image
   - **Title field** — single-line editable text, pre-filled from scraped title
   - **Description field** — multi-line editable textarea, pre-filled from scraped meta description. No automatic trimming — user edits freely.
   - **Full Article Text** — collapsible sub-section (collapsed by default). Multi-line editable textarea pre-filled with scraped article body. Used as AI context. No automatic trimming — user edits freely.

3. **🤖 Summary Section** (open by default)
   - **Source toggle** — two buttons: "Scraped" (default) / "AI Generate"
   - When "Scraped": description field content is used as-is on the card
   - When "AI Generate":
     - **Template dropdown** — lists named templates from `config.toml`; pre-selects the one with `default = true`
     - **Prompt textarea** — editable per-session; pre-filled from selected template. Editing does not auto-save back to TOML.
     - **"Generate ✨" button** — substitutes template variables in Svelte, calls `generate_summary` with the fully-resolved prompt string, writes response into the description field
     - LLM-generated response lands in the description field, which remains fully editable after generation
     - **"Save as template…" button** — clicking it reveals an inline name input and "Save" confirm button directly below the button (no modal); on confirm, appends a new `[[prompt_templates]]` entry to `config.toml`
   - User can switch between Scraped and AI at any time; re-generate as many times as needed

4. **🎨 Style Section** (collapsed by default)
   - **Title font** — dropdown + numeric size input (px)
   - **Body font** — dropdown + numeric size input (px)
   - Detected fonts are marked "✦ detected" and pre-selected; user can change freely
   - Font dropdown contents (in order): detected fonts (✦), then curated Google Fonts (`Playfair Display`, `Merriweather`, `Lora`, `Roboto`, `Inter`, `Open Sans`, `Source Sans Pro`, `Nunito`, `Raleway`, `Montserrat`), then generic fallbacks (`serif`, `sans-serif`, `monospace`)
   - **Platform preset** — dropdown (see Platform Presets below)
   - **"↕ Auto height"** — checkbox below the preset dropdown; when checked, card height grows to fit all text regardless of preset; when unchecked and text overflows, a warning badge appears on the preview
   - **"Show source attribution"** — checkbox; toggles domain line at card bottom
   - **"Save style as default"** — writes current font/size selections to `[style]` block in `config.toml`

5. **⚙ Settings** (gear icon in titlebar, opens modal)
   - LLM endpoint URL
   - LLM model name
   - API key field (stored in OS keychain via Tauri secure store, never written to TOML)
   - "Open config file" button — opens `config.toml` in the system default editor
   - **Note:** changing the default prompt template (`default = true`) is intentionally not exposed in the UI; the user edits `config.toml` directly via the "Open config file" button

### Right Panel — Live Preview (remaining width)
- Header bar: shows active preset name and dimensions; "auto-updating" indicator
- **☀️ White card** — HTML Canvas, updates live on every keystroke in the editor
  - "⬇ Download PNG" button below
- **🌑 Dark card** — same Canvas logic, dark background, updates in sync
  - "⬇ Download PNG" button below
- Both cards scroll vertically if taller than the panel
- Download uses `canvas.toBlob()` → Tauri save dialog (browser-style)

---

## Tauri Commands (Rust → Svelte interface)

### `scrape_url(url: String) -> Result<ScrapedData, String>`
Fetches the page HTML via `reqwest`, parses with `scraper`, returns `ScrapedData`.

### `fetch_image(url: String) -> Result<String, String>`
Fetches image bytes in Rust, returns a `data:image/...;base64,...` string.  
**Purpose:** Canvas `drawImage()` taints the canvas with cross-origin URLs, blocking `toBlob()`. Routing through Rust bypasses CORS and keeps the canvas exportable.

### `generate_summary(prompt: String, config: LLMConfig) -> Result<String, String>`
Receives a fully-resolved prompt string (template variables already substituted by Svelte). Calls the configured OpenAI-compatible endpoint via `async-openai` with a custom base URL. Returns the full response text. Streaming via Tauri events is out of scope for v1.

### `load_config() -> Result<AppConfig, String>`
Reads and deserialises `config.toml` from the OS config directory.

### `save_config(config: AppConfig) -> Result<(), String>`
Serialises and writes `config.toml`. Used by "Save style as default" and "Save as template…".

---

## Data Structures

```rust
pub struct ScrapedData {
    pub title: String,
    pub description: String,
    pub og_image: Option<String>,
    pub all_images: Vec<ImageMeta>,
    pub article_text: String,
    pub detected_fonts: DetectedFonts,
}

pub struct ImageMeta {
    pub src: String,       // absolute URL
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub struct DetectedFonts {
    pub heading_family: Option<String>,
    pub body_family: Option<String>,
    pub google_fonts_url: Option<String>, // inject as <link> in WebView if present
}
```

---

## Scraping Logic

### Metadata Extraction — Priority Waterfall

| Field | Priority order |
|---|---|
| Title | `og:title` → `twitter:title` → `<title>` → first `<h1>` |
| Description | `og:description` → `twitter:description` → `meta[name=description]` |
| OG image | `og:image` → `twitter:image` → first `<img>` with width ≥ `image_min_width` |
| All images | All `<img>` elements with `src`; relative URLs resolved to absolute |

### Article Text Extraction — Two-Pass

**Pass 1 — Semantic selectors:**  
Try `article`, `main`, `[role=main]`, `.post-content`, `.entry-content` in order. Take the longest text content match.

**Pass 2 — Density fallback:**  
If Pass 1 yields fewer than `article_min_chars` characters, score all block-level elements by text/tag-count ratio (Readability heuristic, ~50 lines of Rust). Take the highest-scoring block.

No external crate (avoids `article_scraper`'s webkit2gtk dependency). Implemented with `scraper` only.

### Font Detection — Step by Step

1. Collect all `<link rel=stylesheet>` hrefs (resolved to absolute) and all inline `<style>` blocks.
2. Check stylesheet URLs for `fonts.googleapis.com`. If found, store the full URL as `google_fonts_url`. Frontend injects it as a `<link>` into the WebView so Canvas can use those fonts immediately.
3. Fetch up to `max_stylesheets` non-Google external stylesheets. Scan CSS text with regex for `font-family` declarations on heading selectors (`h1`, `h2`) and body selectors (`body`, `p`). Extract the first font name from each.
4. If steps 1–3 yield no result, inspect `style=""` attributes directly on `h1`, `h2`, and `body` elements in the HTML, extracting `font-family` values from inline style strings.
5. Return `DetectedFonts`. Frontend marks detected fonts with "✦ detected" in the picker and pre-selects them.
6. **Fallback:** if no fonts detected and no saved config: `Georgia, serif` for title; `Inter, system-ui, sans-serif` for body.

### Font Resolution Priority

1. Detected from scraped page (default — automatic)
2. User override via font pickers (per session)
3. Saved config `[style]` block (only if user previously clicked "Save style as default")

---

## Social Card Canvas

### Zone Layout (top to bottom)

| Zone | Description |
|---|---|
| **Image** | Fixed height (~45% of card). `drawImage()` with cover-fit cropping, centred. |
| **Gradient fade** | Thin linear gradient at image/text boundary — softens the hard cut. |
| **Title** | Detected heading font + configured size. Text wraps at card width minus padding. Height measured via `measureText()`. |
| **Summary** | Detected body font + configured size. Same wrap logic. |
| **Attribution** | Source domain, small text. Shown only when "Show source attribution" is checked. |

Background: `#ffffff` (white variant) / `#1a1a2e` (dark variant).  
Text colours: `#111111` / `#f0f0f0` (title), `#444444` / `#cccccc` (summary), `#aaaaaa` / `#555555` (attribution).

### Platform Presets

| Preset | Dimensions (px) | Height behaviour |
|---|---|---|
| Twitter / X Link Card | 1200 × 628 | Fixed |
| Facebook / OG | 1200 × 630 | Fixed |
| LinkedIn | 1200 × 627 | Fixed |
| Instagram Square | 1080 × 1080 | Fixed |
| Instagram Portrait | 1080 × 1350 | Fixed |
| Custom | user-defined width | Auto (grows to fit text) |

For fixed presets: if text overflows the available height, a warning badge appears on the preview. The user can enable auto-height via the "↕ Auto height" checkbox in the Style section, or shorten the text.  
Auto-height is available as an override on any preset via that same checkbox.

---

## Prompt Templating System

### Template Variables

| Variable | Source |
|---|---|
| `{{title}}` | Title field (reflects user edits) |
| `{{description}}` | Description field (reflects user edits) |
| `{{article_text}}` | Full article text field (reflects user edits) |

Variable substitution happens in Svelte before calling `generate_summary`. Unknown variables are left as-is (no silent failure).

### config.toml — Full Structure

```toml
# ~/.config/macscraper/config.toml  (Linux)
# ~/Library/Application Support/macscraper/config.toml  (macOS)
# %APPDATA%\macscraper\config.toml  (Windows)

# ── LLM connection (api_key stored in OS keychain, not here) ──────────────
[llm]
endpoint = "https://api.openai.com/v1"
model    = "gpt-4o-mini"

# ── Scraping heuristics ───────────────────────────────────────────────────
[scraping]
# Minimum character count from semantic selectors before falling back to
# density-scoring heuristic. Increase for sites with short teasers in <article>.
article_min_chars = 200

# Minimum image width (px) to qualify as the "first large image" OG fallback.
image_min_width = 300

# Max number of external CSS files fetched per page for font detection.
# Higher = better detection, slower scrape.
max_stylesheets = 5

# ── Card style defaults (written by "Save style as default") ──────────────
[style]
# Only written when user explicitly saves. Absent = use detected/system fonts.
# title_family = "Playfair Display"
# title_size   = 48
# body_family  = "Source Sans Pro"
# body_size    = 22

# ── Prompt templates (at least one required) ──────────────────────────────
# Available variables: {{title}}, {{description}}, {{article_text}}

[[prompt_templates]]
name    = "Concise teaser"
default = true
prompt  = """
Write a 2-sentence social media teaser for the following article.
Be engaging and avoid clickbait.

Title: {{title}}
Article: {{article_text}}
"""

[[prompt_templates]]
name   = "LinkedIn post"
prompt = """
Write a professional 3-sentence LinkedIn post summary.
Focus on the key insight or takeaway.

Title: {{title}}
Article: {{article_text}}
"""

[[prompt_templates]]
name   = "Twitter / X opener"
prompt = """
Write a punchy opener tweet (max 280 characters) for this article.
No hashtags.

Title: {{title}}
"""
```

Config path is resolved via Tauri's `app_config_dir()` — no hardcoded paths.

> **Naming note:** The font/style section in `config.toml` is `[style]`, not `[fonts]`. Earlier design diagrams used `[fonts]` but the TOML structure canonically uses `[style]`.

---

## Cargo.toml Dependencies

```toml
[dependencies]
tauri          = { version = "2", features = [] }
reqwest        = { version = "0.13", features = ["json", "rustls-tls"], default-features = false }
scraper        = "0.26"
async-openai   = { version = "0.31", default-features = false, features = ["chat-completion"] }
tokio          = { version = "1", features = ["full"] }
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
toml           = "0.8"
base64         = "0.22"

[build-dependencies]
tauri-build    = { version = "2", features = [] }
```

---

## Out of Scope (v1)

- LLM response streaming via Tauri events (future enhancement)
- Multiple URL batch processing
- Image editing beyond selection (crop, filter)
- Sharing directly to social platforms via API
