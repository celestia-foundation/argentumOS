//! Offline-friendly per-app metadata via `flatpak remote-info`. Used as a
//! fallback when the Flathub HTTP API is unreachable or the remote isn't
//! Flathub.

use crate::{Result, classify_flatpak_error, on_runtime};
use super::USER;

#[derive(Debug, Clone, Default)]
pub struct RemoteInfo {
    pub app_id: String,
    pub name: String,
    pub summary: String,
    pub version: String,
    pub license: String,
    pub download_size: String,
    pub installed_size: String,
    pub runtime: String,
    pub sdk: String,
}

pub async fn remote_info(remote: &str, app_id: &str) -> Result<RemoteInfo> {
    let remote = remote.to_string();
    let app_id = app_id.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "remote-info", &remote, &app_id])
            .output()
            .await?;
        if !out.status.success() {
            return Err(classify_flatpak_error(
                format!("flatpak remote-info {remote} {app_id}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(parse(&app_id, &String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

/// `flatpak remote-info` output is human-readable key/value pairs; the labels
/// are stable enough to parse with starts_with.
pub fn parse(app_id: &str, input: &str) -> RemoteInfo {
    let mut info = RemoteInfo { app_id: app_id.to_string(), ..Default::default() };
    for line in input.lines() {
        let t = line.trim_start();
        if let Some(v) = strip(t, "Name:") { info.name = v.into(); }
        else if let Some(v) = strip(t, "Summary:") { info.summary = v.into(); }
        else if let Some(v) = strip(t, "Version:") { info.version = v.into(); }
        else if let Some(v) = strip(t, "License:") { info.license = v.into(); }
        else if let Some(v) = strip(t, "Download:") { info.download_size = v.into(); }
        else if let Some(v) = strip(t, "Installed:") { info.installed_size = v.into(); }
        else if let Some(v) = strip(t, "Runtime:") { info.runtime = v.into(); }
        else if let Some(v) = strip(t, "Sdk:") { info.sdk = v.into(); }
    }
    info
}

fn strip<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.strip_prefix(prefix).map(|s| s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_labels() {
        let raw = "Name: KCalc\nSummary: A simple calculator\nVersion: 23.08\nLicense: GPL-2.0+\nDownload: 12.3 MB\nInstalled: 50.0 MB\nRuntime: org.kde.Platform/x86_64/6.6\n";
        let info = parse("org.kde.kcalc", raw);
        assert_eq!(info.name, "KCalc");
        assert_eq!(info.summary, "A simple calculator");
        assert_eq!(info.version, "23.08");
        assert_eq!(info.license, "GPL-2.0+");
        assert_eq!(info.download_size, "12.3 MB");
        assert_eq!(info.installed_size, "50.0 MB");
        assert!(info.runtime.starts_with("org.kde.Platform"));
    }
}
