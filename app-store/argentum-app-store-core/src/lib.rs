//! `argentum-app-store-core` — backend logic for the argentumOS app store.
//!
//! Wraps the `flatpak` CLI, parses local AppStream metadata, and fetches rich
//! detail from the Flathub HTTP API. Contains **no UI framework dependencies**
//! — it must remain usable from headless contexts (tests, CLI tools).
//!
//! Bridging into GPUI is the binary crate's responsibility: spawn a task on
//! GPUI's executor, `.await` a function from this crate, then dispatch the
//! result back to the UI.

pub mod appstream;
pub mod flathub_api;
pub mod flatpak;
pub mod icons;
pub mod paths;

/// Lazily-constructed multi-threaded tokio runtime shared by all backend
/// functions. Backend code calls `tokio::fs` / `tokio::process` / `reqwest`,
/// which need a running tokio reactor — but GPUI's executor is smol-based, so
/// naive `cx.spawn(async { backend::foo().await })` panics with "no reactor".
///
/// To bridge: every backend entry point spawns its actual work onto this
/// runtime via [`on_runtime`]. The caller can `await` the wrapper future from
/// any executor (GPUI's, smol's, std futures-executor) and tokio's reactor
/// remains in scope across `.await` points because the work executes on
/// tokio's own worker threads.
fn runtime() -> &'static tokio::runtime::Handle {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("argentum-app-store-tokio")
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

/// Errors any backend function may surface to the UI.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested operation is not yet implemented for this backend.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    /// A subprocess (typically `flatpak`) exited non-zero.
    #[error("`{cmd}` exited {code}: {stderr}")]
    Subprocess { cmd: String, code: i32, stderr: String },

    /// Another flatpak operation holds the system or user lock.
    #[error("flatpak is busy — another operation is in progress")]
    FlatpakBusy,

    /// A filesystem operation failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP / network failure when talking to the Flathub API.
    #[error("http: {0}")]
    Http(String),

    /// Parsing output (flatpak columns, appstream XML, Flathub JSON) failed.
    #[error("parse: {0}")]
    Parse(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Inspect `flatpak` stderr for the well-known "could not lock" / "in use"
/// strings and return [`Error::FlatpakBusy`] in that case; otherwise return
/// the generic `Subprocess` error. Centralised so every backend module
/// presents the same UX-relevant signal.
pub(crate) fn classify_flatpak_error(cmd: impl Into<String>, code: i32, stderr: String) -> Error {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("could not lock")
        || lower.contains("another flatpak")
        || lower.contains("in use by another")
    {
        Error::FlatpakBusy
    } else {
        Error::Subprocess { cmd: cmd.into(), code, stderr }
    }
}
