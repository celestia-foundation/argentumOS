//! Appearance page. Click a value row to expand a list of options below it —
//! selecting an item calls gsettings and updates immediately. The single-open
//! pattern keeps the layout tight; clicking the same row again collapses.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::theme_scan;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct AppearanceData {
    pub gtk_theme: String,
    pub icon_theme: String,
    pub interface_font: String,
    pub monospace_font: String,
    pub wallpaper_uri: String,
    pub themes_available: Vec<String>,
    pub icons_available: Vec<String>,
    pub wallpapers_available: Vec<PathBuf>,
}

/// Which section's option list is expanded. At most one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    GtkTheme,
    IconTheme,
    Wallpaper,
}

pub struct AppearancePage {
    state: PageState<AppearanceData>,
    in_flight: HashSet<&'static str>,
    open: Option<Section>,
}

impl AppearancePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, in_flight: Default::default(), open: None };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let themes = theme_scan::list_gtk_themes().await.unwrap_or_default();
            let icons = theme_scan::list_icon_themes().await.unwrap_or_default();
            let wallpapers = argentum_settings_core::wallpaper_scan::list_wallpapers()
                .await
                .unwrap_or_default();
            let gtk_theme = theme_scan::gsettings_get("org.cinnamon.desktop.interface", "gtk-theme").await.unwrap_or_default();
            let icon_theme = theme_scan::gsettings_get("org.cinnamon.desktop.interface", "icon-theme").await.unwrap_or_default();
            let interface_font = theme_scan::gsettings_get("org.cinnamon.desktop.interface", "font-name").await.unwrap_or_default();
            let monospace_font = theme_scan::gsettings_get("org.gnome.desktop.interface", "monospace-font-name").await.unwrap_or_default();
            let wallpaper_uri = theme_scan::gsettings_get("org.cinnamon.desktop.background", "picture-uri").await.unwrap_or_default();

            let data = AppearanceData {
                gtk_theme, icon_theme, interface_font, monospace_font, wallpaper_uri,
                themes_available: themes, icons_available: icons, wallpapers_available: wallpapers,
            };
            weak.update(async_cx, |this, cx| {
                this.state = PageState::Loaded { data, fetched_at: std::time::Instant::now() };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn toggle_section(&mut self, section: Section, cx: &mut Context<Self>) {
        self.open = if self.open == Some(section) { None } else { Some(section) };
        cx.notify();
    }

    fn pick_gtk_theme(&mut self, name: String, cx: &mut Context<Self>) {
        self.set_gsettings(
            "gtk_theme",
            "org.cinnamon.desktop.interface",
            "gtk-theme",
            name,
            |d, v| d.gtk_theme = v,
            cx,
        );
    }

    fn pick_icon_theme(&mut self, name: String, cx: &mut Context<Self>) {
        self.set_gsettings(
            "icon_theme",
            "org.cinnamon.desktop.interface",
            "icon-theme",
            name,
            |d, v| d.icon_theme = v,
            cx,
        );
    }

    fn pick_wallpaper(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let uri = format!("file://{}", path.display());
        self.set_gsettings(
            "wallpaper",
            "org.cinnamon.desktop.background",
            "picture-uri",
            uri,
            |d, v| d.wallpaper_uri = v,
            cx,
        );
    }

    fn set_gsettings(
        &mut self,
        key: &'static str,
        schema: &'static str,
        gkey: &'static str,
        value: String,
        apply_local: impl FnOnce(&mut AppearanceData, String) + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        // Optimistic update.
        if let PageState::Loaded { data, .. } = &mut self.state {
            apply_local(data, value.clone());
        }
        self.in_flight.insert(key);
        self.open = None;
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let _ = theme_scan::gsettings_set(schema, gkey, &value).await;
            weak.update(async_cx, |this, cx| {
                this.in_flight.remove(key);
                cx.notify();
                this.spawn_refresh(cx);
            })
            .ok();
        })
        .detach();
    }
}

impl Render for AppearancePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Appearance.ttl()) {
            self.spawn_refresh(cx);
        }
        let body = match &self.state {
            PageState::Loaded { data, .. } => data_view(data, &self.in_flight, self.open, cx),
            _ => skeleton(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Appearance"))
            .child(body)
    }
}

fn skeleton() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(skeleton_row("Wallpaper"))
        .child(skeleton_row("GTK theme"))
        .child(skeleton_row("Icon theme"))
        .child(skeleton_row("Interface font"))
        .child(skeleton_row("Monospace font"))
}

fn skeleton_row(label: &'static str) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(56.))
        .px_4()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(label))
        .child(div().w(px(180.)).h(px(28.)).bg(rgb(theme::SIDEBAR)).rounded(px(6.)))
}

