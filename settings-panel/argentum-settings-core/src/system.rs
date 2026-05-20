//! System actions: streaming `nixos-rebuild switch --upgrade`.

use crate::{Result, on_runtime};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum LogLine {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

pub async fn run_upgrade() -> Result<mpsc::UnboundedReceiver<LogLine>> {
    on_runtime(async {
        let mut cmd = tokio::process::Command::new("pkexec");
        cmd.args(["nixos-rebuild", "switch", "--upgrade"]);
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
                    if tx.send(LogLine::Stdout(line)).is_err() {
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
                    if tx.send(LogLine::Stderr(line)).is_err() {
                        break;
                    }
                }
            });
        }
        tokio::spawn(async move {
            if let Ok(status) = child.wait().await {
                let _ = tx.send(LogLine::Exit(status.code().unwrap_or(-1)));
            }
        });

        Ok(rx)
    })
    .await
}
