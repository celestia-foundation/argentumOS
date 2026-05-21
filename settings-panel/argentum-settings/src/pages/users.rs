//! Users page.

use crate::pages::{Page, PageState};
use crate::theme;
use crate::widgets::prompt;
use argentum_settings_core::dbus::accounts::{self as backend, AccountType, UserAccount};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct UsersPage {
    state: PageState<UserAccount>,
    password_pending: bool,
    password_status: Option<PasswordStatus>,
}

enum PasswordStatus {
    Ok,
    Err(String),
}

impl UsersPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self {
            state: PageState::Empty,
            password_pending: false,
            password_status: None,
        };
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

    fn start_change_password(&mut self, cx: &mut Context<Self>) {
        if self.password_pending {
            return;
        }
        let username = match &self.state {
            PageState::Loaded { data, .. } => data.username.clone(),
            _ => return,
        };
        self.password_pending = true;
        self.password_status = None;
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let new = match prompt::password("Change password", "New password").await {
                Ok(Some(s)) if !s.is_empty() => s,
                Ok(_) => {
                    weak.update(async_cx, |this, cx| {
                        this.password_pending = false;
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                Err(e) => {
                    weak.update(async_cx, |this, cx| {
                        this.password_pending = false;
                        this.password_status = Some(PasswordStatus::Err(e.to_string()));
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let confirm = match prompt::password("Confirm password", "Re-enter to confirm").await {
                Ok(Some(s)) => s,
                _ => {
                    weak.update(async_cx, |this, cx| {
                        this.password_pending = false;
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            if new != confirm {
                weak.update(async_cx, |this, cx| {
                    this.password_pending = false;
                    this.password_status = Some(PasswordStatus::Err("Passwords didn't match.".into()));
                    cx.notify();
                })
                .ok();
                return;
            }
            let result = backend::set_password(&username, &new).await;
            weak.update(async_cx, |this, cx| {
                this.password_pending = false;
                this.password_status = Some(match result {
                    Ok(()) => PasswordStatus::Ok,
                    Err(e) => PasswordStatus::Err(e.to_string()),
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for UsersPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Users.ttl()) {
            self.spawn_refresh(cx);
        }
        let pending = self.password_pending;
        let change_password_button = div()
            .id("change-password")
            .px_4()
            .py_2()
            .bg(rgb(if pending { theme::SIDEBAR } else { theme::ACCENT }))
            .rounded(px(8.))
            .text_color(rgb(theme::BG))
            .cursor_pointer()
            .on_click(cx.listener(|this, _event, _window, cx| this.start_change_password(cx)))
            .child(if pending { "Changing…" } else { "Change password…" }.to_string());

        let status_line = match &self.password_status {
            Some(PasswordStatus::Ok) => Some((theme::ACCENT, "Password updated.".to_string())),
            Some(PasswordStatus::Err(e)) => Some((theme::TEXT_MUTED, format!("Couldn't update: {e}"))),
            None => None,
        };

        let body = match &self.state {
            PageState::Loaded { data, .. } => account_view(data, change_password_button, status_line),
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

fn account_view(
    user: &UserAccount,
    change_password_button: impl IntoElement,
    status_line: Option<(u32, String)>,
) -> gpui::Div {
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
        .child(match status_line {
            Some((color, text)) => div().text_color(rgb(color)).child(text),
            None => div(),
        })
}
