//! HTTP client with rate limiting and retry logic for prehraj.to
//!
//! Provides a rate-limited HTTP client that respects server limits
//! and implements exponential backoff for transient errors.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::error::{PrehrajtoError, Result};

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Maximum requests per second (default: 2.0)
    pub requests_per_second: f64,
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// Maximum retry attempts for transient errors (default: 3)
    pub max_retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 2.0,
            timeout_secs: 30,
            max_retries: 3,
        }
    }
}

/// Rate limiter to control request frequency
///
/// Ensures requests are spaced at least `min_interval` apart.
pub struct RateLimiter {
    min_interval: Duration,
    last_request: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified requests per second
    ///
    /// # Arguments
    /// * `requests_per_second` - Maximum number of requests allowed per second
    pub fn new(requests_per_second: f64) -> Self {
        let min_interval = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            min_interval,
            last_request: Arc::new(Mutex::new(Instant::now() - min_interval)),
        }
    }

    /// Acquire permission to make a request
    ///
    /// If called before the minimum interval has passed since the last request,
    /// this method will sleep until the interval has elapsed.
    pub async fn acquire(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            sleep(wait_time).await;
        }

        *last = Instant::now();
    }

    /// Get the minimum interval between requests
    pub fn min_interval(&self) -> Duration {
        self.min_interval
    }
}


const BASE_URL: &str = "https://prehraj.to";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// HTTP client wrapper with rate limiting and retry logic
///
/// Handles all HTTP communication with prehraj.to, including:
/// - Rate limiting to avoid overwhelming the server
/// - Automatic retries with exponential backoff for transient errors
/// - Proper headers (User-Agent, Accept-Language)
pub struct PrehrajtoClient {
    client: reqwest::Client,
    rate_limiter: RateLimiter,
    max_retries: u32,
}

impl PrehrajtoClient {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(USER_AGENT)
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT_LANGUAGE,
                    "cs-CZ,cs;q=0.9,en;q=0.8".parse().unwrap(),
                );
                headers
            })
            .build()
            .map_err(PrehrajtoError::HttpError)?;

        Ok(Self {
            client,
            rate_limiter: RateLimiter::new(config.requests_per_second),
            max_retries: config.max_retries,
        })
    }

    /// Fetch HTML content from a path on prehraj.to
    ///
    /// Automatically follows redirects for non-CDN URLs (normal page navigation).
    ///
    /// # Arguments
    /// * `path` - The path to fetch (e.g., "/search?q=test")
    ///
    /// # Returns
    /// The HTML content as a string, or an error if the request fails
    ///
    /// # Errors
    /// - `HttpError` - Network or HTTP errors
    /// - `RateLimited` - Server returned 429 after all retries exhausted
    pub async fn fetch(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", BASE_URL, path);
        self.fetch_with_retry(&url).await
    }

    /// Internal method to fetch with retry logic
    async fn fetch_with_retry(&self, url: &str) -> Result<String> {
        let mut last_error: Option<PrehrajtoError> = None;
        let mut attempt = 0;

        while attempt <= self.max_retries {
            // Wait for rate limiter
            self.rate_limiter.acquire().await;

            match self.do_fetch(url).await {
                Ok(body) => return Ok(body),
                Err(e) => {
                    if Self::is_retryable(&e) && attempt < self.max_retries {
                        // Exponential backoff: 1s, 2s, 4s
                        let backoff = Duration::from_secs(1 << attempt);
                        tokio::time::sleep(backoff).await;
                        last_error = Some(e);
                        attempt += 1;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(PrehrajtoError::RateLimited))
    }

    /// Perform a single fetch attempt with manual redirect following
    ///
    /// Follows redirects for same-site URLs but stops for CDN URLs
    /// to prevent accidentally downloading large binary files.
    async fn do_fetch(&self, url: &str) -> Result<String> {
        let mut current_url = url.to_string();
        let max_redirects = 5;

        for _ in 0..max_redirects {
            let response = self
                .client
                .get(&current_url)
                .send()
                .await
                .map_err(PrehrajtoError::HttpError)?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(PrehrajtoError::RateLimited);
            }

            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(PrehrajtoError::NotFound(current_url));
            }

            if status.is_server_error() {
                return Err(PrehrajtoError::HttpError(
                    response.error_for_status().unwrap_err(),
                ));
            }

            // Handle redirects manually — follow only non-CDN redirects
            if status.is_redirection() {
                if let Some(location) = response.headers().get(reqwest::header::LOCATION)
                    && let Ok(loc_str) = location.to_str()
                {
                    // Don't follow redirects to CDN (would download binary files)
                    if loc_str.contains("premiumcdn.net") {
                        return response.text().await.map_err(PrehrajtoError::HttpError);
                    }
                    current_url = loc_str.to_string();
                    continue;
                }
                // No Location header or can't parse — return the body as-is
                return response.text().await.map_err(PrehrajtoError::HttpError);
            }

            return response.text().await.map_err(PrehrajtoError::HttpError);
        }

        Err(PrehrajtoError::ParseError(
            "Too many redirects".to_string(),
        ))
    }

    /// Fetch a download page without following redirects
    ///
    /// The `?do=download` page returns 302 with an HTML body containing
    /// the CDN link. This uses the main cookie-bearing client but does
    /// NOT follow any redirects — returns the response body as-is.
    pub async fn fetch_download_page(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", BASE_URL, path);

        self.rate_limiter.acquire().await;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(PrehrajtoError::HttpError)?;

        response.text().await.map_err(PrehrajtoError::HttpError)
    }

    /// Check if an error is retryable
    fn is_retryable(error: &PrehrajtoError) -> bool {
        match error {
            PrehrajtoError::RateLimited => true,
            PrehrajtoError::HttpError(e) => {
                // Retry on timeout, connection errors, or 5xx status codes
                e.is_timeout()
                    || e.is_connect()
                    || e.status()
                        .map(|s| s.is_server_error())
                        .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Get a reference to the rate limiter (for testing)
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(2.0);
        assert_eq!(limiter.min_interval(), Duration::from_millis(500));
    }

    #[test]
    fn test_rate_limiter_interval_calculation() {
        let limiter = RateLimiter::new(4.0);
        assert_eq!(limiter.min_interval(), Duration::from_millis(250));
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.requests_per_second, 2.0);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_client_creation() {
        let client = PrehrajtoClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_custom_config() {
        let config = ClientConfig {
            requests_per_second: 1.0,
            timeout_secs: 60,
            max_retries: 5,
        };
        let client = PrehrajtoClient::with_config(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(10.0); // 100ms interval
        
        let start = Instant::now();
        limiter.acquire().await;
        limiter.acquire().await;
        let elapsed = start.elapsed();

        // Second acquire should wait at least 100ms
        assert!(elapsed >= Duration::from_millis(90)); // Allow small tolerance
    }
}
