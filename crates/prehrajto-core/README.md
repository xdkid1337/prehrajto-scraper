# prehrajto-core

Async Rust library for searching videos and getting download links from [prehraj.to](https://prehraj.to).

## Features

- 🔍 Search videos by keywords
- 📥 Generate download URLs
- 🎯 Extract direct CDN URLs (premiumcdn.net) for streaming/downloading
- ⏱️ Built-in rate limiting to respect server limits
- 🔄 Automatic retry with exponential backoff
- 📦 Serde serialization support

## Installation

```toml
[dependencies]
prehrajto-core = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Usage

```rust
use prehrajto_core::{PrehrajtoScraper, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let scraper = PrehrajtoScraper::new()?;
    
    // Search for videos
    let results = scraper.search("doctor who").await?;
    
    for video in results {
        println!("{}", video.name);
        println!("  Duration: {:?}", video.duration);
        println!("  Size: {:?}", video.file_size);
        println!("  Download: {}", video.download_url);
    }
    
    // Get direct CDN URL for streaming/downloading
    if let Some(video) = results.first() {
        let cdn_url = scraper.get_direct_url(&video.video_slug, &video.video_id).await?;
        println!("CDN URL: {}", cdn_url);
        // Returns: https://pf-storage4.premiumcdn.net/...?token=...&expires=...
    }
    
    Ok(())
}
```

## Direct CDN URLs

The `get_direct_url` method extracts the actual CDN URL from the download page:

```rust
let cdn_url = scraper.get_direct_url(&video.video_slug, &video.video_id).await?;
```

**Important notes:**
- The URL contains `token` and `expires` parameters
- URLs expire after a limited time (typically hours) - don't cache long-term
- Use this URL for actual file download or video streaming

## Configuration

Customize the HTTP client behavior:

```rust
use prehrajto_core::{PrehrajtoScraper, ClientConfig};

let config = ClientConfig {
    requests_per_second: 1.0,  // Max requests per second
    timeout_secs: 60,          // Request timeout
    max_retries: 5,            // Retry attempts on failure
};

let scraper = PrehrajtoScraper::with_config(config)?;
```

## VideoResult

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Video title |
| `url` | `String` | Video page URL |
| `video_id` | `String` | Unique video ID |
| `video_slug` | `String` | URL-friendly slug |
| `download_url` | `String` | Download page URL (redirects) |
| `duration` | `Option<String>` | Duration (HH:MM:SS) |
| `quality` | `Option<String>` | Quality (e.g., "HD") |
| `file_size` | `Option<String>` | File size |

## API Methods

| Method | Description |
|--------|-------------|
| `search(query)` | Search videos by keywords |
| `get_download_url(slug, id)` | Get download page URL (sync) |
| `get_direct_url(slug, id)` | Get direct CDN URL (async) |

## License

MIT

## Legal Disclaimer

This project is provided **for educational and research purposes only**.

According to prehraj.to Terms of Service (Articles 7.5 and 7.6), automated requests to their servers are prohibited. By using this library, you acknowledge that:

- You are solely responsible for how you use this software
- The authors are not liable for any misuse or violations of third-party terms of service
- You should obtain proper authorization before scraping any website

**Use at your own risk.**
