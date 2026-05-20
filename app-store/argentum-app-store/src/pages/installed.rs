//! Installed apps — per-row uninstall with streaming progress.

use crate::pages::components::{progress_row, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::flatpak::install::{self, ProgressLine};
use argentum_app_store_core::flatpak::installed::{self as backend, InstalledApp};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct InstalledPage {
    state: PageState<Vec<InstalledApp>>,
    log: Vec<ProgressLine>,
    busy_app: Option<String>,
}

impl InstalledPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, log: Vec::new(), busy_app: None };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::list_installed().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(apps) => PageState::Loaded { data: apps, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn uninstall(&mut self, app_id: String, cx: &mut Context<Self>) {
        if self.busy_app.is_some() {
            return;
        }
        self.busy_app = Some(app_id.clone());
        self.log.clear();
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            // Reuse the streaming primitive so the user sees uninstall output
            // the same way as install. `flatpak uninstall` accepts the same
            // `--user --assumeyes` shape.
            let args = vec![
                "--user".to_string(),
                "uninstall".to_string(),
                "--assumeyes".to_string(),
                "--noninteractive".to_string(),
                app_id.clone(),
            ];
            let rx = spawn_via_install_helper(args).await;
            let mut rx = match rx {
                Ok(rx) => rx,
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.log.push(ProgressLine::Stderr(e.to_string()));
                        this.busy_app = None;
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
                        this.busy_app = None;
                        // Trigger a refresh so the removed app vanishes from the list.
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

impl Render for InstalledPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Installed.ttl()) {
            self.spawn_refresh(cx);
        }
        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } if data.is_empty() => {
                skeleton::empty_view("No flatpak apps installed yet. Head to Discover to install one.")
                    .into_any_element()
            }
            PageState::Loaded { data, .. } => rows_view(data, &self.busy_app, cx).into_any_element(),
            PageState::Error(e) => skeleton::error_view(e).into_any_element(),
            _ => skeleton::rows(4, 64.).into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Installed"))
            .child(body)
            .child(div().h(px(12.)))
            .child(progress_row::render(
                "Uninstall",
                &self.log,
                self.busy_app.is_some(),
            ))
    }
}

fn rows_view(
    apps: &[InstalledApp],
    busy: &Option<String>,
    cx: &mut Context<InstalledPage>,
) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_2();
    for app in apps {
        col = col.child(row(app, busy, cx));
    }
    col
}

fn row(
    app: &InstalledApp,
    busy: &Option<String>,
    cx: &mut Context<InstalledPage>,
) -> AnyElement {
    let app_id_for_listener = app.app_id.clone();
    let is_busy = busy.as_deref() == Some(&app.app_id);
    let any_busy = busy.is_some();

    div()
        .id(SharedString::from(format!("installed-row:{}", app.app_id)))
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
                .child(div().text_color(rgb(theme::TEXT)).child(if app.name.is_empty() {
                    app.app_id.clone()
                } else {
                    app.name.clone()
                }))
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child(format!("{} · {} · {}", app.app_id, app.version, app.size)),
                ),
        )
        .child(
            div()
                .id(SharedString::from(format!("uninstall-btn:{}", app.app_id)))
                .px_3()
                .py_1()
                .bg(rgb(if is_busy || any_busy { theme::SIDEBAR } else { theme::ACCENT }))
                .text_color(rgb(theme::BG))
                .rounded(px(6.))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _e, _w, cx| {
                    this.uninstall(app_id_for_listener.clone(), cx);
                }))
                .child(if is_busy { "Uninstalling…" } else { "Uninstall" }.to_string()),
        )
        .into_any_element()
}

/// Re-uses `install::install` machinery for uninstall by spawning the same
/// way. Centralised so the streaming-mpsc pattern lives in one place.
async fn spawn_via_install_helper(
    args: Vec<String>,
) -> argentum_app_store_core::Result<tokio::sync::mpsc::UnboundedReceiver<ProgressLine>> {
    // Mild abuse — we cross the public surface to reuse the spawn helper.
    // The dedicated helper is private to the install module, so re-implement
    // the small spawn here. Kept small intentionally; the heavy lifting still
    // lives in flatpak::install.
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::sync::mpsc;

    argentum_app_store_core::on_runtime(async move {
        let mut cmd = tokio::process::Command::new("flatpak");
        cmd.args(&args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn()?;
        let (tx, rx) = mpsc::unbounded_channel();
        if let Some(out) = child.stdout.take() {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(out).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx.send(ProgressLine::Stdout(line)).is_err() {
                        break;
                    }
                }
            });
        }
        if let Some(err) = child.stderr.take() {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx.send(ProgressLine::Stderr(line)).is_err() {
                        break;
                    }
                }
            });
        }
        tokio::spawn(async move {
            if let Ok(status) = child.wait().await {
                let _ = tx.send(ProgressLine::Exit(status.code().unwrap_or(-1)));
            }
        });
        Ok(rx)
    })
    .await
}

// `install` module is referenced for the `ProgressLine` re-export only.
#[allow(dead_code)]
fn _install_marker(_: install::ProgressLine) {}
