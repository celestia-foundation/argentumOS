//! argentum-settings — entry point.

mod app;
mod pages;
mod sidebar;
mod theme;
mod widgets;

use clap::Parser;
use pages::Page;

#[derive(Parser, Debug)]
#[command(name = "argentum-settings", version, about = "argentumOS system settings panel")]
struct Args {
    /// Page to focus on launch.
    #[arg(long, value_enum, default_value_t = Page::Appearance)]
    page: Page,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    app::run(args.page);
    Ok(())
}
