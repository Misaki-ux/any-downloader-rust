# Any Downloader - Rust Project

## Project Overview
A modern Rust GUI application for downloading files from various sources (HTTP/HTTPS, torrents, magnet links, git repos, YouTube) without using Windows Explorer dialogs.

## Tech Stack
- **GUI**: eframe + egui (pure Rust, modern, cross-platform)
- **HTTP/HTTPS**: reqwest (async, streaming)
- **File paths**: directories crate (gets Windows Downloads folder automatically)
- **Async runtime**: tokio
- **Config**: serde + serde_json

## Prerequisites
Install Rust via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or on Windows: https://rustup.rs/

**Optional for advanced features:**
- aria2c for torrent/magnet support: https://aria2.github.io/
- yt-dlp for YouTube support: https://github.com/yt-dlp/yt-dlp
- git for repo cloning: https://git-scm.com/

**Note:** You can install these dependencies directly from the app's Dependencies tab using winget.

## Build Commands
```bash
# Build the project
cargo build

# Run the application
cargo run

# Build release version
cargo build --release
```

## Features
- **Dark theme** by default
- **Custom top bar** with title, tabs, and pin button
- **Configuration page** to customize paths and settings
- **Resizable window** with configurable resolution
- **Pin button** (saved in config, requires restart to take effect)
- **Custom theme colors** (primary and secondary color)
- **Dependency installer** - Install aria2c, yt-dlp, and git directly from the app
- **Automatic download** to configurable folder
- **No Explorer "Save As" dialog**
- **Progress bar** with real-time download speed (customizable color)
- **Multi-protocol support**:
  - HTTP/HTTPS file downloads (mp4, img, mkv, iso, txt, etc.)
  - Magnet links (via aria2c)
  - Torrent files (via aria2c)
  - Git repositories (via git clone)
  - YouTube videos (via yt-dlp)
- **Automatic protocol detection** based on URL pattern
- **Config persistence** via JSON file

## Supported URL Patterns
- HTTP/HTTPS: Any standard URL (e.g., `https://example.com/file.zip`)
- Magnet: URLs starting with `magnet:` (e.g., `magnet:?xt=urn:btih:...`)
- Torrent: URLs ending with `.torrent`
- Git: GitHub/GitLab URLs or URLs ending with `.git`
- YouTube: URLs containing `youtube.com` or `youtu.be`

## Configuration
The app creates a `config.json` file in the application directory with the following settings:
- `download_dir`: Directory where files are saved
- `aria2c_path`: Path to aria2c executable (for torrents)
- `yt_dlp_path`: Path to yt-dlp executable (for YouTube)
- `git_path`: Path to git executable (for repos)
- `primary_color`: RGB values for primary theme color
- `secondary_color`: RGB values for secondary theme color (used for progress bar)
- `window_width`: Initial window width
- `window_height`: Initial window height
- `always_on_top`: Whether window should be always on top (requires restart)

### Tabs
- **Download**: Main download interface with URL input and progress display
- **Config**: Configure paths, window size, and theme colors
- **Dependencies**: Install aria2c, yt-dlp, and git using winget

## Architecture
The application uses a modular design with:
- `DownloadType` enum for protocol detection
- `DownloadMessage` enum for thread communication
- `AppConfig` struct for configuration persistence
- Separate spawn functions for each download type:
  - `spawn_http_download()` - Handles HTTP/HTTPS with progress tracking
  - `spawn_torrent_download()` - Spawns aria2c for torrents/magnets
  - `spawn_git_download()` - Spawns git clone for repositories
  - `spawn_youtube_download()` - Spawns yt-dlp for YouTube videos
- `route_download()` - Routes URLs to appropriate handler based on type
- Tab-based UI with Download, Config, and Dependencies tabs
- Installation methods for external dependencies using winget
