# prehrajto-tauri

Tauri plugin for [prehrajto-core](https://crates.io/crates/prehrajto-core) - search videos and get download links from [prehraj.to](https://prehraj.to).

## Features

- 🖥️ Ready-to-use Tauri plugin
- 🔒 Thread-safe state management
- 📡 Async command handlers
- ⏱️ Built-in rate limiting

## Installation

```toml
[dependencies]
prehrajto-tauri = "0.1"
tauri = { version = "2", features = [] }
```

## Setup

Register the plugin in your Tauri app:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(prehrajto_tauri::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Frontend Usage

```javascript
import { invoke } from '@tauri-apps/api/core';

// Search for videos
const results = await invoke('plugin:prehrajto|search_videos', { 
    query: 'doctor who' 
});

// Get download URL
const url = await invoke('plugin:prehrajto|get_download_url', {
    videoSlug: 'doctor-who-s07e05',
    videoId: '63aba7f51f6cf'
});
```

## Commands

### `search_videos`

Search for videos by query string.

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `string` | Search query |

Returns: `VideoResult[]`

### `get_download_url`

Generate download URL for a video.

| Parameter | Type | Description |
|-----------|------|-------------|
| `videoSlug` | `string` | URL-friendly video slug |
| `videoId` | `string` | Unique video ID |

Returns: `string`

## VideoResult

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | Video title |
| `url` | `string` | Video page URL |
| `video_id` | `string` | Unique video ID |
| `video_slug` | `string` | URL-friendly slug |
| `download_url` | `string` | Direct download link |
| `duration` | `string \| null` | Duration (HH:MM:SS) |
| `quality` | `string \| null` | Quality (e.g., "HD") |
| `file_size` | `string \| null` | File size |

## License

MIT

## Legal Disclaimer

This project is provided **for educational and research purposes only**.

According to prehraj.to Terms of Service (Articles 7.5 and 7.6), automated requests to their servers are prohibited. By using this library, you acknowledge that:

- You are solely responsible for how you use this software
- The authors are not liable for any misuse or violations of third-party terms of service
- You should obtain proper authorization before scraping any website

**Use at your own risk.**
