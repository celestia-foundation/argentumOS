//! Sidebar — rendered inline from `App::render`. Stateless; selection lives
//! on the parent `App` entity, which we receive as a handle so click handlers
//! can update it.
//!
//! Selection is painted as a background on the selected row itself, not as a
//! separately-positioned overlay — an `.absolute()` overlay covers the row's
//! hit area and blocks `on_click`. Animating a sliding indicator while keeping
//! clicks pass-through requires either `pointer-events: none` (not exposed in
//! GPUI today) or per-row background-color tweening; the second option is the
//! follow-up, the first option doesn't exist.

use crate::app::App;
use crate::pages::Page;
use crate::theme;
use gpui::{
    Entity, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    div, px, rgb,
};

const SIDEBAR_WIDTH: f32 = 220.0;
const ROW_HEIGHT: f32 = 44.0;
const TOP_PAD: f32 = 16.0;
const HEADER_HEIGHT: f32 = 48.0;

pub fn render(selected: Page, app: Entity<App>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w(px(SIDEBAR_WIDTH))
        .h_full()
        .bg(rgb(theme::SIDEBAR))
        .pt(px(TOP_PAD))
        .child(
            div()
                .h(px(HEADER_HEIGHT))
                .px_4()
                .text_color(rgb(theme::ACCENT))
                .child("argentum-settings"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .px(px(8.))
                .child(row(Page::Appearance, selected, app.clone()))
                .child(row(Page::Display, selected, app.clone()))
                .child(row(Page::Sound, selected, app.clone()))
                .child(row(Page::Network, selected, app.clone()))
                .child(row(Page::Users, selected, app.clone()))
                .child(row(Page::Software, selected, app.clone()))
                .child(row(Page::DateTime, selected, app.clone()))
                .child(row(Page::System, selected, app)),
        )
}

fn row(page: Page, selected: Page, app: Entity<App>) -> impl IntoElement {
    let is_selected = page == selected;
    let mut row = div()
        .id(("sidebar-row", page as usize))
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .h(px(ROW_HEIGHT))
        .rounded(px(8.))
        .text_color(if is_selected { rgb(theme::TEXT) } else { rgb(theme::TEXT_MUTED) })
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::SURFACE)).text_color(rgb(theme::TEXT)))
        .on_click(move |_event, _window, cx| {
            app.update(cx, |app, cx| app.select(page, cx));
        })
        .child(div().w(px(20.)).child(page.icon()))
        .child(page.label());
    if is_selected {
        row = row.bg(rgb(theme::SURFACE));
    }
    row
}
