//! Error types for prehraj.to scraper
//!
//! Provides a comprehensive error enum with human-readable messages
//! and Tauri-compatible serialization.

use serde::{Serialize, Serializer};
use thiserror::Error;

/// Error type for all prehraj.to scraper operations
///
/// Implements Display for human-readable messages and Serialize
/// for Tauri command compatibility.
#[derive(Error, Debug)]
pub enum PrehrajtoError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse HTML content
    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    /// Expected HTML element was not found
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Rate limited by server (HTTP 429)
    #[error("Rate limited - too many requests")]
    RateLimited,

    /// Video not found on server
    #[error("Video not found: {0}")]
    NotFound(String),

    /// Invalid video ID provided
    #[error("Invalid video ID: {0}")]
    InvalidId(String),
}

impl Serialize for PrehrajtoError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result type alias for prehraj.to operations
pub type Result<T> = std::result::Result<T, PrehrajtoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_http_error() {
        // We can't easily create a reqwest::Error, so we test other variants
        let error = PrehrajtoError::ParseError("invalid HTML".to_string());
        assert_eq!(error.to_string(), "Failed to parse HTML: invalid HTML");
    }

    #[test]
    fn test_error_display_parse_error() {
        let error = PrehrajtoError::ParseError("missing element".to_string());
        assert_eq!(error.to_string(), "Failed to parse HTML: missing element");
    }

    #[test]
    fn test_error_display_element_not_found() {
        let error = PrehrajtoError::ElementNotFound("video-card".to_string());
        assert_eq!(error.to_string(), "Element not found: video-card");
    }

    #[test]
    fn test_error_display_invalid_url() {
        let error = PrehrajtoError::InvalidUrl("not-a-url".to_string());
        assert_eq!(error.to_string(), "Invalid URL: not-a-url");
    }

    #[test]
    fn test_error_display_rate_limited() {
        let error = PrehrajtoError::RateLimited;
        assert_eq!(error.to_string(), "Rate limited - too many requests");
    }

    #[test]
    fn test_error_display_not_found() {
        let error = PrehrajtoError::NotFound("abc123".to_string());
        assert_eq!(error.to_string(), "Video not found: abc123");
    }

    #[test]
    fn test_error_display_invalid_id() {
        let error = PrehrajtoError::InvalidId("".to_string());
        assert_eq!(error.to_string(), "Invalid video ID: ");
    }

    #[test]
    fn test_error_serialize() {
        let error = PrehrajtoError::RateLimited;
        let json = serde_json::to_string(&error).expect("Serialization should succeed");
        assert_eq!(json, "\"Rate limited - too many requests\"");
    }

    #[test]
    fn test_error_serialize_with_message() {
        let error = PrehrajtoError::NotFound("video123".to_string());
        let json = serde_json::to_string(&error).expect("Serialization should succeed");
        assert_eq!(json, "\"Video not found: video123\"");
    }
}
