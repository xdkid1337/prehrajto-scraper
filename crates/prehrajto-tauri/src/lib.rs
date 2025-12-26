//! Prehraj.to Tauri Integration
//!
//! Provides Tauri plugin for frontend integration with prehraj.to scraper.
//!
//! # Usage
//!
//! Register the plugin in your Tauri application:
//!
//! ```ignore
//! fn main() {
//!     tauri::Builder::default()
//!         .plugin(prehrajto_tauri::init())
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```
//!
//! Then invoke commands from the frontend:
//!
//! ```javascript
//! import { invoke } from '@tauri-apps/api/core';
//!
//! // Search for videos
//! const results = await invoke('plugin:prehrajto|search_videos', { query: 'doctor who' });
//!
//! // Get download URL
//! const url = await invoke('plugin:prehrajto|get_download_url', {
//!   videoSlug: 'doctor-who-s07e05',
//!   videoId: '63aba7f51f6cf'
//! });
//! ```

use std::sync::Arc;
use tokio::sync::Mutex;

use prehrajto_core::PrehrajtoScraper;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;

/// Thread-safe wrapper for PrehrajtoScraper
///
/// Uses Arc<Mutex<>> for safe concurrent access from multiple Tauri commands.
/// This allows the scraper to be shared across threads while maintaining
/// rate limiting state.
///
/// # Requirements
/// - 7.2: Shared ScraperState for thread-safe access
/// - 7.4: Arc<Mutex<>> wrapper for safe concurrent access
pub struct ScraperState {
    pub(crate) scraper: Arc<Mutex<PrehrajtoScraper>>,
}

impl ScraperState {
    /// Create a new ScraperState with default configuration
    ///
    /// # Returns
    /// A new `ScraperState` instance wrapping a `PrehrajtoScraper`
    ///
    /// # Errors
    /// Returns error string if scraper initialization fails
    pub fn new() -> Result<Self, String> {
        let scraper = PrehrajtoScraper::new().map_err(|e| e.to_string())?;
        Ok(Self {
            scraper: Arc::new(Mutex::new(scraper)),
        })
    }
}

impl Default for ScraperState {
    fn default() -> Self {
        Self::new().expect("Failed to create default ScraperState")
    }
}

/// Initialize the prehrajto plugin
///
/// # Returns
/// A configured TauriPlugin ready to be registered with the Tauri application
///
/// # Example
/// ```ignore
/// tauri::Builder::default()
///     .plugin(prehrajto_tauri::init())
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("prehrajto")
        .invoke_handler(tauri::generate_handler![
            commands::search_videos,
            commands::get_download_url
        ])
        .setup(|app, _api| {
            let state = ScraperState::new().map_err(Box::<dyn std::error::Error>::from)?;
            app.manage(state);
            Ok(())
        })
        .build()
}

// Re-export types for convenience
pub use prehrajto_core::VideoResult as Video;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_state_creation() {
        let state = ScraperState::new();
        assert!(state.is_ok());
    }

    #[test]
    fn test_scraper_state_default() {
        let state = ScraperState::default();
        assert!(state.scraper.try_lock().is_ok());
    }
}
