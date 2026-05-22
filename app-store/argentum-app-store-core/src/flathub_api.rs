//! Flathub HTTP API client.
//!
//! Endpoints we use (v2):
//!   - `GET /api/v2/appstream/{app_id}`           rich detail
//!   - `GET /api/v2/collection/popular`           featured grid (popular)
//!   - `GET /api/v2/collection/recently-added`
//!   - `GET /api/v2/collection/category/{name}`   category browse
//!
//! Note: a previous shape took a `/{limit}` path suffix (e.g.
//! `.../popular/30`); the current API rejects that with 404. Pagination is
//! query-param based now (`?page=…&per_page=…`) but the default first page
//! returns several hundred hits, so we just slice client-side.
//!
//! All responses are cached to `~/.cache/argentum-app-store/api/<key>.json`.
//! On any network failure we fall back to the cached copy if one exists.

use crate::{Error, Result, on_runtime, paths};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE: &str = "https://flathub.org/api/v2";
// 5s was too aggressive in a fresh QEMU image — DNS warm-up + first TLS
// handshake to the Flathub CDN routinely runs 3-7s. Anything past 15s is a
// real failure, not a slow link.
const TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppDetail {
    pub flatpak_app_id: String,
    pub name: String,
    pub summary: String,
    /// Markdown.
    pub description: String,
    pub project_license: String,
    pub developer_name: String,
    pub icon: Option<String>,
    pub screenshots: Vec<Screenshot>,
    pub verified: bool,
    pub installs_total: Option<u64>,
    pub donation_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Screenshot {
    pub thumb_url: Option<String>,
    pub full_url: Option<String>,
}

/// Subset of the JSON we care about — Flathub's response has more fields, we
/// ignore them with serde's default-on-missing behaviour.
#[derive(Debug, Deserialize)]
struct RawAppDetail {
    #[serde(default)]
    flatpak_app_id: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    project_license: Option<String>,
    #[serde(default)]
    developer_name: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    screenshots: Option<Vec<RawScreenshot>>,
    #[serde(default)]
    verified: Option<bool>,
    #[serde(default)]
    installs_total: Option<u64>,
    #[serde(default)]
    donation_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawScreenshot {
    #[serde(default)]
    thumb_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    sizes: Option<serde_json::Value>,
}

/// Item shape used by collection endpoints (`popular`, `recently-added`,
/// `category/...`). Flathub returns a `{ hits: [...] }` list of these.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSummary {
    pub app_id: String,
    pub name: String,
    pub summary: String,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CollectionResponse {
    #[serde(default)]
    hits: Vec<RawSummary>,
}

// The collection endpoints return both `id` (underscored form, e.g.
// `com_discordapp_Discord`) and `app_id` (dotted, the canonical flatpak ref
// `com.discordapp.Discord`). Serde aliases can't share a field when both
// keys are present in the same object — that produces the "duplicate field"
// parse error — so we deserialize each separately and prefer `app_id`,
// falling back to `flatpak_app_id` then `id` (dot-restored).
#[derive(Debug, Deserialize)]
struct RawSummary {
    #[serde(default)]
    app_id: Option<String>,
    #[serde(default)]
    flatpak_app_id: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

pub async fn app_detail(app_id: &str) -> Result<AppDetail> {
    let app_id = app_id.to_string();
    on_runtime(async move {
        let key = format!("appstream-{app_id}");
        let url = format!("{BASE}/appstream/{app_id}");
        match fetch_json::<RawAppDetail>(&url, &key).await {
            Ok(raw) => Ok(into_detail(raw, &app_id)),
            Err(e) => {
                if let Some(cached) = read_cache::<RawAppDetail>(&key).await {
                    tracing::warn!(?e, "flathub api failed, using cache");
                    Ok(into_detail(cached, &app_id))
                } else {
                    Err(e)
                }
            }
        }
    })
    .await
}

pub async fn popular(limit: u32) -> Result<Vec<AppSummary>> {
    fetch_collection("collection/popular".into(), limit).await
}

pub async fn recently_added(limit: u32) -> Result<Vec<AppSummary>> {
    fetch_collection("collection/recently-added".into(), limit).await
}

pub async fn category(name: &str, limit: u32) -> Result<Vec<AppSummary>> {
    let path = format!("collection/category/{name}");
    fetch_collection(path, limit).await
}

async fn fetch_collection(path: String, limit: u32) -> Result<Vec<AppSummary>> {
    on_runtime(async move {
        let key = format!("collection-{}", path.replace('/', "_"));
        let url = format!("{BASE}/{path}");
        let take_n = limit as usize;
        match fetch_json::<CollectionResponse>(&url, &key).await {
            Ok(resp) => Ok(into_summaries(resp.hits).into_iter().take(take_n).collect()),
            Err(e) => {
                if let Some(cached) = read_cache::<CollectionResponse>(&key).await {
                    tracing::warn!(?e, "flathub api failed, using cache");
                    Ok(into_summaries(cached.hits).into_iter().take(take_n).collect())
                } else {
                    Err(e)
                }
            }
        }
    })
    .await
}

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str, cache_key: &str) -> Result<T> {
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .user_agent(concat!("argentum-app-store/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client.get(url).send().await.map_err(|e| {
        tracing::warn!(?e, url, "flathub fetch failed");
        e
    })?;
    if !resp.status().is_success() {
        return Err(Error::Http(format!("{} -> {}", url, resp.status())));
    }
    let body = resp.bytes().await?;
    write_cache_raw(cache_key, &body).await;
    let v: T = serde_json::from_slice(&body).map_err(|e| Error::Parse(e.to_string()))?;
    Ok(v)
}

async fn write_cache_raw(key: &str, body: &[u8]) {
    let path = paths::api_cache_dir().join(format!("{key}.json"));
    if let Err(e) = tokio::fs::write(&path, body).await {
        tracing::debug!(?e, ?path, "api cache write failed");
    }
}

async fn read_cache<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    let path = paths::api_cache_dir().join(format!("{key}.json"));
    let bytes = tokio::fs::read(&path).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn into_detail(raw: RawAppDetail, app_id: &str) -> AppDetail {
    let flatpak_app_id = raw
        .flatpak_app_id
        .or(raw.id)
        .unwrap_or_else(|| app_id.to_string());
    AppDetail {
        flatpak_app_id,
        name: raw.name.unwrap_or_default(),
        summary: raw.summary.unwrap_or_default(),
        description: raw.description.unwrap_or_default(),
        project_license: raw.project_license.unwrap_or_default(),
        developer_name: raw.developer_name.unwrap_or_default(),
        icon: raw.icon,
        screenshots: raw
            .screenshots
            .unwrap_or_default()
            .into_iter()
            .map(|s| Screenshot { thumb_url: s.thumb_url.clone(), full_url: pick_full(s) })
            .collect(),
        verified: raw.verified.unwrap_or(false),
        installs_total: raw.installs_total,
        donation_url: raw.donation_url,
    }
}

fn pick_full(s: RawScreenshot) -> Option<String> {
    if let Some(u) = s.url {
        return Some(u);
    }
    // `sizes` is a free-form object on Flathub's side — best-effort extraction
    // of any string value as the full image URL.
    let sizes = s.sizes?;
    if let serde_json::Value::Object(m) = sizes {
        for (_k, v) in m {
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn into_summaries(raw: Vec<RawSummary>) -> Vec<AppSummary> {
    raw.into_iter()
        .filter_map(|r| {
            let id = r
                .app_id
                .or(r.flatpak_app_id)
                .or_else(|| r.id.map(|s| s.replace('_', ".")))?;
            if id.is_empty() {
                return None;
            }
            Some(AppSummary {
                app_id: id,
                name: r.name.unwrap_or_default(),
                summary: r.summary.unwrap_or_default(),
                icon: r.icon,
            })
        })
        .collect()
}

/// Static list of Flathub categories — mirrors the AppStream spec's standard
/// `freedesktop.org` set, which is what Flathub's `/collection/category/...`
/// endpoint accepts.
pub fn standard_categories() -> &'static [&'static str] {
    &[
        "AudioVideo",
        "Development",
        "Education",
        "Game",
        "Graphics",
        "Network",
        "Office",
        "Science",
        "System",
        "Utility",
    ]
}
