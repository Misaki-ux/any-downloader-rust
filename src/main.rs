use eframe::egui;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Instant;

const DEFAULT_WINDOW_WIDTH: f32 = 640.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 960.0;
const MIN_WINDOW_WIDTH: f32 = 640.0;
const MIN_WINDOW_HEIGHT: f32 = 480.0;

#[derive(Clone, Debug)]
enum DownloadMessage {
    Started(String),
    Progress(f32, String, String), // percentage, speed, downloaded/total
    Finished(String),
    Error(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DownloadType {
    Http,
    Magnet,
    Torrent,
    Git,
    Youtube,
}

impl DownloadType {
    fn from_url(url: &str) -> Self {
        if url.starts_with("magnet:") {
            DownloadType::Magnet
        } else if url.ends_with(".torrent") {
            DownloadType::Torrent
        } else if url.contains("github.com") || url.contains("gitlab.com") || url.ends_with(".git") {
            DownloadType::Git
        } else if url.contains("youtube.com") || url.contains("youtu.be") {
            DownloadType::Youtube
        } else {
            DownloadType::Http
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct AppConfig {
    download_dir: String,
    aria2c_path: String,
    yt_dlp_path: String,
    git_path: String,
    #[serde(default)]
    primary_color: [f32; 3],
    #[serde(default)]
    secondary_color: [f32; 3],
    #[serde(default)]
    window_width: f32,
    #[serde(default)]
    window_height: f32,
    #[serde(default)]
    always_on_top: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Download,
    Config,
    Dependencies,
}

struct App {
    url: String,
    status: String,
    progress: f32,
    speed: String,
    downloaded: String,
    rx: Option<Receiver<DownloadMessage>>,
    config: AppConfig,
    current_tab: Tab,
    install_status: String,
    logo: Option<egui::TextureHandle>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            url: String::new(),
            status: "Idle".to_string(),
            progress: 0.0,
            speed: "0 MB/s".to_string(),
            downloaded: "0 B / 0 B".to_string(),
            rx: None,
            config: AppConfig {
                download_dir: default_download_dir().to_string_lossy().to_string(),
                aria2c_path: "aria2c".to_string(),
                yt_dlp_path: "yt-dlp".to_string(),
                git_path: "git".to_string(),
                primary_color: [0.27, 0.51, 0.71], // Steel blue (normalized RGB)
                secondary_color: [0.39, 0.58, 0.93], // Cornflower blue (normalized RGB)
                window_width: DEFAULT_WINDOW_WIDTH,
                window_height: DEFAULT_WINDOW_HEIGHT,
                always_on_top: false,
            },
            current_tab: Tab::Download,
            install_status: "Ready to install dependencies".to_string(),
            logo: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint for smooth progress bar
        ctx.request_repaint();

        if self.logo.is_none() {
            if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("assets/rust-downloader-icon.png")) {
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width as usize, icon.height as usize],
                    &icon.rgba,
                );
                self.logo = Some(ctx.load_texture("app-logo", image, egui::TextureOptions::LINEAR));
            }
        }

        // Poll for messages from download thread
        let mut should_clear_rx = false;
        if let Some(ref rx) = self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    DownloadMessage::Started(s) => {
                        self.status = s;
                        self.progress = 0.0;
                    }
                    DownloadMessage::Progress(p, s, d) => {
                        self.progress = p;
                        self.speed = s;
                        self.downloaded = d;
                    }
                    DownloadMessage::Finished(s) => {
                        self.status = s;
                        self.progress = 100.0;
                        should_clear_rx = true;
                    }
                    DownloadMessage::Error(s) => {
                        self.status = s;
                        self.progress = 0.0;
                        should_clear_rx = true;
                    }
                }
            }
        }
        if should_clear_rx {
            self.rx = None;
        }

        egui::TopBottomPanel::top("custom_top_bar")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(22, 25, 32)).inner_margin(egui::Margin::symmetric(18.0, 12.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(logo) = &self.logo {
                        ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(38.0, 38.0)));
                    }
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Any Downloader").size(18.0).strong());
                        ui.label(egui::RichText::new("Downloads, made simple").size(11.0).color(egui::Color32::from_gray(160)));
                    });
                    ui.add_space(18.0);

                    for (tab, label) in [
                        (Tab::Download, "Download"),
                        (Tab::Config, "Settings"),
                        (Tab::Dependencies, "Dependencies"),
                    ] {
                        let selected = self.current_tab == tab;
                        let button = egui::Button::new(egui::RichText::new(label).size(13.0).color(
                            if selected { egui::Color32::WHITE } else { egui::Color32::from_gray(175) },
                        )).fill(if selected { egui::Color32::from_rgb(48, 82, 125) } else { egui::Color32::TRANSPARENT });
                        if ui.add(button).clicked() {
                            self.current_tab = tab;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let pin_label = if self.config.always_on_top { "Pinned" } else { "Pin" };
                        if ui.button(format!("📌 {pin_label}")).clicked() {
                            self.config.always_on_top = !self.config.always_on_top;
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                if self.config.always_on_top { egui::viewport::WindowLevel::AlwaysOnTop } else { egui::viewport::WindowLevel::Normal },
                            ));
                            self.status = format!("Pin {}", if self.config.always_on_top { "enabled" } else { "disabled" });
                        }
                    });
                });
            });

        if matches!(self.current_tab, Tab::Download | Tab::Config) {
            egui::TopBottomPanel::bottom("license_footer")
                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(22, 25, 32)).inner_margin(egui::Margin::symmetric(14.0, 5.0)))
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("MIT License · © 2026 Yohann / Misaki-ux").size(10.0).color(egui::Color32::from_gray(145)));
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Download => {
                    ui.add_space(15.0);
                    ui.heading("Download");
                    ui.separator();
                    ui.add_space(15.0);
                    
                    ui.label("Paste URL / magnet / torrent / git repo / youtube / etc.");

                    ui.add_space(10.0);
                    ui.add_sized([ui.available_width().min(500.0), 0.0], egui::TextEdit::singleline(&mut self.url).hint_text("https://example.com/file.zip"));

                    ui.add_space(15.0);

                    if ui.add_sized([150.0, 0.0], egui::Button::new("Download")).clicked() && !self.url.is_empty() {
                        let url = self.url.clone();
                        let download_type = DownloadType::from_url(&url);
                        self.status = format!("Starting {} download...", format!("{:?}", download_type).to_lowercase());
                        self.progress = 0.0;
                        self.rx = Some(route_download(url, download_type, self.config.clone()));
                    }

                    ui.separator();
                    ui.add_space(15.0);

                    ui.label(format!("Status: {}", self.status));

                    if self.progress > 0.0 && self.progress < 100.0 {
                        ui.add_space(20.0);
                        
                        // Progress bar with configurable secondary color
                        let fill_color = egui::Color32::from_rgb(
                            (self.config.secondary_color[0] * 255.0) as u8,
                            (self.config.secondary_color[1] * 255.0) as u8,
                            (self.config.secondary_color[2] * 255.0) as u8,
                        );
                        ui.add(
                            egui::ProgressBar::new(self.progress / 100.0)
                                .show_percentage()
                                .fill(fill_color)
                        );
                        
                        ui.add_space(10.0);
                        ui.label(format!("Speed: {}", self.speed));
                        ui.label(format!("Downloaded: {}", self.downloaded));
                    }
                }
                Tab::Config => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(15.0);
                    ui.heading("Configuration");
                    ui.separator();
                    ui.add_space(15.0);

                    ui.heading("Paths");
                    ui.add_space(10.0);

                    ui.label("Download Directory:");
                    ui.add_sized([ui.available_width().min(500.0), 0.0], egui::TextEdit::singleline(&mut self.config.download_dir));
                    ui.add_space(10.0);

                    ui.label("aria2c Path (for torrents):");
                    ui.add_sized([ui.available_width().min(500.0), 0.0], egui::TextEdit::singleline(&mut self.config.aria2c_path));
                    ui.add_space(10.0);

                    ui.label("yt-dlp Path (for YouTube):");
                    ui.add_sized([ui.available_width().min(500.0), 0.0], egui::TextEdit::singleline(&mut self.config.yt_dlp_path));
                    ui.add_space(10.0);

                    ui.label("git Path (for repos):");
                    ui.add_sized([ui.available_width().min(500.0), 0.0], egui::TextEdit::singleline(&mut self.config.git_path));
                    ui.add_space(20.0);

                    ui.separator();
                    ui.add_space(15.0);

                    ui.heading("Window Settings");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(egui::DragValue::new(&mut self.config.window_width)
                            .speed(10.0)
                            .clamp_range(MIN_WINDOW_WIDTH..=3840.0));
                        ui.label("px");
                        ui.add_space(20.0);
                        ui.label("Height:");
                        ui.add(egui::DragValue::new(&mut self.config.window_height)
                            .speed(10.0)
                            .clamp_range(MIN_WINDOW_HEIGHT..=2160.0));
                        ui.label("px");
                    });
                    ui.add_space(10.0);
                    
                    if ui.button("Apply Window Size").clicked() {
                        self.config.window_width = self.config.window_width.max(MIN_WINDOW_WIDTH);
                        self.config.window_height = self.config.window_height.max(MIN_WINDOW_HEIGHT);
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            self.config.window_width,
                            self.config.window_height,
                        )));
                    }
                    ui.add_space(20.0);

                    ui.separator();
                    ui.add_space(15.0);

                    ui.heading("Theme Colors");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Primary Color:");
                        let mut primary_color = egui::Color32::from_rgb(
                            (self.config.primary_color[0] * 255.0) as u8,
                            (self.config.primary_color[1] * 255.0) as u8,
                            (self.config.primary_color[2] * 255.0) as u8,
                        );
                        if egui::color_picker::color_picker_color32(ui, &mut primary_color, egui::color_picker::Alpha::Opaque) {
                            self.config.primary_color = [
                                primary_color.r() as f32 / 255.0,
                                primary_color.g() as f32 / 255.0,
                                primary_color.b() as f32 / 255.0,
                            ];
                        }
                    });
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Secondary Color (Progress Bar):");
                        let mut secondary_color = egui::Color32::from_rgb(
                            (self.config.secondary_color[0] * 255.0) as u8,
                            (self.config.secondary_color[1] * 255.0) as u8,
                            (self.config.secondary_color[2] * 255.0) as u8,
                        );
                        if egui::color_picker::color_picker_color32(ui, &mut secondary_color, egui::color_picker::Alpha::Opaque) {
                            self.config.secondary_color = [
                                secondary_color.r() as f32 / 255.0,
                                secondary_color.g() as f32 / 255.0,
                                secondary_color.b() as f32 / 255.0,
                            ];
                        }
                    });
                    ui.add_space(20.0);

                    ui.separator();
                    ui.add_space(15.0);

                    if ui.add_sized([120.0, 0.0], egui::Button::new("💾 Save Config")).clicked() {
                        if let Ok(config_str) = serde_json::to_string_pretty(&self.config) {
                            let _ = std::fs::write("config.json", config_str);
                            self.status = "Configuration saved".to_string();
                        }
                    }
                    });
                }
                Tab::Dependencies => {
                    ui.add_space(15.0);
                    ui.heading("Install Dependencies");
                    ui.separator();
                    ui.add_space(15.0);

                    ui.label("Install external tools required for advanced features:");
                    ui.add_space(10.0);

                    ui.label("• aria2c - For torrent and magnet link downloads");
                    ui.label("• yt-dlp - For YouTube video downloads");
                    ui.label("• git - For repository cloning");
                    ui.add_space(20.0);

                    ui.separator();
                    ui.add_space(15.0);

                    ui.label(format!("Status: {}", self.install_status));
                    ui.add_space(15.0);

                    if ui.button("Install aria2c").clicked() {
                        self.install_status = "Installing aria2c...".to_string();
                        self.install_aria2c();
                    }
                    ui.add_space(10.0);

                    if ui.button("Install yt-dlp").clicked() {
                        self.install_status = "Installing yt-dlp...".to_string();
                        self.install_yt_dlp();
                    }
                    ui.add_space(10.0);

                    if ui.button("Install git").clicked() {
                        self.install_status = "Installing git...".to_string();
                        self.install_git();
                    }
                    ui.add_space(20.0);

                    ui.separator();
                    ui.add_space(15.0);

                    if ui.button("Install All").clicked() {
                        self.install_status = "Installing all dependencies...".to_string();
                        self.install_aria2c();
                        self.install_yt_dlp();
                        self.install_git();
                        self.install_status = "Installation complete!".to_string();
                    }
                    ui.add_space(15.0);

                    ui.label("Note: This will attempt to install using winget (Windows Package Manager).");
                    ui.label("If winget is not available, please install manually from the official websites.");
                }
            }
        });
    }
}

