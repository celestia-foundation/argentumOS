//! Scan installed GTK and icon themes on NixOS.

use crate::{Result, on_runtime};
use std::path::PathBuf;

pub fn gtk_theme_roots() -> Vec<PathBuf> {
    let mut v = vec![PathBuf::from("/run/current-system/sw/share/themes")];
    if let Ok(home) = std::env::var("HOME") {
        v.push(PathBuf::from(format!("{home}/.themes")));
        v.push(PathBuf::from(format!("{home}/.local/share/themes")));
    }
    v
}

pub fn icon_theme_roots() -> Vec<PathBuf> {
    let mut v = vec![PathBuf::from("/run/current-system/sw/share/icons")];
    if let Ok(home) = std::env::var("HOME") {
        v.push(PathBuf::from(format!("{home}/.icons")));
        v.push(PathBuf::from(format!("{home}/.local/share/icons")));
    }
    v
}

pub async fn list_gtk_themes() -> Result<Vec<String>> {
    on_runtime(async {
        let mut found: Vec<String> = Vec::new();
        for root in gtk_theme_roots() {
            let mut entries = match tokio::fs::read_dir(&root).await {
                Ok(e) => e,
                Err(_) => continue,
            };
            while let Some(entry) = entries.next_entry().await? {
                let name = match entry.file_name().into_string() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                if found.iter().any(|f| f == &name) {
                    continue;
                }
                let path = entry.path();
                let has_gtk3 = tokio::fs::metadata(path.join("gtk-3.0/gtk.css")).await.is_ok();
                let has_gtk4 = tokio::fs::metadata(path.join("gtk-4.0/gtk.css")).await.is_ok();
                if has_gtk3 || has_gtk4 {
                    found.push(name);
                }
            }
        }
        found.sort();
        Ok(found)
    })
    .await
}

pub async fn list_icon_themes() -> Result<Vec<String>> {
    on_runtime(async {
        let mut found: Vec<String> = Vec::new();
        for root in icon_theme_roots() {
            let mut entries = match tokio::fs::read_dir(&root).await {
                Ok(e) => e,
                Err(_) => continue,
            };
            while let Some(entry) = entries.next_entry().await? {
                let name = match entry.file_name().into_string() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                if name == "default" || name == "hicolor" {
                    continue;
                }
                if found.iter().any(|f| f == &name) {
                    continue;
                }
                if tokio::fs::metadata(entry.path().join("index.theme")).await.is_ok() {
                    found.push(name);
                }
            }
        }
        found.sort();
        Ok(found)
    })
    .await
}

pub async fn gsettings_get(schema: &str, key: &str) -> Result<String> {
    let schema = schema.to_string();
    let key = key.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("gsettings")
            .args(["get", &schema, &key])
            .output()
            .await?;
        let s = String::from_utf8_lossy(&out.stdout)
            .trim()
            .trim_matches('\'')
            .trim_matches('"')
            .to_string();
        Ok(s)
    })
    .await
}

pub async fn gsettings_set(schema: &str, key: &str, value: &str) -> Result<()> {
    let schema = schema.to_string();
    let key = key.to_string();
    let value = value.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("gsettings")
            .args(["set", &schema, &key, &value])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(crate::Error::Subprocess {
                cmd: format!("gsettings set {schema} {key} {value}"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}
