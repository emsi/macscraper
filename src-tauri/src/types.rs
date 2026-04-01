// src-tauri/src/types.rs
use serde::{Deserialize, Serialize};

/// Aggregated data scraped from a blog URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedData {
    pub title: String,
    pub description: String,
    pub og_image: Option<String>,
    pub all_images: Vec<ImageMeta>,
    pub article_text: String,
    pub detected_fonts: DetectedFonts,
}

/// Metadata for a single image found on the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMeta {
    pub src: String,
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Font families detected from the page's CSS.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectedFonts {
    pub heading_family: Option<String>,
    pub body_family: Option<String>,
    pub google_fonts_url: Option<String>,
}
