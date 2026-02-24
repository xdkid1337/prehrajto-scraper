//! HTML parsers for prehraj.to
//!
//! Contains modules for parsing different page types.

pub mod direct_url;
pub mod search;

pub use direct_url::{
    parse_direct_url, parse_original_download_url, parse_subtitle_tracks, parse_video_sources,
};
pub use search::parse_search_results;