impl App {
    fn install_aria2c(&mut self) {
        let _ = std::process::Command::new("winget")
            .args(&["install", "--id", "aria2.aria2", "-e", "--accept-source-agreements", "--accept-package-agreements"])
            .spawn();
        self.install_status = "aria2c installation started".to_string();
    }

    fn install_yt_dlp(&mut self) {
        let _ = std::process::Command::new("winget")
            .args(&["install", "--id", "yt-dlp.yt-dlp", "-e", "--accept-source-agreements", "--accept-package-agreements"])
            .spawn();
        self.install_status = "yt-dlp installation started".to_string();
    }

    fn install_git(&mut self) {
        let _ = std::process::Command::new("winget")
            .args(&["install", "--id", "Git.Git", "-e", "--accept-source-agreements", "--accept-package-agreements"])
            .spawn();
        self.install_status = "git installation started".to_string();
    }
}

fn main() -> eframe::Result<()> {
    // Try to load config from file
    let mut config = if let Ok(config_str) = std::fs::read_to_string("config.json") {
        serde_json::from_str(&config_str).unwrap_or_default()
    } else {
        AppConfig::default()
    };
    normalize_window_size(&mut config);

    let config_clone = config.clone();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([config.window_width, config.window_height])
        .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT])
        .with_resizable(true);
    if config.always_on_top {
        viewport = viewport.with_always_on_top();
    }
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("assets/rust-downloader-icon.png")) {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        default_theme: eframe::Theme::Dark,
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Any Downloader",
        options,
        Box::new(move |_cc| Box::new(App {
            config: config_clone,
            ..Default::default()
        })),
    )
}

