//! `argentum-settings-core` — backend logic for the argentumOS settings panel.
//!
//! This crate exposes async functions that read and mutate system state via
//! D-Bus and subprocess calls. It contains **no UI framework dependencies** —
//! it must remain usable from headless contexts (tests, CLI tools).
//!
//! Bridging into a UI framework (GPUI in the case of the binary crate) is the
//! caller's responsibility: spawn a task on the framework's executor, await a
//! function from this crate, then dispatch the result back to the UI.

pub mod datetime;
pub mod display;
pub mod os_release;
pub mod sound;
pub mod system;
pub mod theme_scan;
pub mod wallpaper_scan;

pub mod dbus;

/// Lazily-constructed multi-threaded tokio runtime shared by all backend
/// functions. Backend code calls `tokio::fs` / `tokio::process`, which need a
/// running tokio reactor — but GPUI's executor is smol-based, so naive
/// `cx.spawn(async { backend::foo().await })` panics with "no reactor".
///
/// To bridge: every backend entry point spawns its actual work onto this
/// runtime via [`on_runtime`]. The caller can `await` the wrapper future
/// from any executor (GPUI's, smol's, std futures-executor) and tokio's
/// reactor remains in scope across `.await` points because the work
/// executes on tokio's own worker threads.
fn runtime() -> &'static tokio::runtime::Handle {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("argentum-settings-tokio")
            .build()
            .expect("build tokio runtime")
    })
    .handle()
}

/// Run a future on the shared tokio runtime so tokio I/O works regardless of
/// the calling executor.
pub async fn on_runtime<F, R>(fut: F) -> R
where
    F: std::future::Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    runtime().spawn(fut).await.expect("backend task panicked")
}

/// Errors that any backend function may surface to the UI.
///
/// The UI binary interprets these to choose between optimistic-update rollback
/// (any error reverts local state) and inline error messaging.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested operation is not yet implemented for this backend.
    /// Used by stubs (Wayland display, change-password, WiFi password).
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    /// A subprocess (xrandr, gsettings, flatpak, hostnamectl) exited non-zero.
    #[error("`{cmd}` exited {code}: {stderr}")]
    Subprocess { cmd: String, code: i32, stderr: String },

    /// A D-Bus call failed.
    #[error("D-Bus error: {0}")]
    DBus(String),

    /// A filesystem operation failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing output (xrandr, os-release, flatpak remotes) failed.
    #[error("parse: {0}")]
    Parse(String),

    /// A secured WiFi network requires credentials the UI hasn't collected yet.
    #[error("credentials required")]
    NeedsCredentials,
}

impl From<zbus::Error> for Error {
    fn from(e: zbus::Error) -> Self {
        Error::DBus(e.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
