//! The visual tile used in catalog/category/search grids and detail views.
//!
//! Icon strategy: when an icon URL is supplied, we trigger a background fetch
//! into `~/.cache/argentum-app-store/icons/` so the bytes are on disk by the
//! time GPUI's image primitive is wired up. Until that's verified at the
//! pinned commit, we render the typographic placeholder (the app name's
//! initial in an accent square) — visually unchanged from before, but the
//! cache is now warm.

use crate::theme;
use argentum_app_store_core::icons;
use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct AppCardData {
    pub app_id: String,
    pub name: String,
    pub summary: String,
    pub icon_url: Option<String>,
}

/// Click-handler signature for cards in a grid hosted on page `P`.
pub fn render<P: Render>(
    data: AppCardData,
    cx: &mut Context<P>,
    on_open: impl Fn(&mut P, String, &mut Context<P>) + 'static,
) -> AnyElement {
    let id_for_listener = data.app_id.clone();
    let initial = data.name.chars().next().unwrap_or('?').to_uppercase().to_string();

    // Warm the disk cache for the icon URL so it's ready when the visual
    // swap to `gpui::img(path)` lands. Background spawn, fire-and-forget,
    // no UI consequence.
    if let Some(url) = data.icon_url.as_deref() {
        if icons::cached_path(url).is_none() {
            let url = url.to_string();
            cx.spawn(async move |_weak, _async_cx| {
                let _ = icons::ensure_cached(url).await;
            })
            .detach();
        }
    }

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
