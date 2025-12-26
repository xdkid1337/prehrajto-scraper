# 🎬 Prehraj.to Scraper

Async Rust library for searching videos and getting download links from [prehraj.to](https://prehraj.to).

## ✨ Features

- 🔍 Search videos by keywords
- 📥 Generate direct download URLs
- ⏱️ Built-in rate limiting to respect server limits
- 🔄 Automatic retry with exponential backoff
- 🖥️ Tauri plugin for desktop apps

## 📦 Project Structure

```
├── crates/
│   ├── prehrajto-core/     # Core scraping library
│   └── prehrajto-tauri/    # Tauri plugin for frontend integration
```

## 🚀 Quick Start

### As a Rust Library

```rust
use prehrajto_core::{PrehrajtoScraper, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let scraper = PrehrajtoScraper::new()?;
    
    // Search for videos
    let results = scraper.search("doctor who").await?;
    
    for video in results {
        println!("📺 {}", video.name);
        println!("   Duration: {:?}", video.duration);
        println!("   Size: {:?}", video.file_size);
        println!("   Download: {}", video.download_url);
        println!();
    }
    
    Ok(())
}
```

### With Tauri

```rust
// main.rs
fn main() {
    tauri::Builder::default()
        .plugin(prehrajto_tauri::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```javascript
// Frontend (JavaScript/TypeScript)
import { invoke } from '@tauri-apps/api/core';

// Search
const results = await invoke('plugin:prehrajto|search_videos', { 
    query: 'doctor who' 
});

// Get download URL
const url = await invoke('plugin:prehrajto|get_download_url', {
    videoSlug: 'doctor-who-s07e05',
    videoId: '63aba7f51f6cf'
});
```

## ⚙️ Configuration

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

## 📋 VideoResult

Structure containing video information:

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

## 🛠️ Development

```bash
# Build
cargo build

# Tests
cargo test

# Run example
cargo run --example debug_html
```

## 📄 License

MIT

## ⚠️ Legal Disclaimer

This project is provided **for educational and research purposes only**.

According to prehraj.to Terms of Service (Articles 7.5 and 7.6), automated requests to their servers are prohibited. By using this library, you acknowledge that:

- You are solely responsible for how you use this software
- The authors are not liable for any misuse or violations of third-party terms of service
- You should obtain proper authorization before scraping any website

**Use at your own risk.**
