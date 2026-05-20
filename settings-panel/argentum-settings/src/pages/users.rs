//! Users page.

use crate::pages::{Page, PageState};
use crate::theme;
use argentum_settings_core::dbus::accounts::{self as backend, AccountType, UserAccount};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct UsersPage {
    state: PageState<UserAccount>,
}

impl UsersPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let result = backend::current().await;
            weak.update(async_cx, |this, cx| {
                this.state = match result {
                    Ok(u) => PageState::Loaded { data: u, fetched_at: std::time::Instant::now() },
                    Err(e) => PageState::Error(e.to_string()),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }
}

impl Render for UsersPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Users.ttl()) {
            self.spawn_refresh(cx);
        }
        // TODO: open a right-side sheet that prompts current + new + confirm
        // passwords and shells `passwd` via pkexec through a pty. For now, the
        // click is a no-op that records intent in the log so it's at least
        // visibly responsive (rather than a dead button).
        let change_password_button = div()
            .id("change-password")
            .px_4()
            .py_2()
            .bg(rgb(theme::BG))
            .rounded(px(8.))
            .text_color(rgb(theme::TEXT))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(theme::SIDEBAR)))
            .on_click(cx.listener(|_this, _event, _window, _cx| {
                tracing::info!("change-password click — modal TODO");
            }))
            .child("Change password…  (coming soon)");

        let body = match &self.state {
            PageState::Loaded { data, .. } => account_view(data, change_password_button),
            PageState::Error(e) => error_view(e),
            _ => skeleton(),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Users & Accounts"))
            .child(body)
    }
}

fn skeleton() -> gpui::Div {
    div().h(px(180.)).bg(rgb(theme::SURFACE)).rounded(px(10.))
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
        .child(format!("Couldn't read accounts: {msg}"))
}

fn account_view(user: &UserAccount, change_password_button: impl IntoElement) -> gpui::Div {
    let initials: String = user
        .real_name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let type_label = match user.account_type {
        AccountType::Administrator => "Administrator",
        AccountType::Standard => "Standard",
    };
    div()
        .flex()
        .flex_col()
        .gap_4()
        .px_6()
        .py_6()
        .bg(rgb(theme::SURFACE))
        .rounded(px(12.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .w(px(64.))
                        .h(px(64.))
                        .rounded_full()
                        .bg(rgb(theme::ACCENT))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(theme::BG))
                        .child(initials),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_color(rgb(theme::TEXT)).child(user.real_name.clone()))
                        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(user.username.clone()))
                        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(type_label.to_string())),
                ),
        )
        .child(change_password_button)
}
