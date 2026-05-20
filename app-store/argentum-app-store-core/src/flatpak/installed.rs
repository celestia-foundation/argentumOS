//! Installed apps — list, uninstall, per-app update.

use crate::{Result, classify_flatpak_error, on_runtime};
use super::USER;

#[derive(Debug, Clone)]
pub struct InstalledApp {
    pub app_id: String,
    pub name: String,
    pub version: String,
    pub branch: String,
    pub origin: String,
    pub size: String,
    pub installation: String,
}

pub async fn list_installed() -> Result<Vec<InstalledApp>> {
    on_runtime(async {
        let out = tokio::process::Command::new("flatpak")
            .args([
                USER,
                "list",
                "--app",
                "--columns=application,name,version,branch,origin,size,installation",
            ])
            .output()
            .await?;
        if !out.status.success() {
            return Err(classify_flatpak_error(
                "flatpak list",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(parse(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

pub async fn uninstall(app_id: &str) -> Result<()> {
    let app_id = app_id.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "uninstall", "--assumeyes", &app_id])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(classify_flatpak_error(
                format!("flatpak uninstall {app_id}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ))
        }
    })
    .await
}

/// List of apps with pending updates. `flatpak remote-ls --updates` is the
/// blessed way; output uses the same columns as `remote-ls`.
pub async fn list_pending_updates() -> Result<Vec<String>> {
    on_runtime(async {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "remote-ls", "--updates", "--app", "--columns=application"])
            .output()
            .await?;
        if !out.status.success() {
            return Err(classify_flatpak_error(
                "flatpak remote-ls --updates",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| {
                let id = l.trim();
                if id.is_empty() { None } else { Some(id.to_string()) }
            })
            .collect())
    })
    .await
}

pub fn parse(input: &str) -> Vec<InstalledApp> {
    let mut out = Vec::new();
    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let p: Vec<&str> = line.split('\t').collect();
        if p.len() < 7 {
            continue;
        }
        out.push(InstalledApp {
            app_id: p[0].trim().to_string(),
            name: p[1].trim().to_string(),
            version: p[2].trim().to_string(),
            branch: p[3].trim().to_string(),
            origin: p[4].trim().to_string(),
            size: p[5].trim().to_string(),
            installation: p[6].trim().to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_installed_rows() {
        let raw = "org.kde.kcalc\tKCalc\t23.08\tstable\tflathub\t12.3 MB\tuser\n";
        let r = parse(raw);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].app_id, "org.kde.kcalc");
        assert_eq!(r[0].size, "12.3 MB");
    }
}
