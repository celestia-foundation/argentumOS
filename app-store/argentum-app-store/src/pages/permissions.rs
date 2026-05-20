//! Permissions page — Flatseal-style per-app override editor.
//!
//! Two-column layout: left rail is the installed-app list; right pane is the
//! override form for the selected app. Toggles flip optimistically; writes
//! go through `permissions::write_override` and roll back on error.
//!
//! Scope: we surface the most common context toggles (sockets, devices,
//! shared, features) — the keyfile-backed string lists for filesystems,
//! persistent, env vars, and bus policies are read and re-written verbatim
//! but not editable inline until we have a real text input. Roundtrip is
//! lossless either way.

use crate::pages::components::skeleton;
use crate::pages::PageState;
use crate::theme;
use argentum_app_store_core::flatpak::installed::{self, InstalledApp};
use argentum_app_store_core::flatpak::permissions::{self as perm, OverrideSet};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::collections::HashSet;

/// Toggle groups we expose as one-click on/off rows. Each is keyed by the
/// (context-key, value) pair we add or remove from the underlying override
/// list when the user clicks.
const TOGGLES: &[(&str, &str, &str)] = &[
    ("Network", "shared", "network"),
    ("IPC", "shared", "ipc"),
    ("X11", "sockets", "x11"),
    ("Fallback X11", "sockets", "fallback-x11"),
    ("Wayland", "sockets", "wayland"),
    ("PulseAudio", "sockets", "pulseaudio"),
    ("Session bus", "sockets", "session-bus"),
    ("System bus", "sockets", "system-bus"),
    ("SSH agent", "sockets", "ssh-auth"),
    ("All devices", "devices", "all"),
    ("DRI (GPU)", "devices", "dri"),
    ("KVM", "devices", "kvm"),
    ("Devel (ptrace)", "features", "devel"),
    ("Bluetooth", "features", "bluetooth"),
];

pub struct PermissionsPage {
    apps_state: PageState<Vec<InstalledApp>>,
    selected: Option<String>,
    override_state: PageState<OverrideSet>,
    in_flight: HashSet<&'static str>,
}

