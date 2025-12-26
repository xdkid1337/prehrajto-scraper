//! Tauri commands for prehraj.to scraper
//!
//! This module contains all Tauri command implementations.

use prehrajto_core::VideoResult;
use tauri::State;

use crate::ScraperState;

/// Search for videos on prehraj.to
///
/// # Arguments
/// * `state` - Managed ScraperState from Tauri
/// * `query` - Search query string
///
/// # Returns
/// Vector of matching video results
///
/// # Errors
/// Returns error message as String if search fails
///
/// # Requirements
/// - 7.1: Exposes search_videos command
/// - 7.3: Returns error message as String on failure
#[tauri::command]
pub async fn search_videos(
    state: State<'_, ScraperState>,
    query: String,
) -> Result<Vec<VideoResult>, String> {
    let scraper = state.scraper.lock().await;
    scraper.search(&query).await.map_err(|e| e.to_string())
}

/// Get download URL for a video
///
/// # Arguments
/// * `state` - Managed ScraperState from Tauri
/// * `video_slug` - URL-friendly video slug
/// * `video_id` - Unique video ID
///
/// # Returns
/// Download URL with ?do=download parameter
///
/// # Errors
/// Returns error message as String if URL generation fails
///
/// # Requirements
/// - 7.1: Exposes get_download_url command
/// - 7.3: Returns error message as String on failure
#[tauri::command]
pub async fn get_download_url(
    state: State<'_, ScraperState>,
    video_slug: String,
    video_id: String,
) -> Result<String, String> {
    let scraper = state.scraper.lock().await;
    scraper
        .get_download_url(&video_slug, &video_id)
        .map_err(|e| e.to_string())
}
