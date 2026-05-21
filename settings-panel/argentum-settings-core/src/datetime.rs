//! Date & time backend via `timedatectl` (systemd-timedated). All write paths
//! shell through `pkexec` so the system D-Bus polkit action gets prompted.

use crate::{Error, Result, on_runtime};

#[derive(Debug, Clone, Default)]
pub struct DateTimeStatus {
    pub timezone: String,
    pub ntp_enabled: bool,
    /// Local time as rendered by timedatectl (already-formatted string).
    pub local_time: String,
}

pub async fn status() -> Result<DateTimeStatus> {
    on_runtime(async {
        let out = tokio::process::Command::new("timedatectl")
            .args(["show", "--property=Timezone,NTP,TimeUSec"])
            .output()
            .await?;
        if !out.status.success() {
            return Err(Error::Subprocess {
                cmd: "timedatectl show".into(),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(parse_status(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

pub async fn list_timezones() -> Result<Vec<String>> {
    on_runtime(async {
        let out = tokio::process::Command::new("timedatectl")
            .args(["list-timezones"])
            .output()
            .await?;
        if !out.status.success() {
            return Ok(Vec::new());
        }
        let mut v: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        v.sort();
        Ok(v)
    })
    .await
}

pub async fn set_timezone(tz: &str) -> Result<()> {
    let tz = tz.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("pkexec")
            .args(["timedatectl", "set-timezone", &tz])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Subprocess {
                cmd: format!("pkexec timedatectl set-timezone {tz}"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}

pub async fn set_ntp(enabled: bool) -> Result<()> {
    on_runtime(async move {
        let flag = if enabled { "true" } else { "false" };
        let out = tokio::process::Command::new("pkexec")
            .args(["timedatectl", "set-ntp", flag])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Subprocess {
                cmd: format!("pkexec timedatectl set-ntp {flag}"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}

pub fn parse_status(raw: &str) -> DateTimeStatus {
    let mut out = DateTimeStatus::default();
    for line in raw.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        match k.trim() {
            "Timezone" => out.timezone = v.trim().to_string(),
            "NTP" => out.ntp_enabled = v.trim().eq_ignore_ascii_case("yes"),
            "TimeUSec" => out.local_time = v.trim().to_string(),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_show_output() {
        let raw = "Timezone=America/Los_Angeles\nNTP=yes\nTimeUSec=Wed 2026-05-21 12:34:56 PDT\n";
        let r = parse_status(raw);
        assert_eq!(r.timezone, "America/Los_Angeles");
        assert!(r.ntp_enabled);
        assert!(r.local_time.contains("2026-05-21"));
    }
}
