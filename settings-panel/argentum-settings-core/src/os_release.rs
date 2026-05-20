//! Parse `/etc/os-release` into structured fields.

use crate::{Error, Result, on_runtime};

#[derive(Debug, Clone, Default)]
pub struct OsRelease {
    pub name: Option<String>,
    pub id: Option<String>,
    pub pretty_name: Option<String>,
    pub version: Option<String>,
    pub version_codename: Option<String>,
    pub home_url: Option<String>,
}

pub async fn load() -> Result<OsRelease> {
    on_runtime(async {
        let raw = tokio::fs::read_to_string("/etc/os-release").await?;
        Ok(parse(&raw))
    })
    .await
}

pub fn parse(raw: &str) -> OsRelease {
    let mut out = OsRelease::default();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else { continue };
        let val = val.trim().trim_matches('"').to_string();
        match key.trim() {
            "NAME" => out.name = Some(val),
            "ID" => out.id = Some(val),
            "PRETTY_NAME" => out.pretty_name = Some(val),
            "VERSION" => out.version = Some(val),
            "VERSION_CODENAME" => out.version_codename = Some(val),
            "HOME_URL" => out.home_url = Some(val),
            _ => {}
        }
    }
    out
}

pub async fn hostname() -> Result<String> {
    on_runtime(async {
        let raw = tokio::fs::read_to_string("/etc/hostname").await?;
        Ok(raw.trim().to_string())
    })
    .await
}

pub async fn set_hostname(new: &str) -> Result<()> {
    let new = new.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("pkexec")
            .args(["hostnamectl", "set-hostname", &new])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Subprocess {
                cmd: format!("pkexec hostnamectl set-hostname {new}"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_argentumos_os_release() {
        let raw = r#"
NAME="argentumOS"
ID=argentumos
ID_LIKE=nixos
PRETTY_NAME="argentumOS"
HOME_URL="https://github.com/"
"#;
        let r = parse(raw);
        assert_eq!(r.name.as_deref(), Some("argentumOS"));
        assert_eq!(r.id.as_deref(), Some("argentumos"));
        assert_eq!(r.pretty_name.as_deref(), Some("argentumOS"));
        assert_eq!(r.home_url.as_deref(), Some("https://github.com/"));
    }

    #[test]
    fn ignores_unknown_keys_and_comments() {
        let raw = "# comment\nNAME=foo\nBOGUS=bar\n";
        let r = parse(raw);
        assert_eq!(r.name.as_deref(), Some("foo"));
    }
}
