//! Flatpak remote management (user scope).

use crate::{Error, Result, classify_flatpak_error, on_runtime};
use super::USER;

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

pub async fn list_remotes() -> Result<Vec<Remote>> {
    on_runtime(async {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "remotes", "--columns=name,url,options"])
            .output()
            .await;
        let out = match out {
            Ok(o) if o.status.success() => o,
            Ok(o) => {
                return Err(classify_flatpak_error(
                    "flatpak remotes",
                    o.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&o.stderr).into_owned(),
                ));
            }
            Err(_) => return Ok(Vec::new()),
        };
        Ok(parse_remotes(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
}

pub async fn add_remote(name: &str, url: &str) -> Result<()> {
    let name = name.to_string();
    let url = url.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "remote-add", "--if-not-exists", &name, &url])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(classify_flatpak_error(
                format!("flatpak remote-add {name} {url}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ))
        }
    })
    .await
}

pub async fn remove_remote(name: &str) -> Result<()> {
    let name = name.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "remote-delete", "--force", &name])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(classify_flatpak_error(
                format!("flatpak remote-delete {name}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ))
        }
    })
    .await
}

pub async fn set_enabled(name: &str, enabled: bool) -> Result<()> {
    let name = name.to_string();
    on_runtime(async move {
        let flag = if enabled { "--enable" } else { "--disable" };
        let out = tokio::process::Command::new("flatpak")
            .args([USER, "remote-modify", flag, &name])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(classify_flatpak_error(
                format!("flatpak remote-modify {flag} {name}"),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ))
        }
    })
    .await
}

pub fn parse_remotes(input: &str) -> Vec<Remote> {
    let mut out = Vec::new();
    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts[0].trim().to_string();
        let url = parts[1].trim().to_string();
        let options = parts.get(2).map(|s| s.trim()).unwrap_or("");
        let enabled = !options.split(',').any(|o| o.trim() == "disabled");
        out.push(Remote { name, url, enabled });
    }
    out
}

// `Error` is `pub use`d via the module path; suppress unused-import lint if
// nothing in this file ends up referencing it directly.
#[allow(dead_code)]
fn _error_ref(_e: Error) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_columns_output() {
        let raw = "flathub\thttps://dl.flathub.org/repo/\t\nbeta\thttps://example/\tdisabled\n";
        let r = parse_remotes(raw);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].name, "flathub");
        assert!(r[0].enabled);
        assert!(!r[1].enabled);
    }
}
