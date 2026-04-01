# macscraper

A local-only desktop app that turns any blog post URL into polished social media card images — white and dark variants — ready to download.

Paste a URL, scrape the page, tweak the title and description, optionally generate an AI summary, and export PNG cards sized for Twitter/X, LinkedIn, Facebook, or Instagram.

![macscraper screenshot placeholder](docs/screenshot.png)

---

## What it does

1. **Scrapes** a blog post URL — extracts OG/Twitter metadata, all images, the article body, and the page's font choices.
2. **Lets you edit** title, description, image selection, and fonts before anything is drawn.
3. **Optionally generates** a short social media summary with a configurable LLM prompt (OpenAI-compatible endpoint).
4. **Renders** white and dark social card images live in the browser canvas as you type.
5. **Downloads** full-resolution PNGs for whichever platform preset you choose.

Everything runs locally — no cloud service, no analytics, no login.

---

## Requirements

| Tool | Version |
|------|---------|
| Rust (stable) | 1.76+ |
| Node.js | 18+ |
| Tauri CLI | 2.x |

### Linux system dependencies

```bash
sudo apt install \
  libsecret-1-dev pkg-config \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev patchelf
```

### Install Tauri CLI (if not already installed)

```bash
cargo install tauri-cli --locked
```

---

## Running in development

```bash
# Install JS dependencies
npm install

# Start Tauri dev mode (opens a desktop window)
npm run tauri dev
```

The Rust backend compiles on first launch — this takes a minute. Subsequent runs are fast.

---

## Building for production

```bash
npm run tauri build
```

The binary and installer are placed in `src-tauri/target/release/`.

---

## Configuration

macscraper stores its config at:

| OS | Path |
|----|------|
| Linux | `~/.config/macscraper/config.toml` |
| macOS | `~/Library/Application Support/macscraper/config.toml` |
| Windows | `%APPDATA%\macscraper\config.toml` |

Click **⚙ → Open config file** in the app to open it in your default editor. A default config is created on first launch.

### config.toml structure

```toml
# ── LLM connection (API key is stored in the OS keychain, not here) ──
[llm]
endpoint = "https://api.openai.com/v1"
model    = "gpt-4o-mini"

# ── Scraping heuristics ──
[scraping]
article_min_chars = 200   # min chars before falling back to density heuristic
image_min_width   = 300   # min image width (px) to qualify as OG fallback
max_stylesheets   = 5     # max external CSS files fetched for font detection

# ── Card style defaults (written by "Save style as default") ──
[style]
# title_family = "Playfair Display"
# title_size   = 48
# body_family  = "Source Sans Pro"
# body_size    = 22

# ── Prompt templates ──
[[prompt_templates]]
name    = "Concise teaser"
default = true
prompt  = """
Write a 2-sentence social media teaser for the following article.
Be engaging and avoid clickbait.

Title: {{title}}
Article: {{article_text}}
"""
```

#### Prompt template variables

| Variable | Source |
|----------|--------|
| `{{title}}` | Title field (reflects any edits) |
| `{{description}}` | Description field |
| `{{article_text}}` | Full article text field |

---

## How to use it

1. **Paste a URL** into the bar at the top and press **Scrape** (or Enter).
2. In the **🖼 Content** section:
   - Pick the card image from the thumbnail row (or click **+disk** to upload from disk).
   - Edit the title and description freely.
   - Expand **Full Article Text** to edit the text that gets sent to the AI.
3. In the **🤖 Summary** section:
   - Toggle **Scraped** to use the scraped description as-is, or **AI Generate** to call the LLM.
   - Pick a prompt template, edit the prompt, and click **Generate ✨**.
   - The result lands in the description field and is fully editable.
   - Click **Save as template…** to keep an edited prompt for next time.
4. In the **🎨 Style** section:
   - Adjust title and body fonts (detected page fonts appear first, marked **✦**).
   - Choose a platform preset or enter a custom width.
   - Check **↕ Auto height** to let the card grow to fit all text.
   - Check **Show source attribution** to add the domain at the bottom.
5. Click **⬇ Download PNG** under either the white or dark card preview.

---

## LLM / AI setup

macscraper works with any OpenAI-compatible endpoint:

- **OpenAI**: set endpoint to `https://api.openai.com/v1`, enter your API key in Settings.
- **Ollama (local)**: set endpoint to `http://localhost:11434/v1`, model to `llama3` (or whichever you have), leave API key blank.
- **Other providers**: set the endpoint to their OpenAI-compatible base URL.

The API key is stored in the OS keychain (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows) — never written to disk.

---

## Platform presets

| Preset | Dimensions |
|--------|-----------|
| Twitter / X | 1200 × 628 |
| Facebook / OG | 1200 × 630 |
| LinkedIn | 1200 × 627 |
| Instagram Square | 1080 × 1080 |
| Instagram Portrait | 1080 × 1350 |
| Custom | user-defined width, auto height |

---

## Running tests

```bash
# Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# TypeScript unit tests
npm run test
```

---

## License

MIT
