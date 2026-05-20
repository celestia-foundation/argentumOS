//! Runtimes / SDKs admin — list + "Prune unused".

use crate::pages::components::{progress_row, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::flatpak::install::{self, ProgressLine};
use argentum_app_store_core::flatpak::runtimes::{self as backend, Runtime};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct RuntimesPage {
    state: PageState<Vec<Runtime>>,
    log: Vec<ProgressLine>,
    pruning: bool,
}

impl RuntimesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, log: Vec::new(), pruning: false };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::list_runtimes().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(rs) => PageState::Loaded { data: rs, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn prune(&mut self, cx: &mut Context<Self>) {
        if self.pruning {
            return;
        }
        self.pruning = true;
        self.log.clear();
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let mut rx = match install::prune_unused().await {
                Ok(rx) => rx,
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.log.push(ProgressLine::Stderr(e.to_string()));
                        this.pruning = false;
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
                        this.pruning = false;
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

impl Render for RuntimesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Runtimes.ttl()) {
            self.spawn_refresh(cx);
        }
        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } if data.is_empty() => {
                skeleton::empty_view("No runtimes installed yet.").into_any_element()
            }
            PageState::Loaded { data, .. } => rows_view(data).into_any_element(),
            PageState::Error(e) => skeleton::error_view(e).into_any_element(),
            _ => skeleton::rows(3, 56.).into_any_element(),
        };
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
                    .child(div().text_xl().child("Runtimes"))
                    .child(prune_button(self.pruning, cx)),
            )
            .child(body)
            .child(div().h(px(12.)))
            .child(progress_row::render("Prune unused", &self.log, self.pruning))
    }
}

fn prune_button(busy: bool, cx: &mut Context<RuntimesPage>) -> impl IntoElement {
    div()
        .id("prune-unused")
        .px_3()
        .py_1()
        .bg(rgb(if busy { theme::SIDEBAR } else { theme::ACCENT }))
        .text_color(rgb(theme::BG))
        .rounded(px(6.))
        .cursor_pointer()
        .on_click(cx.listener(|this, _e, _w, cx| this.prune(cx)))
        .child(if busy { "Pruning…" } else { "Prune unused" }.to_string())
}

fn rows_view(runtimes: &[Runtime]) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_2();
    for r in runtimes {
        col = col.child(
            div()
                .id(SharedString::from(format!("runtime-row:{}", r.id)))
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
                        .child(div().text_color(rgb(theme::TEXT)).child(if r.name.is_empty() {
                            r.id.clone()
                        } else {
                            r.name.clone()
                        }))
                        .child(
                            div()
                                .text_color(rgb(theme::TEXT_MUTED))
                                .child(format!("{} · {} · {}", r.id, r.version, r.branch)),
                        ),
                )
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(r.size.clone())),
        );
    }
    col
}