impl PermissionsPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self {
            apps_state: PageState::Empty,
            selected: None,
            override_state: PageState::Empty,
            in_flight: HashSet::new(),
        };
        s.spawn_refresh_apps(cx);
        s
    }

    fn spawn_refresh_apps(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let r = installed::list_installed().await;
            weak.update(async_cx, |this, cx| {
                this.apps_state = match r {
                    Ok(apps) => PageState::Loaded { data: apps, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn select(&mut self, app_id: String, cx: &mut Context<Self>) {
        self.selected = Some(app_id.clone());
        self.override_state = PageState::Loading;
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let r = perm::read_override(&app_id).await;
            weak.update(async_cx, |this, cx| {
                this.override_state = match r {
                    Ok(s) => PageState::Loaded { data: s, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn toggle(&mut self, key: &'static str, value: &'static str, cx: &mut Context<Self>) {
        let Some(app_id) = self.selected.clone() else { return };
        let snapshot = match &self.override_state {
            PageState::Loaded { data, .. } => data.clone(),
            _ => return,
        };
        let mut next = snapshot.clone();
        flip_in(&mut next, key, value);
        self.override_state = PageState::Loaded { data: next.clone(), fetched_at: std::time::Instant::now() };
        self.in_flight.insert(toggle_tag(key, value));
        cx.notify();
        let tag = toggle_tag(key, value);
        cx.spawn(async move |weak, async_cx| {
            let started = std::time::Instant::now();
            let result = perm::write_override(&app_id, next.clone()).await;
            let elapsed = started.elapsed();
            if elapsed < std::time::Duration::from_millis(1000) {
                tokio::time::sleep(std::time::Duration::from_millis(1000) - elapsed).await;
            }
            weak.update(async_cx, |this, cx| {
                this.in_flight.remove(tag);
                if result.is_err() {
                    this.override_state = PageState::Loaded {
                        data: snapshot,
                        fetched_at: std::time::Instant::now(),
                    };
                }
                cx.notify();
            }).ok();
        })
        .detach();
    }
}

fn flip_in(set: &mut OverrideSet, key: &str, value: &str) {
    let target = match key {
        "shared" => &mut set.shared,
        "sockets" => &mut set.sockets,
        "devices" => &mut set.devices,
        "features" => &mut set.features,
        _ => return,
    };
    if let Some(idx) = target.iter().position(|v| v == value) {
        target.remove(idx);
    } else {
        target.push(value.to_string());
    }
}

fn contains(set: &OverrideSet, key: &str, value: &str) -> bool {
    let list = match key {
        "shared" => &set.shared,
        "sockets" => &set.sockets,
        "devices" => &set.devices,
        "features" => &set.features,
        _ => return false,
    };
    list.iter().any(|v| v == value)
}

fn toggle_tag(key: &'static str, value: &'static str) -> &'static str {
    // Cheap distinct tag — pairs are small + known at compile time, so we
    // just use the value (sufficiently unique within the TOGGLES list).
    let _ = key;
    value
}

impl Render for PermissionsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let left: AnyElement = match &self.apps_state {
            PageState::Loaded { data, .. } if data.is_empty() => {
                skeleton::empty_view("No apps installed.").into_any_element()
            }
            PageState::Loaded { data, .. } => app_list(data, &self.selected, cx).into_any_element(),
            PageState::Error(e) => skeleton::error_view(e).into_any_element(),
            _ => skeleton::rows(6, 48.).into_any_element(),
        };

        let right: AnyElement = if self.selected.is_none() {
            skeleton::empty_view("Select an app on the left.").into_any_element()
        } else {
            match &self.override_state {
                PageState::Loaded { data, .. } => {
                    form(data, &self.in_flight, cx).into_any_element()
                }
                PageState::Error(e) => skeleton::error_view(e).into_any_element(),
                _ => skeleton::rows(6, 36.).into_any_element(),
            }
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Permissions"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .size_full()
                    .child(div().w(px(280.)).flex_none().child(left))
                    .child(div().flex_1().min_w_0().child(right)),
            )
    }
}

fn app_list(
    apps: &[InstalledApp],
    selected: &Option<String>,
    cx: &mut Context<PermissionsPage>,
) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_1();
    for app in apps {
        let id = app.app_id.clone();
        let is_sel = selected.as_deref() == Some(&app.app_id);
        let mut row = div()
            .id(SharedString::from(format!("perm-app:{}", app.app_id)))
            .flex()
            .flex_col()
            .gap_0p5()
            .px_3()
            .py_2()
            .rounded(px(8.))
            .text_color(if is_sel { rgb(theme::TEXT) } else { rgb(theme::TEXT_MUTED) })
            .cursor_pointer()
            .hover(|s| s.bg(rgb(theme::SURFACE)).text_color(rgb(theme::TEXT)))
            .on_click(cx.listener(move |this, _e, _w, cx| this.select(id.clone(), cx)))
            .child(div().child(if app.name.is_empty() {
                app.app_id.clone()
            } else {
                app.name.clone()
            }))
            .child(div().text_color(rgb(theme::TEXT_MUTED)).child(app.app_id.clone()));
        if is_sel {
            row = row.bg(rgb(theme::SURFACE));
        }
        col = col.child(row);
    }
    col
}

fn form(
    set: &OverrideSet,
    in_flight: &HashSet<&'static str>,
    cx: &mut Context<PermissionsPage>,
) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_2();
    col = col.child(
        div()
            .text_color(rgb(theme::TEXT_MUTED))
            .child("Override flatpak sandbox permissions for this app."),
    );
    for (label, key, value) in TOGGLES {
        col = col.child(toggle_row(label, key, value, set, in_flight, cx));
    }
    // Read-only summary of any list-typed overrides we don't yet edit inline.
    if !set.filesystems.is_empty() {
        col = col.child(static_summary("Filesystems", &set.filesystems));
    }
    if !set.persistent.is_empty() {
        col = col.child(static_summary("Persistent", &set.persistent));
    }
    if !set.environment.is_empty() {
        let entries: Vec<String> = set
            .environment
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        col = col.child(static_summary("Environment", &entries));
    }
    col
}

fn toggle_row(
    label: &str,
    key: &'static str,
    value: &'static str,
    set: &OverrideSet,
    in_flight: &HashSet<&'static str>,
    cx: &mut Context<PermissionsPage>,
) -> AnyElement {
    let on = contains(set, key, value);
    let busy = in_flight.contains(toggle_tag(key, value));
    let mut row = div()
        .id(SharedString::from(format!("perm-toggle:{key}:{value}")))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(44.))
        .px_4()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _e, _w, cx| this.toggle(key, value, cx)))
        .child(div().text_color(rgb(theme::TEXT)).child(label.to_string()))
        .child(
            div()
                .text_color(if on { rgb(theme::ACCENT) } else { rgb(theme::TEXT_MUTED) })
                .child(if on { "on" } else { "off" }.to_string()),
        );
    if busy {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row.into_any_element()
}

fn static_summary(label: &str, items: &[String]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(div().text_color(rgb(theme::TEXT)).child(label.to_string()))
        .child(div().child(items.join("  ·  ")))
}
