# Any Downloader V 1.0.3

<img width="644" height="987" alt="image" src="https://github.com/user-attachments/assets/213f7e62-09b4-43a0-984d-8fc2615dbf87" />
A modern Rust GUI application for downloading files from various sources without using Windows Explorer dialogs.

![Rust](https://img.shields.io/badge/Rust-1.99.0-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey)

## Features


-  **Modern Dark Theme** - Clean, professional interface
-  **Custom App Branding** - Uses the project icon in the native window and a modern custom top bar
-  **Multi-Protocol Support**:
  - HTTP/HTTPS file downloads
  - Magnet links (via aria2c)
  - Torrent files (via aria2c)
  - Git repositories (via git clone)
  - YouTube videos (via yt-dlp)
-  **Real-time Progress** - Download speed and progress tracking
-  **Customizable** - Configure paths, window size, and theme colors
-  **Scrollable Settings** - Access the full configuration panel in a compact window
-  **Built-in Dependency Installer** - Install aria2c, yt-dlp, and git directly from the app
-  **Auto-Detection** - Automatically detects download type from URL
-  **Config Persistence** - Settings saved to JSON file
-  **Pin Mode** - Keep window always on top
-  **MIT Licensed** - License and copyright notice shown in the app

## Screenshots

![Download Tab](https://via.placeholder.com/600x400?text=Download+Tab)
![Config Tab](https://via.placeholder.com/600x400?text=Config+Tab)
![Dependencies Tab](https://via.placeholder.com/600x400?text=Dependencies+Tab)

## Installation

### Prerequisites

- Rust 1.99.0 or higher (install from [rustup.rs](https://rustup.rs/))
- Windows 10 or later

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/any_downloader.git
cd any_downloader

# Build release version
cargo build --release

# Run the application
cargo run --release
```

The executable will be located at `target\release\any_downloader.exe`.

## Usage

### Basic Download

1. Launch the application
2. Paste any URL in the input field
3. Click "Download"
4. The file will be saved to your Downloads folder (or configured directory)

### Supported URL Patterns

- **HTTP/HTTPS**: `https://example.com/file.zip`
- **Magnet**: `magnet:?xt=urn:btih:...`
- **Torrent**: `https://example.com/file.torrent`
- **Git**: `https://github.com/user/repo` or `https://gitlab.com/user/repo.git`
- **YouTube**: `https://youtube.com/watch?v=...` or `https://youtu.be/...`

### Configuration

Navigate to the **Settings** tab to customize:

- **Paths**: Download directory, aria2c, yt-dlp, and git executable paths
- **Window Size**: Set initial window dimensions
- **Theme Colors**: Customize primary and secondary colors using the color picker
- **Always on Top**: Toggle pin mode immediately
- Settings can be scrolled vertically; the default window size is 640 × 960 pixels

### Installing Dependencies

Navigate to the **Dependencies** tab to install required tools:

- Click "Install All" to install aria2c, yt-dlp, and git
- Or install individually using the respective buttons
- Installation uses winget (Windows Package Manager)

## Configuration File

The application creates a `config.json` file in the application directory:

```json
{
  "download_dir": "C:\\Users\\YourName\\Downloads",
  "aria2c_path": "aria2c",
  "yt_dlp_path": "yt-dlp",
  "git_path": "git",
  "primary_color": [0.27, 0.51, 0.71],
  "secondary_color": [0.39, 0.58, 0.93],
  "window_width": 640.0,
  "window_height": 960.0,
  "always_on_top": false
}
```

## Dependencies

### Runtime Dependencies (Optional)

- **aria2c** - For torrent and magnet link downloads
  - Install from: https://aria2.github.io/
  - Or use the in-app installer

- **yt-dlp** - For YouTube video downloads
  - Install from: https://github.com/yt-dlp/yt-dlp
  - Or use the in-app installer

- **git** - For repository cloning
  - Install from: https://git-scm.com/
  - Or use the in-app installer

### Build Dependencies

- eframe 0.27
- egui 0.27
- reqwest 0.12
- tokio 1
- directories 5
- serde 1
- futures-util 0.3

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## Troubleshooting

### "aria2c not found"
- Install aria2c using the Dependencies tab
- Or manually install from https://aria2.github.io/
- Or configure the path in Config tab

### "yt-dlp not found"
- Install yt-dlp using the Dependencies tab
- Or manually install from https://github.com/yt-dlp/yt-dlp
- Or configure the path in Config tab

### "git not found"
- Install git using the Dependencies tab
- Or manually install from https://git-scm.com/
- Or configure the path in Config tab

## Development

### Project Structure

```
any_downloader/
├── src/
│   └── main.rs          # Main application code
├── Cargo.toml           # Dependencies
├── config.json          # User configuration (generated)
├── README.md            # This file
└── LICENSE              # MIT License
```

### Architecture

- **DownloadType**: Enum for protocol detection
- **DownloadMessage**: Enum for thread communication
- **AppConfig**: Configuration struct with serde serialization
- **Modular download handlers**: Separate functions for each protocol
- **Tab-based UI**: Download, Config, and Dependencies tabs

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

Copyright © 2026 Yohann / Misaki-ux. This project is licensed under the MIT License; see [LICENSE](LICENSE) for the full terms.

## Acknowledgments

- [egui](https://github.com/emilk/egui) - Immediate mode GUI library
- [eframe](https://github.com/emilk/egui) - Framework for egui applications
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime

## Contact

For issues and questions, please open an issue on GitHub.

---

Made with ❤️ in Rust by [Misaki-ux](https://github.com/Misaki-ux)
