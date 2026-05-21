//! argentum-app-store — entry point.

mod app;
mod pages;
mod sidebar;
mod theme;
mod widgets;

use clap::Parser;
use pages::Page;

#[derive(Parser, Debug)]
#[command(name = "argentum-app-store", version, about = "argentumOS app store — Flathub front-end")]
struct Args {
    /// Page to focus on launch.
    #[arg(long, value_enum, default_value_t = Page::Discover)]
    page: Page,

    /// Optionally jump directly to the detail view for this app id
    /// (e.g. `--app org.kde.kcalc`).
    #[arg(long)]
    app: Option<String>,

    /// Positional URL handler input. Accepts `appstream:<app-id>` and
    /// `mintinstall:<app-id>` forms emitted by xdg-open + the .desktop's
    /// MimeType registration. Falls through to a no-op if it doesn't match.
    url: Option<String>,
}

fn parse_url_deeplink(url: &str) -> Option<String> {
    for scheme in ["appstream:", "mintinstall:"] {
        if let Some(rest) = url.strip_prefix(scheme) {
            let id = rest.trim_start_matches("//").trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    if let Some(url) = args.url.take() {
        if let Some(id) = parse_url_deeplink(&url) {
            args.app = Some(id);
            args.page = Page::Discover;
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Best-effort: ensure cache subdirs exist before we start firing async
    // calls that try to write to them.
    let _ = std::fs::create_dir_all(
        argentum_app_store_core::paths::cache_dir(),
    );
    let _ = std::fs::create_dir_all(
        argentum_app_store_core::paths::icons_cache_dir(),
    );
    let _ = std::fs::create_dir_all(
        argentum_app_store_core::paths::api_cache_dir(),
    );

    app::run(args.page, args.app);
    Ok(())
}
