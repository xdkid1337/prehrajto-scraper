//! Prehraj.to Scraper Core Library
//!
//! Provides async API for searching videos and getting download links from prehraj.to.
//!
//! # Overview
//!
//! This crate provides a complete scraping solution for prehraj.to with:
//! - Rate-limited HTTP client to avoid overwhelming the server
//! - HTML parsers for extracting video information
//! - High-level API for searching and downloading videos
//!
//! # Example
//!
//! ```no_run
//! use prehrajto_core::{PrehrajtoScraper, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let scraper = PrehrajtoScraper::new()?;
//!     
//!     // Search for videos
//!     let results = scraper.search("doctor who").await?;
//!     for video in results {
//!         println!("{}: {}", video.name, video.download_url);
//!     }
//!     
//!     Ok(())
//! }
//! ```

mod client;
mod error;
pub mod parser;
mod scraper;
mod types;
pub mod url;

// Re-export client types
pub use client::{ClientConfig, PrehrajtoClient, RateLimiter};

// Re-export error types
pub use error::{PrehrajtoError, Result};

// Re-export parser functions
pub use parser::parse_search_results;

// Re-export main scraper API
pub use scraper::PrehrajtoScraper;

// Re-export data types
pub use types::VideoResult;

// Re-export URL helper functions for convenience
pub use url::{build_download_url, build_search_url, build_video_url, extract_video_info};
