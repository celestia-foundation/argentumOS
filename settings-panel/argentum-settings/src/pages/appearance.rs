//! Appearance page.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::theme_scan;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // available-options vecs feed future dropdown widgets
pub struct AppearanceData {
    pub gtk_theme: String,
    pub icon_theme: String,
    pub interface_font: String,
    pub monospace_font: String,
    pub wallpaper_uri: String,
    pub themes_available: Vec<String>,
    pub icons_available: Vec<String>,
    pub wallpapers_available: Vec<std::path::PathBuf>,
}

pub struct AppearancePage {
    state: PageState<AppearanceData>,
    in_flight: HashSet<&'static str>,
}

impl AppearancePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, in_flight: Default::default() };
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
}

impl Render for AppearancePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Appearance.ttl()) {
            self.spawn_refresh(cx);
        }
        let body = match &self.state {
            PageState::Loaded { data, .. } => data_view(data, &self.in_flight),
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

fn data_view(data: &AppearanceData, in_flight: &HashSet<&'static str>) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(value_row("GTK theme", &data.gtk_theme, in_flight.contains("gtk_theme")))
        .child(value_row("Icon theme", &data.icon_theme, in_flight.contains("icon_theme")))
        .child(value_row("Interface font", &data.interface_font, in_flight.contains("interface_font")))
        .child(value_row("Monospace font", &data.monospace_font, in_flight.contains("monospace_font")))
        .child(value_row("Wallpaper", &data.wallpaper_uri, in_flight.contains("wallpaper")))
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
        .child(div().text_color(rgb(theme::TEXT)).child(value.to_string()));
    if in_flight {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row
}