fn data_view(
    data: &AppearanceData,
    in_flight: &HashSet<&'static str>,
    open: Option<Section>,
    cx: &mut Context<AppearancePage>,
) -> gpui::Div {
    let mut col = div().flex().flex_col().gap_3();
    col = col.child(picker_row(
        Section::GtkTheme,
        "GTK theme",
        &data.gtk_theme,
        in_flight.contains("gtk_theme"),
        open,
        cx,
    ));
    if open == Some(Section::GtkTheme) {
        col = col.child(theme_list(&data.themes_available, &data.gtk_theme, Section::GtkTheme, cx));
    }
    col = col.child(picker_row(
        Section::IconTheme,
        "Icon theme",
        &data.icon_theme,
        in_flight.contains("icon_theme"),
        open,
        cx,
    ));
    if open == Some(Section::IconTheme) {
        col = col.child(theme_list(&data.icons_available, &data.icon_theme, Section::IconTheme, cx));
    }
    col = col.child(picker_row(
        Section::Wallpaper,
        "Wallpaper",
        &data.wallpaper_uri,
        in_flight.contains("wallpaper"),
        open,
        cx,
    ));
    if open == Some(Section::Wallpaper) {
        col = col.child(wallpaper_list(&data.wallpapers_available, &data.wallpaper_uri, cx));
    }
    // Font rows stay read-only for now — font picker UX (preview, size) is a
    // separate piece of work and the existing gsettings values are still
    // visible here.
    col = col.child(value_row("Interface font", &data.interface_font, false));
    col = col.child(value_row("Monospace font", &data.monospace_font, false));
    col
}

fn picker_row(
    section: Section,
    label: &str,
    value: &str,
    in_flight: bool,
    open: Option<Section>,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let chevron = if open == Some(section) { "▾" } else { "▸" };
    let mut row = div()
        .id(SharedString::from(format!("picker:{label}")))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(56.))
        .px_4()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::SIDEBAR)))
        .on_click(cx.listener(move |this, _e, _w, cx| this.toggle_section(section, cx)))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(label.to_string()))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(div().text_color(rgb(theme::TEXT)).child(short_label(value)))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child(chevron.to_string())),
        );
    if in_flight {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row.into_any_element()
}

fn theme_list(
    available: &[String],
    current: &str,
    section: Section,
    cx: &mut Context<AppearancePage>,
) -> gpui::Div {
    let mut col = div()
        .flex()
        .flex_col()
        .gap_1()
        .px_2()
        .py_2()
        .bg(rgb(theme::BG))
        .rounded(px(10.));
    if available.is_empty() {
        col = col.child(
            div()
                .px_4()
                .py_2()
                .text_color(rgb(theme::TEXT_MUTED))
                .child("No themes found on this system."),
        );
        return col;
    }
    for name in available {
        let is_current = name == current;
        let name_owned = name.clone();
        let row = div()
            .id(SharedString::from(format!("theme-opt:{name}")))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .rounded(px(8.))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(theme::SURFACE)))
            .on_click(cx.listener(move |this, _e, _w, cx| match section {
                Section::GtkTheme => this.pick_gtk_theme(name_owned.clone(), cx),
                Section::IconTheme => this.pick_icon_theme(name_owned.clone(), cx),
                Section::Wallpaper => {}
            }))
            .child(div().text_color(rgb(theme::TEXT)).child(name.clone()))
            .child(
                div()
                    .text_color(rgb(if is_current { theme::ACCENT } else { theme::TEXT_MUTED }))
                    .child(if is_current { "Current" } else { "" }.to_string()),
            );
        col = col.child(row);
    }
    col
}

fn wallpaper_list(
    available: &[PathBuf],
    current_uri: &str,
    cx: &mut Context<AppearancePage>,
) -> gpui::Div {
    let mut col = div()
        .flex()
        .flex_col()
        .gap_1()
        .px_2()
        .py_2()
        .bg(rgb(theme::BG))
        .rounded(px(10.));
    if available.is_empty() {
        col = col.child(
            div()
                .px_4()
                .py_2()
                .text_color(rgb(theme::TEXT_MUTED))
                .child("No wallpapers in /etc/backgrounds or ~/Pictures."),
        );
        return col;
    }
    for path in available {
        let display = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let uri = format!("file://{}", path.display());
        let is_current = uri == current_uri;
        let path_owned = path.clone();
        let row = div()
            .id(SharedString::from(format!("wp-opt:{}", path.display())))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .rounded(px(8.))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(theme::SURFACE)))
            .on_click(cx.listener(move |this, _e, _w, cx| {
                this.pick_wallpaper(path_owned.clone(), cx);
            }))
            .child(div().text_color(rgb(theme::TEXT)).child(display))
            .child(
                div()
                    .text_color(rgb(if is_current { theme::ACCENT } else { theme::TEXT_MUTED }))
                    .child(if is_current { "Current" } else { "" }.to_string()),
            );
        col = col.child(row);
    }
    col
}

fn value_row(label: &str, value: &str, in_flight: bool) -> gpui::Div {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(56.))
        .px_4()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(label.to_string()))
        .child(div().text_color(rgb(theme::TEXT)).child(short_label(value)));
    if in_flight {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row
}

/// Trim a wallpaper URI / theme name for display in the value chip — long
/// `file://...` URIs make the row wrap unattractively.
fn short_label(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("file://") {
        if let Some(name) = std::path::Path::new(rest).file_name().and_then(|n| n.to_str()) {
            return name.to_string();
        }
    }
    s.to_string()
}
