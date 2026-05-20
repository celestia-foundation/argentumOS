//! The visual tile used in catalog/category/search grids and detail views.
//!
//! Iconless variant for now — icon loading is wired through
//! `argentum-app-store-core::icons`, but GPUI image binding from in-memory
//! bytes is a follow-up. Card shows a coloured placeholder square with the
//! app name's initial as a typographic stand-in.

use crate::theme;
use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct AppCardData {
    pub app_id: String,
    pub name: String,
    pub summary: String,
}

/// Click-handler signature for cards in a grid hosted on page `P`.
pub fn render<P: Render>(
    data: AppCardData,
    cx: &mut Context<P>,
    on_open: impl Fn(&mut P, String, &mut Context<P>) + 'static,
) -> AnyElement {
    let id_for_listener = data.app_id.clone();
    let initial = data.name.chars().next().unwrap_or('?').to_uppercase().to_string();

    div()
        .id(SharedString::from(format!("app-card:{}", data.app_id)))
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(12.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::SIDEBAR)))
        .on_click(cx.listener(move |this: &mut P, _ev: &ClickEvent, _window: &mut Window, cx| {
            on_open(this, id_for_listener.clone(), cx);
        }))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(64.))
                .h(px(64.))
                .bg(rgb(theme::SIDEBAR))
                .rounded(px(12.))
                .text_color(rgb(theme::ACCENT))
                .child(initial),
        )
        .child(
            div()
                .text_color(rgb(theme::TEXT))
                .child(if data.name.is_empty() {
                    data.app_id.clone()
                } else {
                    data.name
                }),
        )
        .child(
            div()
                .text_color(rgb(theme::TEXT_MUTED))
                .child(truncate(&data.summary, 80)),
        )
        .into_any_element()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.min(s.len())])
    }
}
