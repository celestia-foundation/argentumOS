//! System page.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::{
    os_release::{self, OsRelease},
    system::{self as sys, LogLine},
};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

#[derive(Default, Clone)]
pub struct SystemData {
    pub os: OsRelease,
    pub hostname: String,
}

pub struct SystemPage {
    state: PageState<SystemData>,
    log: Vec<LogLine>,
    upgrading: bool,
}

impl SystemPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, log: Vec::new(), upgrading: false };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let os = os_release::load().await.unwrap_or_default();
            let hostname = os_release::hostname().await.unwrap_or_default();
            weak.update(async_cx, |this, cx| {
                this.state = PageState::Loaded {
                    data: SystemData { os, hostname },
                    fetched_at: std::time::Instant::now(),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn start_upgrade(&mut self, cx: &mut Context<Self>) {
        if self.upgrading {
            return;
        }
        self.upgrading = true;
        self.log.clear();
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let mut rx = match sys::run_upgrade().await {
                Ok(rx) => rx,
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.log.push(LogLine::Stderr(e.to_string()));
                        this.upgrading = false;
                        cx.notify();
                    }).ok();
                    return;
                }
            };
            while let Some(line) = rx.recv().await {
                let is_exit = matches!(line, LogLine::Exit(_));
                weak.update(async_cx, |this, cx| {
                    this.log.push(line.clone());
                    if is_exit {
                        this.upgrading = false;
                    }
                    cx.notify();
                }).ok();
                if is_exit {
                    break;
                }
            }
        })
        .detach();
    }
}

impl Render for SystemPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::System.ttl()) {
            self.spawn_refresh(cx);
        }

        let upgrading = self.upgrading;
        let upgrade_button = div()
            .id("check-updates")
            .px_4()
            .py_2()
            .bg(rgb(if upgrading { theme::SIDEBAR } else { theme::ACCENT }))
            .text_color(rgb(theme::BG))
            .rounded(px(8.))
            .cursor_pointer()
            .on_click(cx.listener(|this, _event, _window, cx| {
                this.start_upgrade(cx);
            }))
            .child(if upgrading { "Updating…" } else { "Check for Updates" }.to_string());

        let body = match &self.state {
            PageState::Loaded { data, .. } => loaded_view(data, &self.log, upgrade_button),
            _ => skeleton(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("System"))
            .child(body)
    }
}

fn skeleton() -> gpui::Div {
    div().h(px(220.)).bg(rgb(theme::SURFACE)).rounded(px(10.))
}

fn loaded_view(d: &SystemData, log: &[LogLine], upgrade_button: impl IntoElement) -> gpui::Div {
    let pretty = d.os.pretty_name.clone().unwrap_or_else(|| "argentumOS".into());
    let version = d.os.version.clone().unwrap_or_default();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .px_4()
                .py_3()
                .bg(rgb(theme::SURFACE))
                .rounded(px(10.))
                .child(div().text_color(rgb(theme::TEXT)).child(pretty))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(version)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .bg(rgb(theme::SURFACE))
                .rounded(px(10.))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Hostname"))
                .child(div().text_color(rgb(theme::TEXT)).child(d.hostname.clone())),
        )
        .child(upgrade_button)
        .child(log_view(log))
}

fn log_view(log: &[LogLine]) -> gpui::Div {
    let mut pre = div()
        .flex()
        .flex_col()
        .min_h(px(120.))
        .px_3()
        .py_2()
        .bg(rgb(theme::BG))
        .rounded(px(8.))
        .text_color(rgb(theme::TEXT_MUTED));
    for line in log.iter().rev().take(200).rev() {
        let (text, color) = match line {
            LogLine::Stdout(s) => (s.clone(), theme::TEXT),
            LogLine::Stderr(s) => (s.clone(), theme::TEXT_MUTED),
            LogLine::Exit(c) => (format!("[exit {c}]"), theme::ACCENT),
        };
        pre = pre.child(div().text_color(rgb(color)).child(text));
    }
    pre
}
