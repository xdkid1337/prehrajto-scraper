//! Main scraper API for prehraj.to
//!
//! Provides the high-level API combining HTTP client and parsers.

use crate::client::{ClientConfig, PrehrajtoClient};
use crate::error::{PrehrajtoError, Result};
use crate::parser::{
    parse_direct_url, parse_original_download_url, parse_subtitle_tracks, parse_video_sources,
};
use crate::parser::parse_search_results;
use crate::types::{SubtitleTrack, VideoPageData, VideoResult, VideoSource};
use crate::url::{build_download_url, build_search_url};

/// Main scraper API for prehraj.to
///
/// Combines HTTP client with rate limiting and HTML parsers
/// to provide a simple interface for searching videos and
/// getting download URLs.
pub struct PrehrajtoScraper {
    client: PrehrajtoClient,
}

impl PrehrajtoScraper {
    /// Create a new scraper with default configuration
    ///
    /// # Returns
    /// A new `PrehrajtoScraper` instance
    ///
    /// # Errors
    /// Returns error if HTTP client initialization fails
    pub fn new() -> Result<Self> {
        let client = PrehrajtoClient::new()?;
        Ok(Self { client })
    }

    /// Create a new scraper with custom client configuration
    ///
    /// # Arguments
    /// * `config` - Custom client configuration
    ///
    /// # Returns
    /// A new `PrehrajtoScraper` instance
    ///
    /// # Errors
    /// Returns error if HTTP client initialization fails
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let client = PrehrajtoClient::with_config(config)?;
        Ok(Self { client })
    }

    /// Search for videos by query
    ///
    /// # Arguments
    /// * `query` - Search query string
    ///
    /// # Returns
    /// Vector of matching video results, empty if no results found
    ///
    /// # Errors
    /// - `InvalidId` if query is empty or whitespace only
    /// - `HttpError` if network request fails
    /// - `ParseError` if HTML parsing fails
    pub async fn search(&self, query: &str) -> Result<Vec<VideoResult>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Search query cannot be empty".to_string(),
            ));
        }

        let search_url = build_search_url(trimmed);
        let path = search_url
            .strip_prefix("https://prehraj.to")
            .unwrap_or(&search_url);

        let html = self.client.fetch(path).await?;
        parse_search_results(&html)
    }

    /// Get download URL for a video
    ///
    /// # Arguments
    /// * `video_slug` - URL-friendly video slug
    /// * `video_id` - Unique video ID
    ///
    /// # Returns
    /// Download URL with `?do=download` parameter
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    pub fn get_download_url(&self, video_slug: &str, video_id: &str) -> Result<String> {
        if video_id.trim().is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Video ID cannot be empty".to_string(),
            ));
        }

        Ok(build_download_url(video_slug, video_id))
    }

    /// Get direct CDN URL for a video file (best quality)
    ///
    /// Fetches the video page and extracts the highest quality CDN URL
    /// from the player initialization blocks.
    ///
    /// # Arguments
    /// * `video_slug` - URL slug of the video
    /// * `video_id` - ID of the video
    ///
    /// # Returns
    /// Direct URL to CDN (premiumcdn.net) — highest resolution available
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    /// - `NotFound` if CDN URL cannot be found in the response
    /// - `HttpError` for network errors
    ///
    /// # Note
    /// The returned URL has an expiration time (expires parameter),
    /// so it cannot be cached long-term.
    pub async fn get_direct_url(&self, video_slug: &str, video_id: &str) -> Result<String> {
        if video_id.trim().is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Video ID cannot be empty".to_string(),
            ));
        }

        // Fetch the video page (NOT ?do=download) to get player sources
        let path = format!("/{}/{}", video_slug, video_id);
        let html = self.client.fetch(&path).await?;

        parse_direct_url(&html)
    }

    /// Get all streaming quality variants for a video
    ///
    /// Fetches the video page and parses JS player sources to extract
    /// all available quality variants (e.g., 720p, 1080p).
    ///
    /// # Arguments
    /// * `video_slug` - URL slug of the video
    /// * `video_id` - ID of the video
    ///
    /// # Returns
    /// Vector of [`VideoSource`] with all available qualities
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    /// - `HttpError` for network errors
    pub async fn get_video_sources(
        &self,
        video_slug: &str,
        video_id: &str,
    ) -> Result<Vec<VideoSource>> {
        let data = self.get_video_page_data(video_slug, video_id).await?;
        Ok(data.sources)
    }

    /// Get all streaming sources AND subtitle tracks for a video
    ///
    /// Fetches the video page **once** and parses both JS sources and
    /// tracks arrays, avoiding double-fetching.
    ///
    /// # Arguments
    /// * `video_slug` - URL slug of the video
    /// * `video_id` - ID of the video
    ///
    /// # Returns
    /// [`VideoPageData`] with sources and subtitles
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    /// - `HttpError` for network errors
    pub async fn get_video_page_data(
        &self,
        video_slug: &str,
        video_id: &str,
    ) -> Result<VideoPageData> {
        if video_id.trim().is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Video ID cannot be empty".to_string(),
            ));
        }

        let path = format!("/{}/{}", video_slug, video_id);
        let html = self.client.fetch(&path).await?;

        Ok(VideoPageData {
            sources: parse_video_sources(&html),
            subtitles: parse_subtitle_tracks(&html),
        })
    }

    /// Get subtitle tracks for a video
    ///
    /// Convenience method — fetches the video page and extracts subtitle tracks.
    ///
    /// # Arguments
    /// * `video_slug` - URL slug of the video
    /// * `video_id` - ID of the video
    ///
    /// # Returns
    /// Vector of [`SubtitleTrack`] (empty if no subtitles available)
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    /// - `HttpError` for network errors
    pub async fn get_subtitle_tracks(
        &self,
        video_slug: &str,
        video_id: &str,
    ) -> Result<Vec<SubtitleTrack>> {
        let data = self.get_video_page_data(video_slug, video_id).await?;
        Ok(data.subtitles)
    }

    /// Get the original uploaded file URL via download flow
    ///
    /// Performs a two-step cookie flow:
    /// 1. GET video page — sets required cookies (`_nss`, `u_uid`)
    /// 2. GET `?do=download` with cookies — returns redirect page with original file link
    ///
    /// # Arguments
    /// * `video_slug` - URL slug of the video
    /// * `video_id` - ID of the video
    ///
    /// # Returns
    /// A [`VideoSource`] representing the original uploaded file
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    /// - `NotFound` if original file URL cannot be found
    /// - `HttpError` for network errors
    pub async fn get_original_url(
        &self,
        video_slug: &str,
        video_id: &str,
    ) -> Result<VideoSource> {
        if video_id.trim().is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Video ID cannot be empty".to_string(),
            ));
        }

        // Step 1: Fetch video page to set cookies (_nss, u_uid)
        let video_path = format!("/{}/{}", video_slug, video_id);
        let _ = self.client.fetch(&video_path).await?;

        // Step 2: Fetch download page with cookies (no redirect following)
        let download_path = format!("/{}/{}?do=download", video_slug, video_id);
        let html = self.client.fetch_download_page(&download_path).await?;

        parse_original_download_url(&html)
    }

    /// Search for a movie by name, returning the best match
    ///
    /// # Arguments
    /// * `movie_name` - Movie title to search for
    /// * `year` - Optional release year to narrow results
    ///
    /// # Returns
    /// The best matching `VideoResult`, or `None` if no results found
    pub async fn search_movie(
        &self,
        movie_name: &str,
        year: Option<i32>,
    ) -> Result<Option<VideoResult>> {
        let results = self.search_movie_all(movie_name, year).await?;
        Ok(results.into_iter().next())
    }

    /// Search for all movie sources by name
    ///
    /// # Arguments
    /// * `movie_name` - Movie title to search for
    /// * `year` - Optional release year to narrow results
    ///
    /// # Returns
    /// Vector of matching video results, empty if no results found
    pub async fn search_movie_all(
        &self,
        movie_name: &str,
        year: Option<i32>,
    ) -> Result<Vec<VideoResult>> {
        let trimmed = movie_name.trim();
        if trimmed.is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Movie name cannot be empty".to_string(),
            ));
        }

        let query = match year {
            Some(y) => format!("{} {}", trimmed, y),
            None => trimmed.to_string(),
        };

        self.search(&query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_creation() {
        let scraper = PrehrajtoScraper::new();
        assert!(scraper.is_ok());
    }

    #[test]
    fn test_scraper_with_custom_config() {
        let config = ClientConfig {
            requests_per_second: 1.0,
            timeout_secs: 60,
            max_retries: 5,
        };
        let scraper = PrehrajtoScraper::with_config(config);
        assert!(scraper.is_ok());
    }

    #[test]
    fn test_get_download_url_valid() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let url = scraper.get_download_url("doctor-who-s07e05", "63aba7f51f6cf");
        assert!(url.is_ok());
        assert_eq!(
            url.unwrap(),
            "https://prehraj.to/doctor-who-s07e05/63aba7f51f6cf?do=download"
        );
    }

    #[test]
    fn test_get_download_url_empty_id() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.get_download_url("some-slug", "");
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[test]
    fn test_get_download_url_whitespace_id() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.get_download_url("some-slug", "   ");
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(_)) => {}
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.search("").await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_search_whitespace_query() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.search("   ").await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(_)) => {}
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_get_direct_url_empty_id() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.get_direct_url("some-slug", "").await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_get_direct_url_whitespace_id() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.get_direct_url("some-slug", "   ").await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(_)) => {}
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_search_movie_empty_name() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.search_movie("", None).await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_search_movie_whitespace_name() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.search_movie("   ", Some(2020)).await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(_)) => {}
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_search_movie_all_empty_name() {
        let scraper = PrehrajtoScraper::new().unwrap();
        let result = scraper.search_movie_all("", Some(2020)).await;
        assert!(result.is_err());
        match result {
            Err(PrehrajtoError::InvalidId(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidId error"),
        }
    }
}
