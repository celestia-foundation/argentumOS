//! Installed runtime / SDK enumeration + per-runtime uninstall.
//!
//! Pruning ("Uninstall unused") streams progress so the UI can show it like
//! any other install/uninstall — see [`crate::flatpak::install::prune_unused`].

use crate::{Result, classify_flatpak_error, on_runtime};
use super::USER;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub id: String,
    pub name: String,
    pub version: String,
    pub branch: String,
    pub origin: String,
    pub size: String,
}

pub async fn list_runtimes() -> Result<Vec<Runtime>> {
    on_runtime(async {
        let out = tokio::process::Command::new("flatpak")
            .args([
                USER,
                "list",
                "--runtime",
                "--columns=application,name,version,branch,origin,size",
            ])
            .output()
            .await?;
        if !out.status.success() {
            return Err(classify_flatpak_error(
                "flatpak list --runtime",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }
        Ok(parse(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

pub async fn uninstall_runtime(id: &str) -> Result<()> {
    let id = id.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "uninstall", "--runtime", "--assumeyes", &id])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(classify_flatpak_error(
                format!("flatpak uninstall --runtime {id}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ))
        }
    })
    .await
}

pub fn parse(input: &str) -> Vec<Runtime> {
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
        out.push(Runtime {
            id: p[0].trim().to_string(),
            name: p[1].trim().to_string(),
            version: p[2].trim().to_string(),
            branch: p[3].trim().to_string(),
            origin: p[4].trim().to_string(),
            size: p[5].trim().to_string(),
        });
    }
    out
}
