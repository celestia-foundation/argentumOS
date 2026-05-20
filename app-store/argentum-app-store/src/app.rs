//! Root application — owns the sole top-level Entity. Sidebar is rendered
//! inline; each page is its own Entity so caches survive selection changes.

use crate::pages::{Page, PagesView};
use crate::sidebar;
use crate::theme;
use gpui::{
    App as GpuiApp, AppContext as _, Bounds, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowKind, WindowOptions, div, px, rgb, size,
};
use gpui_platform::application;

pub struct App {
    pub selected: Page,
    pub pages: Entity<PagesView>,
}

impl App {
    pub fn new(initial: Page, deep_link_app: Option<String>, cx: &mut Context<Self>) -> Self {
        let pages = cx.new(|cx| PagesView::new(initial, deep_link_app, cx));
        Self { selected: initial, pages }
    }

    pub fn select(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.selected == page {
            return;
        }
        self.selected = page;
        let pages = self.pages.clone();
        pages.update(cx, |p, cx| p.set_active(page, cx));
        cx.notify();
    }
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let app_entity = _cx.entity();
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(theme::BG))
            .text_color(rgb(theme::TEXT))
            .child(sidebar::render(self.selected, app_entity))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.pages.clone()),
            )
    }
}

pub fn run(initial: Page, deep_link_app: Option<String>) {
    application().run(move |cx: &mut GpuiApp| {
        let bounds = Bounds::centered(None, size(px(1200.), px(780.)), cx);
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            window_min_size: Some(size(px(1000.), px(680.))),
            kind: WindowKind::Normal,
            app_id: Some("argentum-app-store".into()),
            ..Default::default()
        };
        let deep = deep_link_app.clone();
        cx.open_window(opts, move |_window, cx| {
            cx.new(|cx| App::new(initial, deep.clone(), cx))
        })
        .expect("open window");
        cx.activate(true);
    });
}
