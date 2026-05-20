//! Remote app catalog — `flatpak remote-ls`.

use crate::{Result, classify_flatpak_error, on_runtime};
use super::USER;

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub app_id: String,
    pub name: String,
    pub version: String,
    pub branch: String,
    pub arch: String,
    pub origin: String,
}

pub async fn list_remote_apps(remote: &str) -> Result<Vec<CatalogEntry>> {
    let remote = remote.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("flatpak")
            .args([
                USER,
                "remote-ls",
                "--app",
                "--columns=application,name,version,branch,arch,origin",
                &remote,
            ])
            .output()
            .await?;
        if !out.status.success() {
            return Err(classify_flatpak_error(
                format!("flatpak remote-ls {remote}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(parse_entries(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

pub fn parse_entries(input: &str) -> Vec<CatalogEntry> {
    let mut out = Vec::new();
    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let p: Vec<&str> = line.split('\t').collect();
        if p.len() < 6 {
            continue;
        }
        out.push(CatalogEntry {
            app_id: p[0].trim().to_string(),
            name: p[1].trim().to_string(),
            version: p[2].trim().to_string(),
            branch: p[3].trim().to_string(),
            arch: p[4].trim().to_string(),
            origin: p[5].trim().to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_six_columns() {
        let raw = "org.kde.kcalc\tKCalc\t23.08\tstable\tx86_64\tflathub\n";
        let r = parse_entries(raw);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].app_id, "org.kde.kcalc");
        assert_eq!(r[0].origin, "flathub");
    }
}
