fn main() {
    println!("cargo:rerun-if-changed=src/assets/rust-downloader-icon.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resources = winres::WindowsResource::new();
        resources.set_icon("src/assets/rust-downloader-icon.ico");
        resources.set("ProductName", "Any Downloader");
        resources.set("FileDescription", "Any Downloader");
        resources.set("LegalCopyright", "Copyright © 2026 Yohann / Misaki-ux");
        resources.compile().expect("failed to embed Windows app icon and metadata");
    }
}
