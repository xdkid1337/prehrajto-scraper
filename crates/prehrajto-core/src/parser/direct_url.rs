//! Direct URL parser for prehraj.to
//!
//! Parses HTML from download page to extract the direct CDN URL.

use crate::error::{PrehrajtoError, Result};
use regex::Regex;
use scraper::{Html, Selector};

/// Parses download page HTML and extracts the direct CDN URL
///
/// The download page may contain the CDN URL in various places:
/// - `<a>` tag with href containing premiumcdn.net
/// - `<video>` or `<source>` element with src attribute
/// - JavaScript redirect: `window.location = "..."` or `window.location.href = "..."`
/// - Meta refresh: `<meta http-equiv="refresh" content="0;url=...">`
///
/// # Arguments
/// * `html` - Raw HTML string from download page
///
/// # Returns
/// Direct CDN URL string
///
/// # Errors
/// Returns `NotFound` if no CDN URL could be extracted
pub fn parse_direct_url(html: &str) -> Result<String> {
    // Try each extraction method in order of likelihood
    
    // 1. Look for <a> tag with premiumcdn.net href
    if let Some(url) = extract_from_anchor(html) {
        return Ok(url);
    }
    
    // 2. Look for <video> or <source> element
    if let Some(url) = extract_from_video_element(html) {
        return Ok(url);
    }
    
    // 3. Look for JavaScript redirect
    if let Some(url) = extract_from_javascript(html) {
        return Ok(url);
    }
    
    // 4. Look for meta refresh
    if let Some(url) = extract_from_meta_refresh(html) {
        return Ok(url);
    }
    
    // 5. Generic regex search for premiumcdn.net URLs
    if let Some(url) = extract_cdn_url_generic(html) {
        return Ok(url);
    }
    
    Err(PrehrajtoError::NotFound(
        "Could not find direct CDN URL in download page".to_string(),
    ))
}

/// Extracts CDN URL from anchor tags
fn extract_from_anchor(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").ok()?;
    
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href")
            && is_cdn_url(href)
        {
            return Some(decode_html_entities(href));
        }
    }
    None
}

/// Extracts CDN URL from video/source elements
fn extract_from_video_element(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    
    // Try <video src="...">
    if let Ok(selector) = Selector::parse("video[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src")
                && is_cdn_url(src)
            {
                return Some(decode_html_entities(src));
            }
        }
    }
    
    // Try <source src="...">
    if let Ok(selector) = Selector::parse("source[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src")
                && is_cdn_url(src)
            {
                return Some(decode_html_entities(src));
            }
        }
    }
    
    None
}

/// Extracts CDN URL from JavaScript redirects
fn extract_from_javascript(html: &str) -> Option<String> {
    // Match patterns like:
    // window.location = "https://..."
    // window.location.href = "https://..."
    // location.href = "https://..."
    // location = "https://..."
    let patterns = [
        r#"window\.location\.href\s*=\s*["']([^"']+premiumcdn[^"']+)["']"#,
        r#"window\.location\s*=\s*["']([^"']+premiumcdn[^"']+)["']"#,
        r#"location\.href\s*=\s*["']([^"']+premiumcdn[^"']+)["']"#,
        r#"location\s*=\s*["']([^"']+premiumcdn[^"']+)["']"#,
    ];
    
    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern)
            && let Some(caps) = re.captures(html)
            && let Some(url) = caps.get(1)
        {
            return Some(url.as_str().to_string());
        }
    }
    
    None
}

/// Extracts CDN URL from meta refresh tag
fn extract_from_meta_refresh(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"meta[http-equiv="refresh"]"#).ok()?;
    
    for element in document.select(&selector) {
        if let Some(content) = element.value().attr("content") {
            // Parse content like "0;url=https://..."
            if let Some(url_part) = content.split("url=").nth(1) {
                let url = url_part.trim();
                if is_cdn_url(url) {
                    return Some(decode_html_entities(url));
                }
            }
        }
    }
    
    None
}

/// Generic regex search for CDN URLs in HTML
fn extract_cdn_url_generic(html: &str) -> Option<String> {
    // Match any URL containing premiumcdn.net with token and expires params
    let re = Regex::new(
        r#"https?://[^"'\s<>]+premiumcdn\.net[^"'\s<>]*(?:token|expires)[^"'\s<>]*"#
    ).ok()?;
    
    if let Some(m) = re.find(html) {
        return Some(decode_html_entities(m.as_str()));
    }
    
    // Fallback: any premiumcdn.net URL
    let re_fallback = Regex::new(
        r#"https?://[^"'\s<>]+premiumcdn\.net[^"'\s<>]+"#
    ).ok()?;
    
    re_fallback.find(html).map(|m| decode_html_entities(m.as_str()))
}

/// Decodes common HTML entities in URLs
fn decode_html_entities(url: &str) -> String {
    url.replace("&amp;", "&")
       .replace("&lt;", "<")
       .replace("&gt;", ">")
       .replace("&quot;", "\"")
       .replace("&#39;", "'")
}

/// Checks if URL is a CDN URL (premiumcdn.net)
fn is_cdn_url(url: &str) -> bool {
    url.contains("premiumcdn.net") || url.contains("cdn.") && url.contains("premium")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_anchor() {
        let html = r#"
        <html>
        <body>
            <a href="https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&expires=123">Download</a>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(url.contains("premiumcdn.net"));
        assert!(url.contains("token="));
    }

    #[test]
    fn test_extract_from_video_element() {
        let html = r#"
        <html>
        <body>
            <video src="https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&expires=123"></video>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("premiumcdn.net"));
    }

    #[test]
    fn test_extract_from_source_element() {
        let html = r#"
        <html>
        <body>
            <video>
                <source src="https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&expires=123">
            </video>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("premiumcdn.net"));
    }

    #[test]
    fn test_extract_from_javascript_location() {
        let html = r#"
        <html>
        <body>
            <script>
                window.location = "https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&expires=123";
            </script>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("premiumcdn.net"));
    }

    #[test]
    fn test_extract_from_javascript_location_href() {
        let html = r#"
        <html>
        <body>
            <script>
                window.location.href = "https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&expires=123";
            </script>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("premiumcdn.net"));
    }

    #[test]
    fn test_extract_from_meta_refresh() {
        let html = r#"
        <html>
        <head>
            <meta http-equiv="refresh" content="0;url=https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&expires=123">
        </head>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("premiumcdn.net"));
    }

    #[test]
    fn test_no_cdn_url_found() {
        let html = r#"
        <html>
        <body>
            <p>No video here</p>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_is_cdn_url() {
        assert!(is_cdn_url("https://prg-c8-storage5.premiumcdn.net/123/file.mp4"));
        assert!(is_cdn_url("https://cdn.premiumcdn.net/file.mp4"));
        assert!(!is_cdn_url("https://prehraj.to/video/123"));
        assert!(!is_cdn_url("https://example.com/file.mp4"));
    }

    #[test]
    fn test_decode_html_entities() {
        let url = "https://example.com?a=1&amp;b=2&amp;c=3";
        let decoded = decode_html_entities(url);
        assert_eq!(decoded, "https://example.com?a=1&b=2&c=3");
    }

    #[test]
    fn test_extract_url_with_html_entities() {
        let html = r#"
        <html>
        <body>
            <a href="https://prg-c8-storage5.premiumcdn.net/123/file.mp4?token=abc&amp;expires=123">Download</a>
        </body>
        </html>
        "#;
        
        let result = parse_direct_url(html);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(url.contains("token=abc&expires=123"));
        assert!(!url.contains("&amp;"));
    }
}
