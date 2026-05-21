//! System page.

use crate::pages::{Page, PageState};
use crate::theme;
use crate::widgets::prompt;
use argentum_settings_core::{
    os_release::{self, OsRelease},
    system::{self as sys, LogLine},
};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
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
    hostname_pending: bool,
    hostname_error: Option<String>,
}

impl SystemPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self {
            state: PageState::Empty,
            log: Vec::new(),
            upgrading: false,
            hostname_pending: false,
            hostname_error: None,
        };
        s.spawn_refresh(cx);
        s
    }

    fn start_hostname_edit(&mut self, cx: &mut Context<Self>) {
        if self.hostname_pending {
            return;
        }
        let current = match &self.state {
            PageState::Loaded { data, .. } => data.hostname.clone(),
            _ => String::new(),
        };
        self.hostname_pending = true;
        self.hostname_error = None;
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let new_name = match prompt::text(
                "Change hostname",
                "New hostname",
                &current,
            )
            .await
            {
                Ok(Some(s)) if !s.is_empty() && s != current => s,
                Ok(_) => {
                    weak.update(async_cx, |this, cx| {
                        this.hostname_pending = false;
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.hostname_pending = false;
                        this.hostname_error = Some(e.to_string());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let result = os_release::set_hostname(&new_name).await;
            weak.update(async_cx, |this, cx| {
                this.hostname_pending = false;
                match result {
                    Ok(()) => {
                        if let PageState::Loaded { data, .. } = &mut this.state {
                            data.hostname = new_name;
                        }
                        this.hostname_error = None;
                        this.spawn_refresh(cx);
                    }
                    Err(e) => {
                        this.hostname_error = Some(e.to_string());
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

        let hostname_pending = self.hostname_pending;
        let hostname_error = self.hostname_error.clone();
        let body = match &self.state {
            PageState::Loaded { data, .. } => loaded_view(
                data,
                &self.log,
                upgrade_button,
                hostname_pending,
                hostname_error.as_deref(),
                cx,
            ),
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

fn loaded_view(
    d: &SystemData,
    log: &[LogLine],
    upgrade_button: impl IntoElement,
    hostname_pending: bool,
    hostname_error: Option<&str>,
    cx: &mut Context<SystemPage>,
) -> gpui::Div {
    let pretty = d.os.pretty_name.clone().unwrap_or_else(|| "argentumOS".into());
    let version = d.os.version.clone().unwrap_or_default();
    let hostname_value: SharedString = if hostname_pending {
        "Updating…".to_string().into()
    } else {
        d.hostname.clone().into()
    };
    let hostname_row = div()
        .id("hostname-row")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::SIDEBAR)))
        .on_click(cx.listener(|this, _e, _w, cx| this.start_hostname_edit(cx)))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Hostname"))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(div().text_color(rgb(theme::TEXT)).child(hostname_value))
                .child(div().text_color(rgb(theme::ACCENT)).child("Edit")),
        );
    let mut col = div()
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
        .child(hostname_row);
    if let Some(err) = hostname_error {
        col = col.child(
            div()
                .px_4()
                .py_2()
                .text_color(rgb(theme::TEXT_MUTED))
                .child(format!("Couldn't update hostname: {err}")),
        );
    }
    col.child(upgrade_button).child(log_view(log))
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
