//! Modal text prompt for the app-store binary. Duplicate of the
//! settings-panel widgets/prompt.rs — both will collapse into a shared
//! `argentum-ui` crate in M5. The duplication is intentional until then;
//! both apps must work as standalone Cargo workspaces today.

use anyhow::{Context, Result};
use argentum_app_store_core::on_runtime;
use tokio::process::Command;

/// One-line free-text prompt. `initial` pre-fills the entry box.
pub async fn text(title: &str, label: &str, initial: &str) -> Result<Option<String>> {
    run_zenity(&[
        "--entry",
        "--title",
        title,
        "--text",
        label,
        "--entry-text",
        initial,
    ])
    .await
}

/// Two-field prompt for name + value pairs (flatpak remote name + URL).
pub async fn name_and_url(
    title: &str,
    name_label: &str,
    url_label: &str,
) -> Result<Option<(String, String)>> {
    let out = run_zenity(&[
        "--forms",
        "--title",
        title,
        "--text",
        title,
        "--separator=|",
        &format!("--add-entry={name_label}"),
        &format!("--add-entry={url_label}"),
    ])
    .await?;
    let Some(text) = out else { return Ok(None) };
    let mut parts = text.splitn(2, '|');
    let name = parts.next().unwrap_or("").trim().to_string();
    let url = parts.next().unwrap_or("").trim().to_string();
    if name.is_empty() || url.is_empty() {
        return Ok(None);
    }
    Ok(Some((name, url)))
}

async fn run_zenity(args: &[&str]) -> Result<Option<String>> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    on_runtime(async move {
        let out = Command::new("zenity")
            .args(&args)
            .output()
            .await
            .context("failed to spawn zenity")?;
        match out.status.code() {
            Some(0) => {
                let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                Ok(Some(text))
            }
            Some(1) => Ok(None),
            _ => Err(anyhow::anyhow!(
                "zenity exited unexpectedly: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
        }
    })
    .await
}
