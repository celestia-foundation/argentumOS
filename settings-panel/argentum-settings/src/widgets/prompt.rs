//! Modal text and password prompts.
//!
//! Wraps `zenity` as a subprocess: the dialog is its own GTK window, so it
//! sidesteps the missing GPUI text-input primitive entirely. Returns
//! `Ok(Some(text))` on submit, `Ok(None)` on cancel/close, `Err(_)` only on
//! genuinely unexpected failure (zenity missing, permission error, etc.).
//!
//! Zenity is pulled in by `modules/settings.nix`, so callers can assume it is
//! always on PATH in a built argentumOS image. In a development shell without
//! it, the future resolves to an error instead of panicking.

use anyhow::{Context, Result};
use argentum_settings_core::on_runtime;
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

/// Single masked-entry password prompt.
pub async fn password(title: &str, label: &str) -> Result<Option<String>> {
    // zenity's `--password` doesn't accept `--text`; use `--entry --hide-text`
    // so we can still show a caller-supplied label.
    run_zenity(&[
        "--entry",
        "--hide-text",
        "--title",
        title,
        "--text",
        label,
    ])
    .await
}

/// Two-field prompt for name + value pairs (e.g. flatpak remote name + URL).
/// Returns `(name, url)` on submit. Cancel returns `Ok(None)`.
pub async fn name_and_url(
    title: &str,
    name_label: &str,
    url_label: &str,
) -> Result<Option<(String, String)>> {
    let out = run_zenity_raw(&[
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
    run_zenity_raw(args).await
}

async fn run_zenity_raw(args: &[&str]) -> Result<Option<String>> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    on_runtime(async move {
        let out = Command::new("zenity")
            .args(&args)
            .output()
            .await
            .context("failed to spawn zenity")?;
        // zenity convention: exit 0 = submitted, 1 = cancelled/closed.
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
