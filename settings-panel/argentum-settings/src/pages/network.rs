//! Network page.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::dbus::network as backend;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};

#[derive(Default, Clone)]
pub struct NetworkData {
    pub wifi: Vec<backend::WifiNetwork>,
    pub vpn: Vec<backend::VpnConnection>,
}

pub struct NetworkPage {
    state: PageState<NetworkData>,
}

impl NetworkPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let wifi = backend::list_wifi().await.unwrap_or_default();
            let vpn = backend::list_vpn().await.unwrap_or_default();
            weak.update(async_cx, |this, cx| {
                this.state = PageState::Loaded {
                    data: NetworkData { wifi, vpn },
                    fetched_at: std::time::Instant::now(),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }
}

impl Render for NetworkPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Network.ttl()) {
            self.spawn_refresh(cx);
        }
        let body = match &self.state {
            PageState::Loaded { data, .. } => data_view(data),
            _ => skeleton(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Network"))
            .child(body)
    }
}

fn skeleton() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().h(px(56.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
        .child(div().h(px(56.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
        .child(div().h(px(56.)).bg(rgb(theme::SURFACE)).rounded(px(10.)))
}

fn data_view(data: &NetworkData) -> gpui::Div {
    let mut col = div().flex().flex_col().gap_4();
    col = col.child(section_header("Wi-Fi"));
    if data.wifi.is_empty() {
        col = col.child(empty_card("No wireless networks visible. (NetworkManager AP enumeration is a TODO.)"));
    } else {
        for w in &data.wifi {
            col = col.child(wifi_row(w));
        }
    }
    col = col.child(section_header("VPN"));
    if data.vpn.is_empty() {
        col = col.child(empty_card("No VPN connections configured."));
    } else {
        for v in &data.vpn {
            col = col.child(vpn_row(v));
        }
    }
    col
}

fn section_header(label: &'static str) -> gpui::Div {
    div().text_color(rgb(theme::TEXT_MUTED)).pt_2().child(label)
}

fn empty_card(message: &str) -> gpui::Div {
    div()
        .h(px(60.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(message.to_string())
}

fn wifi_row(w: &backend::WifiNetwork) -> gpui::Div {
    let chip = if w.secured && !w.connected {
        Some("Password required (coming soon)".to_string())
    } else {
        None
    };
    let mut col = div()
        .flex()
        .flex_col()
        .gap_1()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(div().text_color(rgb(theme::TEXT)).child(w.ssid.clone()))
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child(format!("{}%{}", w.strength, if w.connected { " • connected" } else { "" })),
                ),
        );
    if let Some(c) = chip {
        col = col.child(div().text_color(rgb(theme::ACCENT)).child(c));
    }
    col
}

fn vpn_row(v: &backend::VpnConnection) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .child(div().child(v.name.clone()))
        .child(
            div()
                .text_color(if v.active { rgb(theme::ACCENT) } else { rgb(theme::TEXT_MUTED) })
                .child(if v.active { "active" } else { "inactive" }.to_string()),
        )
}
