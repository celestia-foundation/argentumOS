//! Scan available wallpapers on argentumOS.

use crate::{Result, on_runtime};
use std::path::PathBuf;

const IMAGE_SUFFIXES: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];

pub fn wallpaper_roots() -> Vec<PathBuf> {
    let mut v = vec![PathBuf::from("/etc/backgrounds")];
    if let Ok(home) = std::env::var("HOME") {
        v.push(PathBuf::from(format!("{home}/Pictures")));
        v.push(PathBuf::from(format!("{home}/.local/share/backgrounds")));
    }
    v
}

pub async fn list_wallpapers() -> Result<Vec<PathBuf>> {
    on_runtime(async {
        let mut found = Vec::new();
        for root in wallpaper_roots() {
            let mut entries = match tokio::fs::read_dir(&root).await {
                Ok(e) => e,
                Err(_) => continue,
            };
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_ascii_lowercase());
                if let Some(ext) = ext {
                    if IMAGE_SUFFIXES.iter().any(|s| *s == ext) {
                        found.push(path);
                    }
                }
            }
        }
        Ok(found)
    })
    .await
}
