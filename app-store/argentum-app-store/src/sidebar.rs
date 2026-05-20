//! Sidebar — stateless, selection lives on the parent `App` entity.

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
                .child("app store"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .px(px(8.))
                .child(row(Page::Discover, selected, app.clone()))
                .child(row(Page::Categories, selected, app.clone()))
                .child(row(Page::Search, selected, app.clone()))
                .child(row(Page::Installed, selected, app.clone()))
                .child(row(Page::Updates, selected, app.clone()))
                .child(row(Page::Permissions, selected, app.clone()))
                .child(row(Page::Remotes, selected, app.clone()))
                .child(row(Page::Runtimes, selected, app)),
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
