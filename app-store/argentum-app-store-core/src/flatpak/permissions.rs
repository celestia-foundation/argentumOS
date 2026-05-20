//! Flatseal-style per-app permission overrides.
//!
//! Override files live at `~/.local/share/flatpak/overrides/<app-id>` and use
//! the freedesktop keyfile format (`man flatpak-override`). They take the
//! shape:
//!
//! ```ini
//! [Context]
//! shared=network;ipc
//! sockets=x11;wayland;!fallback-x11
//! devices=dri
//! filesystems=home;~/Music:ro
//! features=devel
//! persistent=.minecraft
//!
//! [Session Bus Policy]
//! org.freedesktop.Flatpak=talk
//!
//! [System Bus Policy]
//! org.freedesktop.UPower=talk
//!
//! [Environment]
//! GTK_DEBUG=interactive
//! ```
//!
//! We treat this file as a typed struct round-trip. We **do not** merge with
//! the app manifest's declared permissions — we simply store overrides as the
//! user expresses them. The UI is responsible for displaying both "what the
//! manifest declares" and "what the user has overridden".

use crate::{Result, on_runtime, paths};
use std::collections::BTreeMap;

/// Parsed `<override>` file. Empty `Vec`s mean "no override entries in that
/// list" (which the keyfile represents by omitting the key entirely).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverrideSet {
    pub shared: Vec<String>,
    pub sockets: Vec<String>,
    pub devices: Vec<String>,
    pub filesystems: Vec<String>,
    pub features: Vec<String>,
    pub persistent: Vec<String>,
    pub session_bus: BTreeMap<String, String>,
    pub system_bus: BTreeMap<String, String>,
    pub environment: BTreeMap<String, String>,
}

pub async fn read_override(app_id: &str) -> Result<OverrideSet> {
    let app_id = app_id.to_string();
    on_runtime(async move {
        let path = paths::user_overrides_dir().join(&app_id);
        match tokio::fs::read_to_string(&path).await {
            Ok(s) => Ok(parse(&s)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(OverrideSet::default()),
            Err(e) => Err(e.into()),
        }
    })
    .await
}

pub async fn write_override(app_id: &str, set: OverrideSet) -> Result<()> {
    let app_id = app_id.to_string();
    on_runtime(async move {
        let dir = paths::user_overrides_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let final_path = dir.join(&app_id);
        let tmp_path = dir.join(format!(".{app_id}.tmp"));
        let body = serialize(&set);
        tokio::fs::write(&tmp_path, body.as_bytes()).await?;
        tokio::fs::rename(&tmp_path, &final_path).await?;
        Ok(())
    })
    .await
}

/// Strip the override entirely — equivalent to deleting the file.
pub async fn clear_override(app_id: &str) -> Result<()> {
    let app_id = app_id.to_string();
    on_runtime(async move {
        let path = paths::user_overrides_dir().join(&app_id);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    })
    .await
}

pub fn parse(s: &str) -> OverrideSet {
    let mut out = OverrideSet::default();
    let mut section = "";
    for raw in s.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            section = match rest {
                "Context" => "context",
                "Session Bus Policy" => "session-bus",
                "System Bus Policy" => "system-bus",
                "Environment" => "environment",
                _ => "",
            };
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let k = k.trim();
        let v = v.trim();
        match section {
            "context" => {
                let list = split_list(v);
                match k {
                    "shared" => out.shared = list,
                    "sockets" => out.sockets = list,
                    "devices" => out.devices = list,
                    "filesystems" => out.filesystems = list,
                    "features" => out.features = list,
                    "persistent" => out.persistent = list,
                    _ => {}
                }
            }
            "session-bus" => { out.session_bus.insert(k.into(), v.into()); }
            "system-bus" => { out.system_bus.insert(k.into(), v.into()); }
            "environment" => { out.environment.insert(k.into(), v.into()); }
            _ => {}
        }
    }
    out
}

pub fn serialize(set: &OverrideSet) -> String {
    let mut s = String::new();

    let context_has_any = !set.shared.is_empty()
        || !set.sockets.is_empty()
        || !set.devices.is_empty()
        || !set.filesystems.is_empty()
        || !set.features.is_empty()
        || !set.persistent.is_empty();
    if context_has_any {
        s.push_str("[Context]\n");
        push_list(&mut s, "shared", &set.shared);
        push_list(&mut s, "sockets", &set.sockets);
        push_list(&mut s, "devices", &set.devices);
        push_list(&mut s, "filesystems", &set.filesystems);
        push_list(&mut s, "features", &set.features);
        push_list(&mut s, "persistent", &set.persistent);
        s.push('\n');
    }

    push_map(&mut s, "Session Bus Policy", &set.session_bus);
    push_map(&mut s, "System Bus Policy", &set.system_bus);
    push_map(&mut s, "Environment", &set.environment);

    s
}

fn split_list(v: &str) -> Vec<String> {
    v.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn push_list(s: &mut String, key: &str, list: &[String]) {
    if list.is_empty() {
        return;
    }
    s.push_str(key);
    s.push('=');
    s.push_str(&list.join(";"));
    s.push('\n');
}

fn push_map(s: &mut String, header: &str, map: &BTreeMap<String, String>) {
    if map.is_empty() {
        return;
    }
    s.push('[');
    s.push_str(header);
    s.push_str("]\n");
    for (k, v) in map {
        s.push_str(k);
        s.push('=');
        s.push_str(v);
        s.push('\n');
    }
    s.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let mut set = OverrideSet {
            shared: vec!["network".into(), "ipc".into()],
            filesystems: vec!["home".into(), "~/Music:ro".into()],
            ..Default::default()
        };
        set.session_bus.insert("org.freedesktop.Flatpak".into(), "talk".into());

        let s = serialize(&set);
        let parsed = parse(&s);
        assert_eq!(parsed, set);
    }

    #[test]
    fn parses_empty() {
        let parsed = parse("");
        assert!(parsed.shared.is_empty());
        assert!(parsed.environment.is_empty());
    }
}
