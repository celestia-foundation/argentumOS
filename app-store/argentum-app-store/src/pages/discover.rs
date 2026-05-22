//! Discover page — Flathub "popular" + "recently-added" with drill-in to a
//! detail view per app. Detail merges local AppStream + Flathub API.

use crate::pages::components::{app_card, progress_row, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::flathub_api::{self as api, AppDetail, AppSummary};
use argentum_app_store_core::flatpak::install::{self, ProgressLine};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct DiscoverData {
    pub popular: Vec<AppSummary>,
    pub recent: Vec<AppSummary>,
    /// First error encountered while fetching either collection. We don't fail
    /// the whole page on one collection error — empty grids render the
    /// existing empty-state — but we surface the reason as a small banner so
    /// the user knows whether to wait, retry, or check connectivity.
    pub fetch_error: Option<String>,
}

pub enum View {
    List,
    Detail {
        app_id: String,
        detail: PageState<AppDetail>,
        install_log: Vec<ProgressLine>,
        installing: bool,
    },
}

pub struct DiscoverPage {
    state: PageState<DiscoverData>,
    view: View,
}

impl DiscoverPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, view: View::List };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let pop_res = api::popular(30).await;
            let rec_res = api::recently_added(30).await;
            let fetch_error = match (&pop_res, &rec_res) {
                (Err(e), _) | (_, Err(e)) => Some(e.to_string()),
                _ => None,
            };
            let popular = pop_res.unwrap_or_default();
            let recent = rec_res.unwrap_or_default();
            weak.update(async_cx, |this, cx| {
                this.state = PageState::Loaded {
                    data: DiscoverData { popular, recent, fetch_error },
                    fetched_at: std::time::Instant::now(),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    pub fn open_detail(&mut self, app_id: String, cx: &mut Context<Self>) {
        self.view = View::Detail {
            app_id: app_id.clone(),
            detail: PageState::Loading,
            install_log: Vec::new(),
            installing: false,
        };
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let result = api::app_detail(&app_id).await;
            weak.update(async_cx, |this, cx| {
                if let View::Detail { detail, .. } = &mut this.view {
                    *detail = match result {
                        Ok(d) => PageState::Loaded { data: d, fetched_at: std::time::Instant::now() },
                        Err(e) => PageState::Error(e.to_string()),
                    };
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }

    pub fn back_to_list(&mut self, cx: &mut Context<Self>) {
        self.view = View::List;
        cx.notify();
    }

    pub fn start_install(&mut self, cx: &mut Context<Self>) {
        let app_id = match &self.view {
            View::Detail { app_id, installing, .. } if !*installing => app_id.clone(),
            _ => return,
        };
        if let View::Detail { installing, install_log, .. } = &mut self.view {
            *installing = true;
            install_log.clear();
        }
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let mut rx = match install::install("flathub", &app_id).await {
                Ok(rx) => rx,
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        if let View::Detail { install_log, installing, .. } = &mut this.view {
                            install_log.push(ProgressLine::Stderr(e.to_string()));
                            *installing = false;
                        }
                        cx.notify();
                    }).ok();
                    return;
                }
            };
            while let Some(line) = rx.recv().await {
                let exit = matches!(line, ProgressLine::Exit(_));
                weak.update(async_cx, |this, cx| {
                    if let View::Detail { install_log, installing, .. } = &mut this.view {
                        install_log.push(line.clone());
                        if exit { *installing = false; }
                    }
                    cx.notify();
                }).ok();
                if exit { break; }
            }
        })
        .detach();
    }

    pub fn launch_installed(&self, app_id: &str) {
        // Fire-and-forget: spawn `flatpak run` outside our address space so
        // the app store remains usable. stderr/stdout are detached.
        if let Err(e) = std::process::Command::new("flatpak")
            .args(["run", app_id])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            tracing::warn!(?e, app_id, "failed to flatpak run");
        }
    }
}

impl Render for DiscoverPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if matches!(self.view, View::List) && self.state.should_refresh(Page::Discover.ttl()) {
            self.spawn_refresh(cx);
        }

        let body: AnyElement = match &self.view {
            View::List => match &self.state {
                PageState::Loaded { data, .. } => list_view(data, cx).into_any_element(),
                PageState::Error(e) => skeleton::error_view(e).into_any_element(),
                _ => list_skeleton().into_any_element(),
            },
            View::Detail { app_id, detail, install_log, installing } => {
                detail_view(app_id, detail, install_log, *installing, cx).into_any_element()
            }
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(header(&self.view, cx))
            .child(body)
    }
}

fn header(view: &View, cx: &mut Context<DiscoverPage>) -> AnyElement {
    match view {
        View::List => div().text_xl().pb_4().child("Discover").into_any_element(),
        View::Detail { app_id, .. } => div()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .pb_4()
            .child(
                div()
                    .id("back-to-list")
                    .px_3()
                    .py_1()
                    .bg(rgb(theme::SURFACE))
                    .rounded(px(6.))
                    .text_color(rgb(theme::TEXT_MUTED))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _e, _w, cx| this.back_to_list(cx)))
                    .child("← Discover"),
            )
            .child(div().text_xl().text_color(rgb(theme::TEXT)).child(app_id.clone()))
            .into_any_element(),
    }
}

