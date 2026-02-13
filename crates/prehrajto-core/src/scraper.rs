//! Main scraper API for prehraj.to
//!
//! Provides the high-level API combining HTTP client and parsers.

use crate::client::{ClientConfig, PrehrajtoClient};
use crate::error::{PrehrajtoError, Result};
use crate::parser::{parse_direct_url, parse_search_results};
use crate::types::VideoResult;
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
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> prehrajto_core::Result<()> {
    /// use prehrajto_core::PrehrajtoScraper;
    /// let scraper = PrehrajtoScraper::new()?;
    /// let results = scraper.search("doctor who").await?;
    /// for video in results {
    ///     println!("{}: {}", video.name, video.download_url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: &str) -> Result<Vec<VideoResult>> {
        // Validate query is not empty or whitespace
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Search query cannot be empty".to_string(),
            ));
        }

        // Build search URL with encoded query
        let search_url = build_search_url(trimmed);
        
        // Extract path from full URL for client.fetch()
        let path = search_url
            .strip_prefix("https://prehraj.to")
            .unwrap_or(&search_url);

        // Fetch and parse results
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
    ///
    /// # Example
    /// ```
    /// # fn example() -> prehrajto_core::Result<()> {
    /// use prehrajto_core::PrehrajtoScraper;
    /// let scraper = PrehrajtoScraper::new()?;
    /// let url = scraper.get_download_url("doctor-who-s07e05", "63aba7f51f6cf")?;
    /// assert_eq!(url, "https://prehraj.to/doctor-who-s07e05/63aba7f51f6cf?do=download");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_download_url(&self, video_slug: &str, video_id: &str) -> Result<String> {
        // Validate video_id is not empty
        if video_id.trim().is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Video ID cannot be empty".to_string(),
            ));
        }

        Ok(build_download_url(video_slug, video_id))
    }

    /// Get direct CDN URL for a video file
    ///
    /// Fetches the download page and extracts the actual CDN URL
    /// (premiumcdn.net) with token and expiration parameters.
    ///
    /// # Arguments
    /// * `video_slug` - URL slug of the video (e.g., "teorie-velkeho-tresku-s01e01-cz-dabing")
    /// * `video_id` - ID of the video (e.g., "5cf41ef5c543f")
    ///
    /// # Returns
    /// Direct URL to CDN (premiumcdn.net) with token and expiration
    ///
    /// # Errors
    /// - `InvalidId` if video_id is empty
    /// - `NotFound` if CDN URL cannot be found in the response
    /// - `HttpError` for network errors
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> prehrajto_core::Result<()> {
    /// use prehrajto_core::PrehrajtoScraper;
    /// let scraper = PrehrajtoScraper::new()?;
    /// let direct_url = scraper.get_direct_url("doctor-who-s07e05", "63aba7f51f6cf").await?;
    /// // Returns something like:
    /// // https://prg-c8-storage5.premiumcdn.net/13756776/...?filename=...&token=...&expires=...
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    /// The returned URL has an expiration time (expires parameter),
    /// so it cannot be cached long-term.
    pub async fn get_direct_url(&self, video_slug: &str, video_id: &str) -> Result<String> {
        // Validate video_id is not empty
        if video_id.trim().is_empty() {
            return Err(PrehrajtoError::InvalidId(
                "Video ID cannot be empty".to_string(),
            ));
        }

        // Build the download URL path
        let path = format!("/{}/{}?do=download", video_slug, video_id);

        // Fetch the download page
        let html = self.client.fetch(&path).await?;

        // Parse and extract the direct CDN URL
        parse_direct_url(&html)
    }

    /// Search for a movie by name, returning the best match
    ///
    /// Builds a query from the movie name and optional release year,
    /// then returns the first (best) matching result.
    ///
    /// # Arguments
    /// * `movie_name` - Movie title to search for
    /// * `year` - Optional release year to narrow results
    ///
    /// # Returns
    /// The best matching `VideoResult`, or `None` if no results found
    ///
    /// # Errors
    /// - `InvalidId` if movie_name is empty or whitespace only
    /// - `HttpError` if network request fails
    /// - `ParseError` if HTML parsing fails
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> prehrajto_core::Result<()> {
    /// use prehrajto_core::PrehrajtoScraper;
    /// let scraper = PrehrajtoScraper::new()?;
    /// let movie = scraper.search_movie("Inception", Some(2010)).await?;
    /// if let Some(video) = movie {
    ///     println!("{}: {}", video.name, video.download_url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// Builds a query from the movie name and optional release year,
    /// then returns all matching results.
    ///
    /// # Arguments
    /// * `movie_name` - Movie title to search for
    /// * `year` - Optional release year to narrow results
    ///
    /// # Returns
    /// Vector of matching video results, empty if no results found
    ///
    /// # Errors
    /// - `InvalidId` if movie_name is empty or whitespace only
    /// - `HttpError` if network request fails
    /// - `ParseError` if HTML parsing fails
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> prehrajto_core::Result<()> {
    /// use prehrajto_core::PrehrajtoScraper;
    /// let scraper = PrehrajtoScraper::new()?;
    /// let results = scraper.search_movie_all("Inception", Some(2010)).await?;
    /// for video in results {
    ///     println!("{}: {}", video.name, video.download_url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
