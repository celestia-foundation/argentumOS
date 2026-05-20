//! AppStream catalog parser.
//!
//! `flatpak update --appstream` writes `appstream.xml.gz` under the user and
//! system flatpak roots. The XML lists every app the remote ships, with name,
//! summary, categories, and an icon path relative to the same `active/` dir.
//!
//! We stream-parse with `quick-xml` to keep memory bounded — Flathub's full
//! catalog is ~4–8 MB uncompressed at time of writing.

use crate::{Error, Result, on_runtime, paths};
use flate2::read::GzDecoder;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct AppMeta {
    pub app_id: String,
    pub name: String,
    pub summary: String,
    /// Absolute filesystem path of the best icon we could find for this app
    /// in the remote's AppStream payload, or `None` if no `<icon type="cached">`
    /// element was provided.
    pub icon_path: Option<PathBuf>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub developer: String,
    pub project_license: String,
    pub origin_remote: String,
}

/// Try the user-scope cache first, fall back to system-scope. argentumOS uses
/// `services.flatpak.enable = true` system-wide so the system path is
/// expected to be populated on first boot even before the user has run
/// anything.
pub async fn load_remote(remote: &str) -> Result<Vec<AppMeta>> {
    let remote = remote.to_string();
    on_runtime(async move {
        let mut tried = Vec::new();
        for base in [
            paths::user_appstream_dir(&remote, paths::ARCH),
            paths::system_appstream_dir(&remote, paths::ARCH),
        ] {
            let xml_gz = base.join("appstream.xml.gz");
            tried.push(xml_gz.clone());
            if let Ok(bytes) = tokio::fs::read(&xml_gz).await {
                let entries = decode_and_parse(&bytes, &remote, &base)?;
                return Ok(entries);
            }
        }
        Err(Error::Parse(format!(
            "no appstream.xml.gz found for remote `{remote}` (tried: {:?})",
            tried
        )))
    })
    .await
}

fn decode_and_parse(gz: &[u8], remote: &str, base: &Path) -> Result<Vec<AppMeta>> {
    let mut xml = String::new();
    GzDecoder::new(gz)
        .read_to_string(&mut xml)
        .map_err(|e| Error::Parse(format!("gunzip appstream: {e}")))?;
    Ok(parse_xml(&xml, remote, base))
}

