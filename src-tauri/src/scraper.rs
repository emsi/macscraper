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

/// Extract title using priority: og:title → twitter:title → <title> → first <h1>
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

/// Extract description using priority: og:description → twitter:description → meta[name=description]
pub fn extract_description(doc: &Html) -> String {
    meta_property(doc, "og:description")
        .or_else(|| meta_name(doc, "twitter:description"))
        .or_else(|| meta_name(doc, "description"))
        .unwrap_or_default()
}

/// Extract OG image URL: og:image → twitter:image
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

/// Collect all <img> elements, resolved to absolute URLs.
/// Images with width >= min_width are sorted before smaller images.
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
    images.sort_by_key(|img| {
        let w = img.width.unwrap_or(0);
        if w >= min_width { 0u8 } else { 1u8 }
    });
    images
}

const SEMANTIC_SELECTORS: &[&str] = &[
    "article", "main", "[role=main]", ".post-content", ".entry-content",
];

/// Extract article text using two-pass approach:
/// Pass 1: semantic selectors (article, main, etc.) — take longest match.
/// Pass 2: density fallback if Pass 1 result is shorter than min_chars.
pub fn extract_article_text(doc: &Html, min_chars: usize) -> String {
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

/// Synchronously extract all data from a parsed HTML document.
/// Returns a tuple of (title, description, og_image, all_images, article_text,
/// stylesheet_urls, inline_css, heading_inline, body_inline).
fn extract_all(
    doc: &Html,
    base_url: &str,
    image_min_width: u32,
    article_min_chars: usize,
) -> (String, String, Option<String>, Vec<ImageMeta>, String, Vec<String>, String, Option<String>, Option<String>) {
    let title = extract_title(doc);
    let description = extract_description(doc);
    let og_image = extract_og_image(doc);
    let all_images = collect_all_images(doc, base_url, image_min_width);
    let article_text = extract_article_text(doc, article_min_chars);
    let stylesheet_urls = crate::fonts::collect_stylesheet_urls(doc, base_url);
    let inline_css = {
        let sel = Selector::parse("style").unwrap();
        doc.select(&sel).map(|el| el.text().collect::<String>()).collect::<Vec<_>>().join("\n")
    };
    let heading_inline = crate::fonts::extract_inline_style_font(doc, &["h1", "h2"]);
    let body_inline = crate::fonts::extract_inline_style_font(doc, &["body"]);
    (title, description, og_image, all_images, article_text, stylesheet_urls, inline_css, heading_inline, body_inline)
}

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

    let (title, description, og_image, all_images, article_text, stylesheet_urls, inline_css, heading_inline, body_inline) = {
        let doc = Html::parse_document(&html);
        extract_all(&doc, &url, scraping_config.image_min_width, scraping_config.article_min_chars)
    };

    let detected_fonts = crate::fonts::detect_fonts_from_parts(
        stylesheet_urls,
        inline_css,
        heading_inline,
        body_inline,
        scraping_config.max_stylesheets,
    ).await;

    Ok(ScrapedData { title, description, og_image, all_images, article_text, detected_fonts })
}

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
        let doc = parse(r#"<html><body>
          <div class="sidebar"><a>link</a><a>link</a><a>link</a></div>
          <div class="content"><p>This is a long paragraph with substantial text content that should win the density score because it has a high text to tag ratio and provides the actual article body.</p></div>
        </body></html>"#);
        let text = extract_article_text(&doc, 200);
        assert!(text.contains("high text to tag ratio"));
    }
}
