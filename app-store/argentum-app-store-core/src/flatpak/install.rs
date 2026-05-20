//! Streaming install / update operations. Mirrors the
//! `argentum-settings-core::system::run_upgrade` mpsc pattern: spawn the
//! subprocess, fan stdout/stderr line-by-line into an unbounded channel, and
//! deliver a final `Exit(code)` so the UI can stop the progress row.

use crate::{Result, on_runtime};
use super::USER;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum ProgressLine {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

/// Install `app_id` from `remote`. Streams `flatpak` output line by line.
pub async fn install(remote: &str, app_id: &str) -> Result<mpsc::UnboundedReceiver<ProgressLine>> {
    let remote = remote.to_string();
    let app_id = app_id.to_string();
    spawn_stream(vec![
        USER.into(),
        "install".into(),
        "--assumeyes".into(),
        "--noninteractive".into(),
        remote,
        app_id,
    ])
    .await
}

/// Update a single installed app.
pub async fn update(app_id: &str) -> Result<mpsc::UnboundedReceiver<ProgressLine>> {
    let app_id = app_id.to_string();
    spawn_stream(vec![
        USER.into(),
        "update".into(),
        "--assumeyes".into(),
        "--noninteractive".into(),
        app_id,
    ])
    .await
}

/// Update every installed app + runtime that has a pending update.
pub async fn update_all() -> Result<mpsc::UnboundedReceiver<ProgressLine>> {
    spawn_stream(vec![
        USER.into(),
        "update".into(),
        "--assumeyes".into(),
        "--noninteractive".into(),
    ])
    .await
}

/// `flatpak uninstall --unused` — prune runtimes / extensions nothing
/// installed currently depends on.
pub async fn prune_unused() -> Result<mpsc::UnboundedReceiver<ProgressLine>> {
    spawn_stream(vec![
        USER.into(),
        "uninstall".into(),
        "--unused".into(),
        "--assumeyes".into(),
        "--noninteractive".into(),
    ])
    .await
}

async fn spawn_stream(args: Vec<String>) -> Result<mpsc::UnboundedReceiver<ProgressLine>> {
    on_runtime(async move {
        let mut cmd = tokio::process::Command::new("flatpak");
        cmd.args(&args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn()?;

        let (tx, rx) = mpsc::unbounded_channel();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(out) = stdout {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(out).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx.send(ProgressLine::Stdout(line)).is_err() {
                        break;
                    }
                }
            });
        }
        if let Some(err) = stderr {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx.send(ProgressLine::Stderr(line)).is_err() {
                        break;
                    }
                }
            });
        }
        tokio::spawn(async move {
            if let Ok(status) = child.wait().await {
                let _ = tx.send(ProgressLine::Exit(status.code().unwrap_or(-1)));
            }
        });

        Ok(rx)
    })
    .await
}