/// Parse the AppStream `<components>` XML. We only extract enough fields to
/// power the catalog grid and search; the detail page is expected to either
/// hit Flathub's API or call `flatpak remote-info` for richer data.
pub fn parse_xml(xml: &str, remote: &str, base: &Path) -> Vec<AppMeta> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut out = Vec::new();
    let mut buf = Vec::new();

    // Per-component scratch state.
    let mut current: Option<AppMeta> = None;
    let mut in_field: Option<Field> = None;
    let mut icon_attrs: HashMap<String, String> = HashMap::new();
    let mut name_lang_preferred = false;
    let mut summary_lang_preferred = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) | Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let tag = std::str::from_utf8(e.name().as_ref()).unwrap_or("").to_string();
                match tag.as_str() {
                    "component" => {
                        // Only ship type="desktop" / "desktop-application"; skip fonts, addons, etc.
                        let mut keep = false;
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"type" {
                                let v = String::from_utf8_lossy(&a.value).into_owned();
                                if v == "desktop" || v == "desktop-application" {
                                    keep = true;
                                }
                            }
                        }
                        current = if keep {
                            Some(AppMeta {
                                origin_remote: remote.to_string(),
                                ..Default::default()
                            })
                        } else {
                            None
                        };
                        name_lang_preferred = false;
                        summary_lang_preferred = false;
                    }
                    "id" if current.is_some() => in_field = Some(Field::Id),
                    "name" if current.is_some() => {
                        let lang = attr_lang(&e);
                        if lang.is_none() && name_lang_preferred {
                            in_field = None;
                        } else if lang.is_none() {
                            in_field = Some(Field::Name);
                        } else if lang.as_deref() == Some("en") || lang.as_deref() == Some("C") {
                            in_field = Some(Field::Name);
                            name_lang_preferred = true;
                        } else {
                            in_field = None;
                        }
                    }
                    "summary" if current.is_some() => {
                        let lang = attr_lang(&e);
                        if lang.is_none() && summary_lang_preferred {
                            in_field = None;
                        } else if lang.is_none() {
                            in_field = Some(Field::Summary);
                        } else if lang.as_deref() == Some("en") || lang.as_deref() == Some("C") {
                            in_field = Some(Field::Summary);
                            summary_lang_preferred = true;
                        } else {
                            in_field = None;
                        }
                    }
                    "developer_name" | "developer" if current.is_some() => {
                        in_field = Some(Field::Developer)
                    }
                    "project_license" if current.is_some() => in_field = Some(Field::License),
                    "category" if current.is_some() => in_field = Some(Field::Category),
                    "keyword" if current.is_some() => in_field = Some(Field::Keyword),
                    "icon" if current.is_some() => {
                        icon_attrs.clear();
                        for a in e.attributes().flatten() {
                            let k = String::from_utf8_lossy(a.key.as_ref()).into_owned();
                            let v = String::from_utf8_lossy(&a.value).into_owned();
                            icon_attrs.insert(k, v);
                        }
                        in_field = Some(Field::Icon);
                    }
                    _ => {}
                }
            }

            Ok(Event::Text(t)) => {
                let Some(comp) = current.as_mut() else { continue };
                let text = t.unescape().unwrap_or_default().into_owned();
                match in_field {
                    Some(Field::Id) => comp.app_id = text.trim().to_string(),
                    Some(Field::Name) => comp.name = text,
                    Some(Field::Summary) => comp.summary = text,
                    Some(Field::Developer) => comp.developer = text,
                    Some(Field::License) => comp.project_license = text,
                    Some(Field::Category) => comp.categories.push(text),
                    Some(Field::Keyword) => comp.keywords.push(text),
                    Some(Field::Icon) => {
                        // Only consider cached icons — the on-disk ones we can
                        // actually display without a network round trip.
                        if icon_attrs.get("type").map(|s| s.as_str()) == Some("cached") {
                            let candidate = base
                                .join("icons")
                                .join(icon_attrs.get("width").cloned().unwrap_or_default())
                                .join(text.trim());
                            // Prefer largest icon we encounter.
                            let new_w = icon_attrs
                                .get("width")
                                .and_then(|w| w.parse::<u32>().ok())
                                .unwrap_or(0);
                            let take = match &comp.icon_path {
                                Some(prev) => {
                                    let prev_w = prev
                                        .parent()
                                        .and_then(|p| p.file_name())
                                        .and_then(|n| n.to_str())
                                        .and_then(|s| s.parse::<u32>().ok())
                                        .unwrap_or(0);
                                    new_w > prev_w
                                }
                                None => true,
                            };
                            if take {
                                comp.icon_path = Some(candidate);
                            }
                        }
                    }
                    None => {}
                }
            }

            Ok(Event::End(e)) => {
                let tag = std::str::from_utf8(e.name().as_ref()).unwrap_or("").to_string();
                if tag == "component" {
                    if let Some(c) = current.take() {
                        if !c.app_id.is_empty() {
                            out.push(c);
                        }
                    }
                } else if tag == "icon" {
                    icon_attrs.clear();
                    in_field = None;
                } else {
                    in_field = None;
                }
            }

            _ => {}
        }
        buf.clear();
    }

    out
}

fn attr_lang(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == b"xml:lang" {
            return Some(String::from_utf8_lossy(&a.value).into_owned());
        }
    }
    None
}

enum Field {
    Id,
    Name,
    Summary,
    Developer,
    License,
    Category,
    Keyword,
    Icon,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_minimal_component() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <components>
              <component type="desktop-application">
                <id>org.kde.kcalc</id>
                <name>KCalc</name>
                <summary>A simple calculator</summary>
                <project_license>GPL-2.0+</project_license>
                <category>Utility</category>
                <icon type="cached" width="128" height="128">org.kde.kcalc.png</icon>
              </component>
              <component type="font">
                <id>org.example.font</id>
                <name>Should be skipped</name>
              </component>
            </components>"#;
        let entries = parse_xml(xml, "flathub", &PathBuf::from("/base"));
        assert_eq!(entries.len(), 1);
        let m = &entries[0];
        assert_eq!(m.app_id, "org.kde.kcalc");
        assert_eq!(m.name, "KCalc");
        assert_eq!(m.summary, "A simple calculator");
        assert_eq!(m.categories, vec!["Utility"]);
        assert!(m.icon_path.as_ref().unwrap().ends_with("icons/128/org.kde.kcalc.png"));
    }
}
