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
#[allow(dead_code)]
pub async fn detect_fonts(
    doc: &Html,
    base_url: &str,
    max_stylesheets: usize,
) -> crate::types::DetectedFonts {
    let urls = collect_stylesheet_urls(doc, base_url);
    let inline_css: String = {
        let sel = Selector::parse("style").unwrap();
        doc.select(&sel).map(|el| el.text().collect::<String>()).collect::<Vec<_>>().join("\n")
    };
    let heading_inline = extract_inline_style_font(doc, &["h1", "h2"]);
    let body_inline = extract_inline_style_font(doc, &["body"]);
    detect_fonts_from_parts(urls, inline_css, heading_inline, body_inline, max_stylesheets).await
}

/// Perform font detection from pre-extracted parts (stylesheet URLs, inline CSS, inline style fallbacks).
/// Fetches up to `max_stylesheets` external (non-Google) stylesheets.
///
/// :param stylesheet_urls: List of resolved stylesheet URLs from the page.
/// :param inline_css: Combined inline <style> block content.
/// :param heading_inline: Fallback font from heading element inline style attribute.
/// :param body_inline: Fallback font from body element inline style attribute.
/// :param max_stylesheets: Maximum number of external stylesheets to fetch.
/// :return: Detected font families and Google Fonts URL.
pub async fn detect_fonts_from_parts(
    stylesheet_urls: Vec<String>,
    inline_css: String,
    heading_inline: Option<String>,
    body_inline: Option<String>,
    max_stylesheets: usize,
) -> crate::types::DetectedFonts {
    let google_fonts_url = find_google_fonts_url(&stylesheet_urls);

    let client = reqwest::Client::builder().user_agent("macscraper/0.1").build().ok();
    let mut external_css = String::new();
    if let Some(client) = client {
        let external_urls: Vec<_> = stylesheet_urls.iter()
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
        .or(heading_inline);

    let body_family = extract_font_for_selectors(&all_css, &["body", "p"])
        .or(body_inline);

    crate::types::DetectedFonts { heading_family, body_family, google_fonts_url }
}

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
