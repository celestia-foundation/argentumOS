//! Streaming-progress row rendered at the bottom of pages that drive long
//! operations (Discover/install, Installed/uninstall, Updates/update_all,
//! Runtimes/prune). Inline, never modal — matches the UI philosophy.

use crate::theme;
use argentum_app_store_core::flatpak::install::ProgressLine;
use gpui::{IntoElement, ParentElement, Styled, div, px, rgb};

/// Render the most recent ~6 lines of a progress stream.
pub fn render(label: &str, lines: &[ProgressLine], in_flight: bool) -> impl IntoElement {
    let mut block = div()
        .flex()
        .flex_col()
        .gap_1()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.));

    block = block.child(
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(div().text_color(rgb(theme::TEXT)).child(label.to_string()))
            .child(
                div()
                    .text_color(rgb(if in_flight { theme::ACCENT } else { theme::TEXT_MUTED }))
                    .child(if in_flight { "running" } else { "idle" }.to_string()),
            ),
    );

    for line in lines.iter().rev().take(6).rev() {
        let (s, c) = match line {
            ProgressLine::Stdout(s) => (s.clone(), theme::TEXT_MUTED),
            ProgressLine::Stderr(s) => (s.clone(), theme::TEXT_MUTED),
            ProgressLine::Exit(code) => (format!("[exit {code}]"), theme::ACCENT),
        };
        block = block.child(div().text_color(rgb(c)).child(s));
    }

    block
}