fn list_skeleton() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Popular"))
        .child(skeleton::rows(2, 100.))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Recently added"))
        .child(skeleton::rows(2, 100.))
}

fn list_view(data: &DiscoverData, cx: &mut Context<DiscoverPage>) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_4();
    if let Some(err) = data.fetch_error.as_deref() {
        // Distinguish "network down" from "Flathub returned 500" from
        // "DNS resolution failed" — the message comes straight from reqwest
        // and is the most actionable thing we can show the user.
        col = col.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_3()
                .px_4()
                .py_3()
                .bg(rgb(theme::SURFACE))
                .rounded(px(10.))
                .text_color(rgb(theme::TEXT_MUTED))
                .child(div().child(format!("Flathub fetch error: {err}")))
                .child(
                    div()
                        .id("retry-fetch")
                        .px_3()
                        .py_1()
                        .bg(rgb(theme::ACCENT))
                        .text_color(rgb(theme::BG))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _e, _w, cx| this.spawn_refresh(cx)))
                        .child("Retry"),
                ),
        );
    }
    col.child(div().text_color(rgb(theme::TEXT_MUTED)).child("Popular"))
        .child(grid(&data.popular, cx))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Recently added"))
        .child(grid(&data.recent, cx))
}

pub(crate) fn grid(items: &[AppSummary], cx: &mut Context<DiscoverPage>) -> impl IntoElement {
    if items.is_empty() {
        return div()
            .child(skeleton::empty_view(
                "Couldn't reach Flathub. Showing nothing for now — try again when the network is up.",
            ))
            .into_any_element();
    }
    let mut grid = div()
        .id(SharedString::from("discover-grid"))
        .flex()
        .flex_row()
        .flex_wrap()
        .gap_3();
    for item in items.iter().take(30) {
        let data = app_card::AppCardData {
            app_id: item.app_id.clone(),
            name: item.name.clone(),
            summary: item.summary.clone(),
            icon_url: item.icon.clone(),
        };
        let card = app_card::render::<DiscoverPage>(data, cx, |this, id, cx| this.open_detail(id, cx));
        grid = grid.child(div().w(px(220.)).child(card));
    }
    grid.into_any_element()
}

fn detail_view(
    app_id: &str,
    detail: &PageState<AppDetail>,
    install_log: &[ProgressLine],
    installing: bool,
    cx: &mut Context<DiscoverPage>,
) -> impl IntoElement {
    let body: AnyElement = match detail {
        PageState::Loaded { data, .. } => loaded_detail(data, install_log, installing, cx).into_any_element(),
        PageState::Error(e) => skeleton::error_view(e).into_any_element(),
        _ => detail_skeleton(app_id).into_any_element(),
    };
    div().flex().flex_col().gap_3().child(body)
}

fn detail_skeleton(app_id: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(format!("Loading {app_id}…")))
        .child(skeleton::bar(180.))
        .child(skeleton::rows(3, 24.))
}

fn loaded_detail(
    d: &AppDetail,
    install_log: &[ProgressLine],
    installing: bool,
    cx: &mut Context<DiscoverPage>,
) -> impl IntoElement {
    let installs = d
        .installs_total
        .map(|n| format!("{n} installs"))
        .unwrap_or_default();
    let verified = if d.verified { " · verified publisher" } else { "" };

    // After an install finishes successfully, the last log line is
    // `Exit(0)`. Use that as the cue to surface the "Open" button so the
    // user can launch what they just installed without leaving the page.
    let install_succeeded = matches!(install_log.last(), Some(ProgressLine::Exit(0)));

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
                .child(div().text_color(rgb(theme::TEXT)).child(d.name.clone()))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(d.summary.clone()))
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child(format!("{}{verified}", installs)),
                ),
        )
        .child(install_button(installing, cx))
        .child(if install_succeeded {
            open_button(&d.flatpak_app_id, cx).into_any_element()
        } else {
            div().into_any_element()
        })
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .px_4()
                .py_3()
                .bg(rgb(theme::SURFACE))
                .rounded(px(10.))
                .text_color(rgb(theme::TEXT_MUTED))
                .child(truncate_paragraph(&d.description, 1200)),
        )
        .child(progress_row::render("Install", install_log, installing))
}

fn open_button(app_id: &str, cx: &mut Context<DiscoverPage>) -> impl IntoElement {
    let id = app_id.to_string();
    div()
        .id("open-installed")
        .h(px(36.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .text_color(rgb(theme::ACCENT))
        .rounded(px(6.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::SIDEBAR)))
        .on_click(cx.listener(move |this, _e, _w, _cx| this.launch_installed(&id)))
        .child("Open ↗")
}

fn install_button(installing: bool, cx: &mut Context<DiscoverPage>) -> impl IntoElement {
    div()
        .id("install-btn")
        .h(px(36.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(if installing { theme::SIDEBAR } else { theme::ACCENT }))
        .text_color(rgb(theme::BG))
        .rounded(px(6.))
        .cursor_pointer()
        .on_click(cx.listener(|this, _e, _w, cx| this.start_install(cx)))
        .child(if installing { "Installing…" } else { "Install" }.to_string())
}

fn truncate_paragraph(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.min(s.len())])
    }
}
