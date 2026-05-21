//! Sound page — output device list. Clicking a non-default sink makes it the
//! new default. Volume is rendered for context (master volume control is a
//! follow-up that needs a real slider widget).

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::sound::{self as backend, AudioSink};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::collections::HashSet;

pub struct SoundPage {
    state: PageState<Vec<AudioSink>>,
    in_flight: HashSet<String>,
}

impl SoundPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, in_flight: Default::default() };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::list_sinks().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(s) => PageState::Loaded { data: s, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn make_default(&mut self, name: String, cx: &mut Context<Self>) {
        if self.in_flight.contains(&name) {
            return;
        }
        // Optimistic flip.
        if let PageState::Loaded { data, .. } = &mut self.state {
            for s in data.iter_mut() {
                s.is_default = s.name == name;
            }
        }
        self.in_flight.insert(name.clone());
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let _ = backend::set_default_sink(&name).await;
            weak.update(async_cx, |this, cx| {
                this.in_flight.remove(&name);
                cx.notify();
                this.spawn_refresh(cx);
            })
            .ok();
        })
        .detach();
    }
}

impl Render for SoundPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Sound.ttl()) {
            self.spawn_refresh(cx);
        }
        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } if data.is_empty() => empty_view().into_any_element(),
            PageState::Loaded { data, .. } => sinks_view(data, &self.in_flight, cx).into_any_element(),
            PageState::Error(e) => error_view(e).into_any_element(),
            _ => skeleton().into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Sound"))
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
}

fn empty_view() -> gpui::Div {
    div()
        .h(px(120.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child("No audio output devices detected.")
}

fn error_view(msg: &str) -> gpui::Div {
    div()
        .h(px(120.))
        .px_4()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(10.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(format!("pactl: {msg}"))
}

fn sinks_view(
    sinks: &[AudioSink],
    in_flight: &HashSet<String>,
    cx: &mut Context<SoundPage>,
) -> gpui::Div {
    let mut col = div().flex().flex_col().gap_2();
    col = col.child(
        div()
            .text_color(rgb(theme::TEXT_MUTED))
            .child("Output device"),
    );
    for s in sinks {
        col = col.child(sink_row(s, in_flight.contains(&s.name), cx));
    }
    col
}

fn sink_row(s: &AudioSink, in_flight: bool, cx: &mut Context<SoundPage>) -> AnyElement {
    let name_owned = s.name.clone();
    let mut row = div()
        .id(SharedString::from(format!("sink:{}", s.name)))
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
        .on_click(cx.listener(move |this, _e, _w, cx| this.make_default(name_owned.clone(), cx)))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_color(rgb(theme::TEXT)).child(s.description.clone()))
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child(format!(
                            "{}{}",
                            s.name,
                            s.volume_percent
                                .map(|v| format!("  •  {v}%"))
                                .unwrap_or_default()
                        )),
                ),
        )
        .child(
            div()
                .text_color(if s.is_default { rgb(theme::ACCENT) } else { rgb(theme::TEXT_MUTED) })
                .child(if s.is_default { "Default" } else { "" }.to_string()),
        );
    if in_flight {
        row = row.border_b_2().border_color(rgb(theme::ACCENT));
    }
    row.into_any_element()
}
