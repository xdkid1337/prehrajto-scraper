//! URL helper functions for prehraj.to
//!
//! Provides functions for building video, download, and search URLs.

const BASE_URL: &str = "https://prehraj.to";

/// Builds the full video page URL from slug and ID
///
/// # Arguments
/// * `slug` - URL-friendly video slug (e.g., "doctor-who-s07e05-andele-dobyvaji-manhattan")
/// * `id` - Unique video ID (e.g., "63aba7f51f6cf")
///
/// # Returns
/// Full URL to the video page
///
/// # Example
/// ```
/// use prehrajto_core::url::build_video_url;
/// let url = build_video_url("test-video", "abc123");
/// assert_eq!(url, "https://prehraj.to/test-video/abc123");
/// ```
pub fn build_video_url(slug: &str, id: &str) -> String {
    format!("{}/{}/{}", BASE_URL, slug, id)
}

/// Builds the download URL from slug and ID
///
/// Appends `?do=download` to the video URL as per prehraj.to format.
///
/// # Arguments
/// * `slug` - URL-friendly video slug
/// * `id` - Unique video ID
///
/// # Returns
/// Full download URL with query parameter
///
/// # Example
/// ```
/// use prehrajto_core::url::build_download_url;
/// let url = build_download_url("test-video", "abc123");
/// assert_eq!(url, "https://prehraj.to/test-video/abc123?do=download");
/// ```
pub fn build_download_url(slug: &str, id: &str) -> String {
    format!("{}?do=download", build_video_url(slug, id))
}

/// Builds the search URL for a given query
///
/// URL encodes the query and constructs the search URL.
///
/// # Arguments
/// * `query` - Search query string
///
/// # Returns
/// Full search URL with encoded query
///
/// # Example
/// ```
/// use prehrajto_core::url::build_search_url;
/// let url = build_search_url("doctor who");
/// assert_eq!(url, "https://prehraj.to/hledej/doctor%20who");
/// ```
pub fn build_search_url(query: &str) -> String {
    let encoded = urlencoding::encode(query);
    format!("{}/hledej/{}", BASE_URL, encoded)
}

/// Extracts video slug and ID from a URL path
///
/// Parses URLs in format `/{slug}/{id}` and returns both components.
///
/// # Arguments
/// * `url` - URL string or path (e.g., "/test-video/abc123" or "https://prehraj.to/test-video/abc123")
///
/// # Returns
/// `Some((slug, id))` if parsing succeeds, `None` otherwise
///
/// # Example
/// ```
/// use prehrajto_core::url::extract_video_info;
/// let info = extract_video_info("/doctor-who/63aba7f51f6cf");
/// assert_eq!(info, Some(("doctor-who".to_string(), "63aba7f51f6cf".to_string())));
/// ```
pub fn extract_video_info(url: &str) -> Option<(String, String)> {
    // Remove base URL if present
    let path = url
        .strip_prefix(BASE_URL)
        .unwrap_or(url);
    
    // Remove leading slash and any query parameters
    let path = path.trim_start_matches('/');
    let path = path.split('?').next().unwrap_or(path);
    
    // Split by '/' and get slug and id
    let parts: Vec<&str> = path.split('/').collect();
    
    if parts.len() >= 2 {
        let slug = parts[0];
        let id = parts[1];
        
        // Validate that both are non-empty
        if !slug.is_empty() && !id.is_empty() {
            return Some((slug.to_string(), id.to_string()));
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_video_url() {
        let url = build_video_url("doctor-who-s07e05", "63aba7f51f6cf");
        assert_eq!(url, "https://prehraj.to/doctor-who-s07e05/63aba7f51f6cf");
    }

    #[test]
    fn test_build_download_url() {
        let url = build_download_url("doctor-who-s07e05", "63aba7f51f6cf");
        assert_eq!(url, "https://prehraj.to/doctor-who-s07e05/63aba7f51f6cf?do=download");
    }

    #[test]
    fn test_build_search_url_simple() {
        let url = build_search_url("doctor");
        assert_eq!(url, "https://prehraj.to/hledej/doctor");
    }

    #[test]
    fn test_build_search_url_with_spaces() {
        let url = build_search_url("doctor who s07e05");
        assert_eq!(url, "https://prehraj.to/hledej/doctor%20who%20s07e05");
    }

    #[test]
    fn test_extract_video_info_from_path() {
        let info = extract_video_info("/doctor-who/63aba7f51f6cf");
        assert_eq!(info, Some(("doctor-who".to_string(), "63aba7f51f6cf".to_string())));
    }

    #[test]
    fn test_extract_video_info_from_full_url() {
        let info = extract_video_info("https://prehraj.to/doctor-who/63aba7f51f6cf");
        assert_eq!(info, Some(("doctor-who".to_string(), "63aba7f51f6cf".to_string())));
    }

    #[test]
    fn test_extract_video_info_with_query_params() {
        let info = extract_video_info("/doctor-who/63aba7f51f6cf?do=download");
        assert_eq!(info, Some(("doctor-who".to_string(), "63aba7f51f6cf".to_string())));
    }

    #[test]
    fn test_extract_video_info_invalid_single_part() {
        let info = extract_video_info("/only-slug");
        assert_eq!(info, None);
    }

    #[test]
    fn test_extract_video_info_empty_parts() {
        let info = extract_video_info("//");
        assert_eq!(info, None);
    }
}