fn normalize_window_size(config: &mut AppConfig) {
    if !config.window_width.is_finite() || config.window_width < MIN_WINDOW_WIDTH {
        config.window_width = DEFAULT_WINDOW_WIDTH;
    }
    if !config.window_height.is_finite() || config.window_height < MIN_WINDOW_HEIGHT {
        config.window_height = DEFAULT_WINDOW_HEIGHT;
    }
}

fn get_download_dir(config: &AppConfig) -> PathBuf {
    if !config.download_dir.is_empty() {
        PathBuf::from(&config.download_dir)
    } else {
        default_download_dir()
    }
}

fn default_download_dir() -> PathBuf {
    let user_dirs = directories::UserDirs::new().expect("cannot get user dirs");
    user_dirs.download_dir()
        .unwrap_or(user_dirs.home_dir())
        .to_path_buf()
}

fn route_download(url: String, download_type: DownloadType, config: AppConfig) -> Receiver<DownloadMessage> {
    match download_type {
        DownloadType::Magnet | DownloadType::Torrent => spawn_torrent_download(url, config),
        DownloadType::Git => spawn_git_download(url, config),
        DownloadType::Youtube => spawn_youtube_download(url, config),
        DownloadType::Http => spawn_http_download(url, config),
    }
}

fn spawn_http_download(url: String, config: AppConfig) -> Receiver<DownloadMessage> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let download_dir = get_download_dir(&config);
            
            // Extract filename from URL, handle query parameters
            let filename = url
                .split('/')
                .last()
                .unwrap_or("download.bin")
                .split('?')
                .next()
                .unwrap_or("download.bin")
                .to_string();

            let mut path = PathBuf::from(&download_dir);
            path.push(&filename);

            let _ = tx.send(DownloadMessage::Started(format!("Downloading {}", filename)));

            let client = reqwest::Client::new();
            let res = match client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(DownloadMessage::Error(format!("HTTP Error: {}", e)));
                    return;
                }
            };

            let total_size = res.content_length().unwrap_or(0);
            let mut file = match File::create(&path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(DownloadMessage::Error(format!("File error: {}", e)));
                    return;
                }
            };

            let mut downloaded: u64 = 0;
            let start_time = Instant::now();
            let mut stream = res.bytes_stream();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let chunk_size = bytes.len() as u64;
                        if let Err(e) = file.write_all(&bytes) {
                            let _ = tx.send(DownloadMessage::Error(format!("Write error: {}", e)));
                            return;
                        }
                        downloaded += chunk_size;

                        let progress = if total_size > 0 {
                            (downloaded as f32 / total_size as f32) * 100.0
                        } else {
                            0.0
                        };

                        let elapsed = start_time.elapsed().as_secs_f32();
                        let speed = if elapsed > 0.0 {
                            (downloaded as f32 / elapsed) / (1024.0 * 1024.0)
                        } else {
                            0.0
                        };

                        let _ = tx.send(DownloadMessage::Progress(
                            progress,
                            format!("{:.2} MB/s", speed),
                            format_bytes(downloaded, total_size),
                        ));
                    }
                    Err(e) => {
                        let _ = tx.send(DownloadMessage::Error(format!("Stream error: {}", e)));
                        return;
                    }
                }
            }

            let _ = tx.send(DownloadMessage::Finished(format!("Done: {}", path.display())));
        });
    });

    rx
}

