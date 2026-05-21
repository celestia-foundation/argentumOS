//! Lazy icon loader. Each call returns either:
//!   - cached bytes from the local AppStream icon path (instant), or
//!   - cached bytes from `~/.cache/argentum-app-store/icons/<hash>`, or
//!   - freshly-downloaded bytes from a Flathub CDN URL (with disk cache write).
//!
//! GPUI's image rendering accepts raw PNG/JPEG bytes; we don't decode here.

use crate::{Error, Result, on_runtime, paths};
use std::path::PathBuf;

pub async fn load(source: IconSource) -> Result<Vec<u8>> {
    on_runtime(async move {
        match source {
            IconSource::Local(p) => Ok(tokio::fs::read(&p).await?),
            IconSource::Url(u) => fetch_url(u).await,
        }
    })
    .await
}

pub enum IconSource {
    Local(PathBuf),
    Url(String),
}

/// Synchronous cache lookup. Returns `Some(path)` if the file is already on
/// disk (no I/O beyond a `metadata()` syscall). UI render paths call this on
/// every frame: cheap, branchless, never blocks.
pub fn cached_path(url: &str) -> Option<PathBuf> {
    let p = cache_path_for(url);
    if p.exists() { Some(p) } else { None }
}

/// Fetch into the disk cache if not already there. Returns the cache path on
/// success. Idempotent — calling repeatedly is safe and cheap once cached.
pub async fn ensure_cached(url: String) -> Result<PathBuf> {
    on_runtime(async move {
        let path = cache_path_for(&url);
        if path.exists() {
            return Ok(path);
        }
        let _ = fetch_url(url.clone()).await?;
        Ok(path)
    })
    .await
}

async fn fetch_url(url: String) -> Result<Vec<u8>> {
    let cached_path = cache_path_for(&url);
    if let Ok(bytes) = tokio::fs::read(&cached_path).await {
        return Ok(bytes);
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!("argentum-app-store/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(Error::Http(format!("{} -> {}", url, resp.status())));
    }
    let bytes = resp.bytes().await?.to_vec();
    if let Some(parent) = cached_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Err(e) = tokio::fs::write(&cached_path, &bytes).await {
        tracing::debug!(?e, ?cached_path, "icon cache write failed");
    }
    Ok(bytes)
}

fn cache_path_for(url: &str) -> PathBuf {
    let hash = stable_hash(url);
    let ext = url
        .rsplit_once('.')
        .map(|(_, e)| e)
        .filter(|e| e.len() <= 5)
        .unwrap_or("img");
    paths::icons_cache_dir().join(format!("{hash:016x}.{ext}"))
}

/// FNV-1a over the URL bytes — deterministic, no external dep, collision
/// space is fine for a per-user icon cache (10k+ apps).
fn stable_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
