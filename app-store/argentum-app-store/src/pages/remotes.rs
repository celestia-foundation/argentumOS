//! Remotes page — full CRUD on flatpak remotes (user scope).
//!
//! Mirrors and extends the settings-panel's Software page: enable/disable
//! toggle remains optimistic; remove is a confirmed (destructive) op via an
//! inline "Remove" button. Add-remote text fields share the settings-panel's
//! stub limitation (GPUI text input TODO).

use crate::pages::components::skeleton;
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::flatpak::remotes::{self as backend, Remote};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::collections::HashSet;

pub struct RemotesPage {
    state: PageState<Vec<Remote>>,
    in_flight: HashSet<String>,
    add_pending: bool,
}

impl RemotesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self {
            state: PageState::Empty,
            in_flight: Default::default(),
            add_pending: false,
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

    fn toggle_enabled(&mut self, name: String, cx: &mut Context<Self>) {
        let snapshot_enabled = match &mut self.state {
            PageState::Loaded { data, .. } => match data.iter_mut().find(|r| r.name == name) {
                Some(r) => {
                    let prev = r.enabled;
                    r.enabled = !r.enabled;
                    prev
                }
                None => return,
            },
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

    fn remove(&mut self, name: String, cx: &mut Context<Self>) {
        self.in_flight.insert(name.clone());
        cx.notify();
        let name_clone = name.clone();
        cx.spawn(async move |weak, async_cx| {
            let result = backend::remove_remote(&name_clone).await;
            weak.update(async_cx, |this, cx| {
                this.in_flight.remove(&name_clone);
                if result.is_ok() {
                    if let PageState::Loaded { data, .. } = &mut this.state {
                        data.retain(|r| r.name != name_clone);
                    }
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }
}

impl Render for RemotesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Remotes.ttl()) {
            self.spawn_refresh(cx);
        }
        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } => remotes_view(data, &self.in_flight, self.add_pending, cx).into_any_element(),
            PageState::Error(e) => skeleton::error_view(e).into_any_element(),
            _ => skeleton::rows(2, 56.).into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Remotes"))
            .child(body)
    }
}

fn remotes_view(
    remotes: &[Remote],
    in_flight: &HashSet<String>,
    add_pending: bool,
    cx: &mut Context<RemotesPage>,
) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_3();
    col = col.child(div().text_color(rgb(theme::TEXT_MUTED)).child("Flatpak remotes (user scope)"));
    for r in remotes {
        col = col.child(remote_row(r, in_flight.contains(&r.name), cx));
    }
    col = col.child(add_form(add_pending, cx));
    col
}

fn remote_row(r: &Remote, in_flight: bool, cx: &mut Context<RemotesPage>) -> AnyElement {
    let name_toggle = r.name.clone();
    let name_remove = r.name.clone();
    let mut row = div()
        .id(SharedString::from(format!("remote-row:{}", r.name)))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_color(rgb(theme::TEXT)).child(r.name.clone()))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(r.url.clone())),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap_3()
                .items_center()
                .child(
                    div()
                        .id(SharedString::from(format!("toggle-remote:{}", r.name)))
                        .text_color(if r.enabled { rgb(theme::ACCENT) } else { rgb(theme::TEXT_MUTED) })
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _e, _w, cx| {
                            this.toggle_enabled(name_toggle.clone(), cx);
                        }))
                        .child(if r.enabled { "enabled" } else { "disabled" }.to_string()),
                )
                .child(
                    div()
                        .id(SharedString::from(format!("remove-remote:{}", r.name)))
                        .px_3()
                        .py_1()
                        .bg(rgb(theme::SIDEBAR))
                        .text_color(rgb(theme::TEXT_MUTED))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(|s| s.text_color(rgb(theme::TEXT)))
                        .on_click(cx.listener(move |this, _e, _w, cx| {
                            this.remove(name_remove.clone(), cx);
                        }))
                        .child("Remove"),
                ),
        );
    if in_flight {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row.into_any_element()
}

fn add_form(pending: bool, cx: &mut Context<RemotesPage>) -> impl IntoElement {
    // TODO (shared limitation): real text input. Once GPUI text input lands,
    // wire two inputs + Add button → `backend::add_remote(name, url)`.
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
        .on_click(cx.listener(|_this, _e, _w, _cx| {
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

fn input_placeholder(label: &str) -> impl IntoElement {
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
