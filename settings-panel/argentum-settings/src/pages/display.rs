//! Display page.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::display::{self as backend, Monitor};
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};

pub struct DisplayPage {
    state: PageState<Vec<Monitor>>,
}

impl DisplayPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::query().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(m) => PageState::Loaded { data: m, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }
}

impl Render for DisplayPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Display.ttl()) {
            self.spawn_refresh(cx);
        }
        let body = match &self.state {
            PageState::Loaded { data, .. } if data.is_empty() => empty_view(),
            PageState::Loaded { data, .. } => monitors_view(data),
            PageState::Error(e) => error_view(e),
            _ => skeleton(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Display"))
            .child(body)
    }
}

fn skeleton() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().h(px(140.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
        .child(div().h(px(140.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
}

fn empty_view() -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(200.))
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child("No displays detected. (Wayland backend coming soon — TODO.)")
}

fn error_view(msg: &str) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(200.))
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(format!("Couldn't query displays: {msg}"))
}

fn monitors_view(monitors: &[Monitor]) -> gpui::Div {
    let mut col = div().flex().flex_col().gap_3();
    for m in monitors {
        col = col.child(monitor_card(m));
    }
    col
}

fn monitor_card(m: &Monitor) -> gpui::Div {
    let cur = m.current.as_ref().map(|c| c.label()).unwrap_or_else(|| "off".into());
    div()
        .flex()
        .flex_col()
        .gap_2()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(div().text_color(rgb(theme::TEXT)).child(format!("{}{}", m.name, if m.primary { " (primary)" } else { "" })))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(if m.connected { "connected" } else { "disconnected" }.to_string())),
        )
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(format!("Current: {cur}  •  Scale: {:.2}", m.scale)))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(format!("{} mode(s) available", m.modes.len())))
}
