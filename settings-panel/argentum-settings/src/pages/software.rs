//! Software page.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::dbus::flatpak as backend;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::collections::HashSet;

pub struct SoftwarePage {
    state: PageState<Vec<backend::Remote>>,
    add_pending: bool,
    in_flight: HashSet<String>,
}

impl SoftwarePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self {
            state: PageState::Empty,
            add_pending: false,
            in_flight: Default::default(),
        };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::list_remotes().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(r) => PageState::Loaded { data: r, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn toggle_enabled(&mut self, cx: &mut Context<Self>, name: String) {
        let snapshot_enabled = match &mut self.state {
            PageState::Loaded { data, .. } => {
                let row = match data.iter_mut().find(|r| r.name == name) {
                    Some(r) => r,
                    None => return,
                };
                let prev = row.enabled;
                row.enabled = !row.enabled;
                prev
            }
            _ => return,
        };
        self.in_flight.insert(name.clone());
        cx.notify();
        let name_clone = name.clone();
        let new_state = !snapshot_enabled;
        cx.spawn(async move |weak, async_cx| {
            let started = std::time::Instant::now();
            let result = backend::set_enabled(&name_clone, new_state).await;
            let elapsed = started.elapsed();
            if elapsed < std::time::Duration::from_millis(1000) {
                // Minimum-visible duration so the in-flight underline registers.
                tokio::time::sleep(std::time::Duration::from_millis(1000) - elapsed).await;
            }
            weak.update(async_cx, |this, cx| {
                this.in_flight.remove(&name_clone);
                if result.is_err() {
                    if let PageState::Loaded { data, .. } = &mut this.state {
                        if let Some(r) = data.iter_mut().find(|r| r.name == name_clone) {
                            r.enabled = snapshot_enabled;
                        }
                    }
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }
}

impl Render for SoftwarePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Software.ttl()) {
            self.spawn_refresh(cx);
        }
        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } => {
                remotes_view(data, &self.in_flight, self.add_pending, cx).into_any_element()
            }
            PageState::Error(e) => error_view(e).into_any_element(),
            _ => skeleton().into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Software"))
            .child(body)
    }
}

fn skeleton() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().h(px(56.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
        .child(div().h(px(56.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
}

fn error_view(msg: &str) -> gpui::Div {
    div()
        .h(px(100.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(format!("flatpak: {msg}"))
}

fn remotes_view(
    remotes: &[backend::Remote],
    in_flight: &HashSet<String>,
    add_pending: bool,
    cx: &mut Context<SoftwarePage>,
) -> gpui::Div {
    let mut col = div().flex().flex_col().gap_3();
    col = col.child(div().text_color(rgb(theme::TEXT_MUTED)).child("Flatpak remotes"));
    for r in remotes {
        col = col.child(remote_row(r, in_flight.contains(&r.name), cx));
    }
    col = col.child(add_remote_form(add_pending, cx));
    col
}

fn remote_row(
    r: &backend::Remote,
    in_flight: bool,
    cx: &mut Context<SoftwarePage>,
) -> AnyElement {
    let name = r.name.clone();
    let mut row = div()
        .id(SharedString::from(format!("remote-row:{}", name)))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _event, _window, cx| {
            this.toggle_enabled(cx, name.clone());
        }))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_color(rgb(theme::TEXT)).child(r.name.clone()))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(r.url.clone())),
        )
        .child(
            div()
                .text_color(if r.enabled { rgb(theme::ACCENT) } else { rgb(theme::TEXT_MUTED) })
                .child(if r.enabled { "enabled" } else { "disabled" }.to_string()),
        );
    if in_flight {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row.into_any_element()
}

fn add_remote_form(pending: bool, cx: &mut Context<SoftwarePage>) -> gpui::Div {
    // TODO: real text input. For now Add is a no-op (logs intent) so the
    // button is at least responsive — actual flatpak remote-add happens once
    // the input widget exists.
    let add_button = div()
        .id("add-remote")
        .h(px(36.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(if pending { theme::SIDEBAR } else { theme::ACCENT }))
        .text_color(rgb(theme::BG))
        .rounded(px(6.))
        .cursor_pointer()
        .on_click(cx.listener(|_this, _event, _window, _cx| {
            tracing::info!("add-remote click — text input TODO");
        }))
        .child(if pending { "Adding…" } else { "Add" }.to_string());

    div()
        .flex()
        .flex_col()
        .gap_2()
        .mt_4()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Add remote"))
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(input_placeholder("Name (e.g. flathub-beta)"))
                .child(input_placeholder("URL"))
                .child(add_button),
        )
}

fn input_placeholder(label: &str) -> gpui::Div {
    div()
        .h(px(36.))
        .flex_1()
        .px_3()
        .flex()
        .items_center()
        .bg(rgb(theme::BG))
        .rounded(px(6.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(label.to_string())
}
