//! Updates page — list of apps with pending updates + "Update all" button.

use crate::pages::components::{progress_row, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::flatpak::install::{self, ProgressLine};
use argentum_app_store_core::flatpak::installed as backend;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct UpdatesPage {
    state: PageState<Vec<String>>,
    log: Vec<ProgressLine>,
    busy: bool,
}

impl UpdatesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, log: Vec::new(), busy: false };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::list_pending_updates().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(ids) => PageState::Loaded { data: ids, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn update_all(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.log.clear();
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let mut rx = match install::update_all().await {
                Ok(rx) => rx,
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.log.push(ProgressLine::Stderr(e.to_string()));
                        this.busy = false;
                        cx.notify();
                    }).ok();
                    return;
                }
            };
            while let Some(line) = rx.recv().await {
                let exit = matches!(line, ProgressLine::Exit(_));
                weak.update(async_cx, |this, cx| {
                    this.log.push(line.clone());
                    if exit {
                        this.busy = false;
                        this.spawn_refresh(cx);
                    }
                    cx.notify();
                }).ok();
                if exit { break; }
            }
        })
        .detach();
    }

    fn update_one(&mut self, app_id: String, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.log.clear();
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let mut rx = match install::update(&app_id).await {
                Ok(rx) => rx,
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.log.push(ProgressLine::Stderr(e.to_string()));
                        this.busy = false;
                        cx.notify();
                    }).ok();
                    return;
                }
            };
            while let Some(line) = rx.recv().await {
                let exit = matches!(line, ProgressLine::Exit(_));
                weak.update(async_cx, |this, cx| {
                    this.log.push(line.clone());
                    if exit {
                        this.busy = false;
                        this.spawn_refresh(cx);
                    }
                    cx.notify();
                }).ok();
                if exit { break; }
            }
        })
        .detach();
    }
}

impl Render for UpdatesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Updates.ttl()) {
            self.spawn_refresh(cx);
        }
        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } if data.is_empty() => {
                skeleton::empty_view("Everything is up to date.").into_any_element()
            }
            PageState::Loaded { data, .. } => rows_view(data, self.busy, cx).into_any_element(),
            PageState::Error(e) => skeleton::error_view(e).into_any_element(),
            _ => skeleton::rows(3, 64.).into_any_element(),
        };

        let any_pending = matches!(&self.state, PageState::Loaded { data, .. } if !data.is_empty());

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .pb_4()
                    .child(div().text_xl().child("Updates"))
                    .child(update_all_button(self.busy, any_pending, cx)),
            )
            .child(body)
            .child(div().h(px(12.)))
            .child(progress_row::render("Update", &self.log, self.busy))
    }
}

fn update_all_button(
    busy: bool,
    any_pending: bool,
    cx: &mut Context<UpdatesPage>,
) -> impl IntoElement {
    let disabled = busy || !any_pending;
    div()
        .id("update-all")
        .px_3()
        .py_1()
        .bg(rgb(if disabled { theme::SIDEBAR } else { theme::ACCENT }))
        .text_color(rgb(theme::BG))
        .rounded(px(6.))
        .cursor_pointer()
        .on_click(cx.listener(|this, _e, _w, cx| this.update_all(cx)))
        .child(if busy { "Updating…" } else { "Update all" }.to_string())
}

fn rows_view(ids: &[String], busy: bool, cx: &mut Context<UpdatesPage>) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_2();
    for id in ids {
        let id_clone = id.clone();
        let id_clone2 = id.clone();
        col = col.child(
            div()
                .id(SharedString::from(format!("update-row:{}", id)))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .bg(rgb(theme::SURFACE))
                .rounded(px(10.))
                .child(div().text_color(rgb(theme::TEXT)).child(id_clone))
                .child(
                    div()
                        .id(SharedString::from(format!("update-btn:{}", id)))
                        .px_3()
                        .py_1()
                        .bg(rgb(if busy { theme::SIDEBAR } else { theme::ACCENT }))
                        .text_color(rgb(theme::BG))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _e, _w, cx| {
                            this.update_one(id_clone2.clone(), cx);
                        }))
                        .child("Update".to_string()),
                ),
        );
    }
    col
}
