//! Skeleton primitives — drawn in the final layout, never a centered spinner.

use crate::theme;
use gpui::{IntoElement, ParentElement, Styled, div, px, rgb};

/// A flat surface-coloured rectangle, height parameterised so each page can
/// shape its skeleton to match its real rows.
pub fn bar(height: f32) -> impl IntoElement {
    div().h(px(height)).bg(rgb(theme::SURFACE)).rounded(px(10.))
}

/// N stacked bars at the same height.
pub fn rows(n: usize, height: f32) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_3();
    for _ in 0..n {
        col = col.child(bar(height));
    }
    col
}

/// Square placeholder for a grid tile.
#[allow(dead_code)]
pub fn tile() -> impl IntoElement {
    div().h(px(160.)).bg(rgb(theme::SURFACE)).rounded(px(12.))
}

/// Inline error chip used by every page.
pub fn error_view(msg: &str) -> impl IntoElement {
    div()
        .h(px(100.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(format!("Error: {msg}"))
}

/// Inline empty-state chip.
pub fn empty_view(msg: &str) -> impl IntoElement {
    div()
        .h(px(100.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(msg.to_string())
}