fn spawn_torrent_download(url: String, config: AppConfig) -> Receiver<DownloadMessage> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let download_dir = get_download_dir(&config);
        let _ = tx.send(DownloadMessage::Started("Starting torrent download...".to_string()));

        let result = std::process::Command::new(&config.aria2c_path)
            .arg("--dir")
            .arg(download_dir.to_string_lossy().to_string())
            .arg(&url)
            .spawn();

        match result {
            Ok(mut child) => {
                let _ = tx.send(DownloadMessage::Progress(
                    50.0,
                    "N/A (external)".to_string(),
                    "External download".to_string(),
                ));

                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            let _ = tx.send(DownloadMessage::Finished(
                                "Torrent download completed".to_string(),
                            ));
                        } else {
                            let _ = tx.send(DownloadMessage::Error(
                                "Torrent download failed".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(DownloadMessage::Error(format!("aria2c error: {}", e)));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(DownloadMessage::Error(format!(
                    "aria2c not found. Please install aria2c or configure path in settings: {}",
                    e
                )));
            }
        }
    });

    rx
}

fn spawn_youtube_download(url: String, config: AppConfig) -> Receiver<DownloadMessage> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let download_dir = get_download_dir(&config);
        let _ = tx.send(DownloadMessage::Started("Starting YouTube download...".to_string()));

        let result = std::process::Command::new(&config.yt_dlp_path)
            .arg("-o")
            .arg(format!("{}/%(title)s.%(ext)s", download_dir.to_string_lossy()))
            .arg(&url)
            .spawn();

        match result {
            Ok(mut child) => {
                let _ = tx.send(DownloadMessage::Progress(
                    50.0,
                    "N/A (external)".to_string(),
                    "Downloading via yt-dlp".to_string(),
                ));

                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            let _ = tx.send(DownloadMessage::Finished(
                                "YouTube download completed".to_string(),
                            ));
                        } else {
                            let _ = tx.send(DownloadMessage::Error(
                                "YouTube download failed".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(DownloadMessage::Error(format!("yt-dlp error: {}", e)));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(DownloadMessage::Error(format!(
                    "yt-dlp not found. Please install yt-dlp or configure path in settings: {}",
                    e
                )));
            }
        }
    });

    rx
}

