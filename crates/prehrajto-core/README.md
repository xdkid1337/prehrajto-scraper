# prehrajto-core

Async Rust library for searching videos and getting download links from [prehraj.to](https://prehraj.to).

## Features

- 🔍 Search videos by keywords
- 📥 Generate direct download URLs
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
    
    Ok(())
}
```

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
| `download_url` | `String` | Direct download link |
| `duration` | `Option<String>` | Duration (HH:MM:SS) |
| `quality` | `Option<String>` | Quality (e.g., "HD") |
| `file_size` | `Option<String>` | File size |

## License

MIT

## Legal Disclaimer

This project is provided **for educational and research purposes only**.

According to prehraj.to Terms of Service (Articles 7.5 and 7.6), automated requests to their servers are prohibited. By using this library, you acknowledge that:

- You are solely responsible for how you use this software
- The authors are not liable for any misuse or violations of third-party terms of service
- You should obtain proper authorization before scraping any website

**Use at your own risk.**
