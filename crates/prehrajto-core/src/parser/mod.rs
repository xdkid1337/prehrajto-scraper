//! HTML parsers for prehraj.to
//!
//! Contains modules for parsing different page types.

pub mod direct_url;
pub mod search;

pub use direct_url::parse_direct_url;
pub use search::parse_search_results;
