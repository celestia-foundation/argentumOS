//! Page enum, router, and the `PageState<T>` cache primitive.

use crate::theme;
use clap::ValueEnum;
use gpui::{
    AnyElement, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window, div, rgb,
};
use std::time::{Duration, Instant};

pub mod appearance;
pub mod datetime;
pub mod display;
pub mod network;
pub mod software;
pub mod sound;
pub mod system;
pub mod users;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Page {
    Appearance = 0,
    Display = 1,
    Sound = 2,
    Network = 3,
    Users = 4,
    Software = 5,
    DateTime = 6,
    System = 7,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Appearance => "Appearance",
            Page::Display => "Display",
            Page::Sound => "Sound",
            Page::Network => "Network",
            Page::Users => "Users",
            Page::Software => "Software",
            Page::DateTime => "Date & Time",
            Page::System => "System",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Page::Appearance => "◐",
            Page::Display => "▭",
            Page::Sound => "♪",
            Page::Network => "≋",
            Page::Users => "◉",
            Page::Software => "⬚",
            Page::DateTime => "⌚",
            Page::System => "⚙",
        }
    }

    pub fn ttl(self) -> Duration {
        match self {
            Page::Appearance => Duration::from_secs(5 * 60),
            Page::Display => Duration::from_secs(10),
            Page::Sound => Duration::from_secs(15),
            Page::Network => Duration::from_secs(5),
            Page::Users => Duration::from_secs(60),
            Page::Software => Duration::from_secs(30),
            Page::DateTime => Duration::from_secs(30),
            Page::System => Duration::from_secs(60),
        }
    }
}

/// Stale-while-revalidate cache for a page's loaded data.
#[allow(dead_code)] // `Loading` and `data()` are part of the API surface, not all paths use them yet
pub enum PageState<T> {
    Empty,
    Loading,
    Loaded { data: T, fetched_at: Instant },
    Error(String),
}

impl<T> Default for PageState<T> {
    fn default() -> Self {
        PageState::Empty
    }
}

#[allow(dead_code)]
impl<T> PageState<T> {
    pub fn should_refresh(&self, ttl: Duration) -> bool {
        match self {
            PageState::Empty => true,
            PageState::Loading => false,
            PageState::Loaded { fetched_at, .. } => fetched_at.elapsed() >= ttl,
            PageState::Error(_) => true,
        }
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            PageState::Loaded { data, .. } => Some(data),
            _ => None,
        }
    }
}

/// Host view: holds every page entity, toggles which is visible.
pub struct PagesView {
    active: Page,
    appearance: Entity<appearance::AppearancePage>,
    display: Entity<display::DisplayPage>,
    sound: Entity<sound::SoundPage>,
    network: Entity<network::NetworkPage>,
    users: Entity<users::UsersPage>,
    software: Entity<software::SoftwarePage>,
    datetime: Entity<datetime::DateTimePage>,
    system: Entity<system::SystemPage>,
}

impl PagesView {
    pub fn new(initial: Page, cx: &mut Context<Self>) -> Self {
        Self {
            active: initial,
            appearance: cx.new(|cx| appearance::AppearancePage::new(cx)),
            display: cx.new(|cx| display::DisplayPage::new(cx)),
            sound: cx.new(|cx| sound::SoundPage::new(cx)),
            network: cx.new(|cx| network::NetworkPage::new(cx)),
            users: cx.new(|cx| users::UsersPage::new(cx)),
            software: cx.new(|cx| software::SoftwarePage::new(cx)),
            datetime: cx.new(|cx| datetime::DateTimePage::new(cx)),
            system: cx.new(|cx| system::SystemPage::new(cx)),
        }
    }

    pub fn set_active(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.active == page {
            return;
        }
        self.active = page;
        cx.notify();
    }
}

impl Render for PagesView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content: AnyElement = match self.active {
            Page::Appearance => self.appearance.clone().into_any_element(),
            Page::Display => self.display.clone().into_any_element(),
            Page::Sound => self.sound.clone().into_any_element(),
            Page::Network => self.network.clone().into_any_element(),
            Page::Users => self.users.clone().into_any_element(),
            Page::Software => self.software.clone().into_any_element(),
            Page::DateTime => self.datetime.clone().into_any_element(),
            Page::System => self.system.clone().into_any_element(),
        };

        div()
            .id(("page-content", self.active as usize))
            .size_full()
            .bg(rgb(theme::BG))
            .child(content)
    }
}