fn spawn_git_download(url: String, config: AppConfig) -> Receiver<DownloadMessage> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let download_dir = get_download_dir(&config);
        let repo_name = url
            .split('/')
            .last()
            .unwrap_or("repo")
            .trim_end_matches(".git")
            .to_string();

        let mut path = PathBuf::from(&download_dir);
        path.push(&repo_name);

        let _ = tx.send(DownloadMessage::Started(format!("Cloning {}...", repo_name)));

        let result = std::process::Command::new(&config.git_path)
            .arg("clone")
            .arg(&url)
            .arg(&path)
            .spawn();

        match result {
            Ok(mut child) => {
                let _ = tx.send(DownloadMessage::Progress(
                    50.0,
                    "N/A (external)".to_string(),
                    "Cloning via git".to_string(),
                ));

                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            let _ = tx.send(DownloadMessage::Finished(format!(
                                "Repo cloned to: {}",
                                path.display()
                            )));
                        } else {
                            let _ = tx.send(DownloadMessage::Error("Git clone failed".to_string()));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(DownloadMessage::Error(format!("git error: {}", e)));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(DownloadMessage::Error(format!(
                    "git not found. Please install git or configure path in settings: {}",
                    e
                )));
            }
        }
    });

    rx
}

fn format_bytes(downloaded: u64, total: u64) -> String {
    fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.2} KB", size as f32 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.2} MB", size as f32 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size as f32 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    if total > 0 {
        format!("{} / {}", format_size(downloaded), format_size(total))
    } else {
        format_size(downloaded)
    }
}
