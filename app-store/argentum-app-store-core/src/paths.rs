//! Filesystem locations the app store reads and writes — user-scoped only.
//!
//! All install operations target `--user`, so all of these are under `$HOME`.

use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// `~/.local/share/flatpak` — user-scope flatpak installation root.
pub fn user_flatpak_root() -> PathBuf {
    home().join(".local/share/flatpak")
}

/// `~/.local/share/flatpak/overrides` — Flatseal-style per-app override keyfiles.
pub fn user_overrides_dir() -> PathBuf {
    user_flatpak_root().join("overrides")
}

/// `~/.local/share/flatpak/appstream/<remote>/<arch>/active/appstream.xml.gz`
/// is the canonical layout `flatpak update --appstream` writes to.
pub fn user_appstream_dir(remote: &str, arch: &str) -> PathBuf {
    user_flatpak_root()
        .join("appstream")
        .join(remote)
        .join(arch)
        .join("active")
}

/// `/var/lib/flatpak/appstream/<remote>/<arch>/active/...` — system-scope
/// AppStream cache, populated by `services.flatpak.enable = true`.
pub fn system_appstream_dir(remote: &str, arch: &str) -> PathBuf {
    PathBuf::from("/var/lib/flatpak/appstream")
        .join(remote)
        .join(arch)
        .join("active")
}

/// `~/.cache/argentum-app-store` — disk cache for Flathub API responses and
/// fetched icons / screenshots.
pub fn cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".cache"))
        .join("argentum-app-store")
}

/// Per-subsystem cache subdirectories — call [`ensure_cache_dirs`] once on
/// startup so reads and writes can assume they exist.
pub fn icons_cache_dir() -> PathBuf {
    cache_dir().join("icons")
}

pub fn api_cache_dir() -> PathBuf {
    cache_dir().join("api")
}

/// Best-effort: create every cache subdirectory. Errors are logged, not
/// returned — the app starts up either way.
pub async fn ensure_cache_dirs() {
    for d in [cache_dir(), icons_cache_dir(), api_cache_dir()] {
        if let Err(e) = tokio::fs::create_dir_all(&d).await {
            tracing::warn!(?d, ?e, "failed to ensure cache dir");
        }
    }
}

/// argentumOS only ships x86_64-linux, so this is the only arch we ever look
/// for in the AppStream cache. If we ever expand: read from
/// `/var/lib/flatpak/repo/config` `default-arch`.
pub const ARCH: &str = "x86_64";
