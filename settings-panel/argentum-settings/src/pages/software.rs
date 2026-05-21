//! Software page.

use crate::pages::{Page, PageState};
use crate::theme;
use crate::widgets::prompt;
use argentum_settings_core::dbus::flatpak as backend;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::collections::HashSet;

pub struct SoftwarePage {
    state: PageState<Vec<backend::Remote>>,
    add_pending: bool,
    add_error: Option<String>,
    in_flight: HashSet<String>,
}

impl SoftwarePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self {
            state: PageState::Empty,
            add_pending: false,
            add_error: None,
            in_flight: Default::default(),
        };
        s.spawn_refresh(cx);
        s
    }

    fn start_add_remote(&mut self, cx: &mut Context<Self>) {
        if self.add_pending {
            return;
        }
        self.add_pending = true;
        self.add_error = None;
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let pair = match prompt::name_and_url(
                "Add Flatpak remote",
                "Name (e.g. flathub-beta)",
                "URL (.flatpakrepo or repo root)",
            )
            .await
            {
                Ok(Some(p)) => p,
                Ok(None) => {
                    weak.update(async_cx, |this, cx| {
                        this.add_pending = false;
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.add_pending = false;
                        this.add_error = Some(e.to_string());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let (name, url) = pair;
            let result = backend::add_remote(&name, &url).await;
            weak.update(async_cx, |this, cx| {
                this.add_pending = false;
                match result {
                    Ok(()) => {
                        this.add_error = None;
                        this.spawn_refresh(cx);
                    }
                    Err(e) => {
                        this.add_error = Some(e.to_string());
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
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
            PageState::Loaded { data, .. } => remotes_view(
                data,
                &self.in_flight,
                self.add_pending,
                self.add_error.as_deref(),
                cx,
            )
            .into_any_element(),
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
    add_error: Option<&str>,
    cx: &mut Context<SoftwarePage>,
) -> gpui::Div {
    let mut col = div().flex().flex_col().gap_3();
    col = col.child(open_app_store_row(cx));
    col = col.child(div().text_color(rgb(theme::TEXT_MUTED)).child("Flatpak remotes"));
    for r in remotes {
        col = col.child(remote_row(r, in_flight.contains(&r.name), cx));
    }
    col = col.child(add_remote_button(add_pending, cx));
    if let Some(err) = add_error {
        col = col.child(
            div()
                .px_4()
                .py_2()
                .text_color(rgb(theme::TEXT_MUTED))
                .child(format!("Add remote failed: {err}")),
        );
    }
    col
}

fn open_app_store_row(cx: &mut Context<SoftwarePage>) -> gpui::Div {
    let inner = div()
        .id("open-app-store")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .size_full()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .cursor_pointer()
        .on_click(cx.listener(|_this, _event, _window, _cx| {
            if let Err(e) = std::process::Command::new("argentum-app-store").spawn() {
                tracing::warn!(?e, "failed to launch argentum-app-store");
            }
        }))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_color(rgb(theme::TEXT)).child("App Store"))
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child("Browse, install, and manage Flathub apps"),
                ),
        )
        .child(div().text_color(rgb(theme::ACCENT)).child("Open →"));
    div().mb_2().child(inner)
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

fn add_remote_button(pending: bool, cx: &mut Context<SoftwarePage>) -> gpui::Div {
    let button = div()
        .id("add-remote")
        .h(px(40.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(if pending { theme::SIDEBAR } else { theme::ACCENT }))
        .text_color(rgb(theme::BG))
        .rounded(px(8.))
        .cursor_pointer()
        .on_click(cx.listener(|this, _event, _window, cx| this.start_add_remote(cx)))
        .child(if pending { "Adding…" } else { "+ Add remote…" }.to_string());

    div()
        .flex()
        .flex_col()
        .gap_2()
        .mt_4()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(
            div()
                .text_color(rgb(theme::TEXT_MUTED))
                .child("Add a Flatpak remote (e.g. flathub-beta) — opens a name + URL prompt."),
        )
        .child(button)
}
