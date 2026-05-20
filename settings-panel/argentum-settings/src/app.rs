//! Root application — owns the sole top-level Entity. The sidebar is rendered
//! inline (it's stateless; selection lives on `App`). Each page is its own
//! Entity so caches survive selection changes.

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
    pub fn new(initial: Page, cx: &mut Context<Self>) -> Self {
        let pages = cx.new(|cx| PagesView::new(initial, cx));
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_entity = cx.entity();
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

/// Open the main window and run the event loop.
pub fn run(initial: Page) {
    application().run(move |cx: &mut GpuiApp| {
        let bounds = Bounds::centered(None, size(px(1100.), px(700.)), cx);
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            window_min_size: Some(size(px(900.), px(600.))),
            kind: WindowKind::Normal,
            app_id: Some("argentum-settings".into()),
            ..Default::default()
        };
        cx.open_window(opts, move |_window, cx| cx.new(|cx| App::new(initial, cx)))
            .expect("open window");
        cx.activate(true);
    });
}
