//! Page enum, router, and the `PageState<T>` cache primitive.

use crate::theme;
use clap::ValueEnum;
use gpui::{
    AnyElement, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window, div, rgb,
};
use std::time::{Duration, Instant};

pub mod categories;
pub mod discover;
pub mod installed;
pub mod permissions;
pub mod remotes;
pub mod runtimes;
pub mod search;
pub mod updates;

mod components;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Page {
    Discover = 0,
    Categories = 1,
    Search = 2,
    Installed = 3,
    Updates = 4,
    Permissions = 5,
    Remotes = 6,
    Runtimes = 7,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Discover => "Discover",
            Page::Categories => "Categories",
            Page::Search => "Search",
            Page::Installed => "Installed",
            Page::Updates => "Updates",
            Page::Permissions => "Permissions",
            Page::Remotes => "Remotes",
            Page::Runtimes => "Runtimes",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Page::Discover => "✦",
            Page::Categories => "☷",
            Page::Search => "⌕",
            Page::Installed => "▣",
            Page::Updates => "⟳",
            Page::Permissions => "◈",
            Page::Remotes => "⌬",
            Page::Runtimes => "⚙",
        }
    }

    pub fn ttl(self) -> Duration {
        match self {
            Page::Discover => Duration::from_secs(60 * 10),
            Page::Categories => Duration::from_secs(60 * 60),
            Page::Search => Duration::from_secs(60 * 10),
            Page::Installed => Duration::from_secs(30),
            Page::Updates => Duration::from_secs(60),
            Page::Permissions => Duration::from_secs(30),
            Page::Remotes => Duration::from_secs(30),
            Page::Runtimes => Duration::from_secs(30),
        }
    }
}

/// Stale-while-revalidate cache for a page's loaded data.
#[allow(dead_code)]
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
    discover: Entity<discover::DiscoverPage>,
    categories: Entity<categories::CategoriesPage>,
    search: Entity<search::SearchPage>,
    installed: Entity<installed::InstalledPage>,
    updates: Entity<updates::UpdatesPage>,
    permissions: Entity<permissions::PermissionsPage>,
    remotes: Entity<remotes::RemotesPage>,
    runtimes: Entity<runtimes::RuntimesPage>,
}

impl PagesView {
    pub fn new(initial: Page, deep_link_app: Option<String>, cx: &mut Context<Self>) -> Self {
        let discover = cx.new(|cx| {
            let mut p = discover::DiscoverPage::new(cx);
            if initial == Page::Discover {
                if let Some(id) = &deep_link_app {
                    p.open_detail(id.clone(), cx);
                }
            }
            p
        });
        Self {
            active: initial,
            discover,
            categories: cx.new(|cx| categories::CategoriesPage::new(cx)),
            search: cx.new(|cx| search::SearchPage::new(cx)),
            installed: cx.new(|cx| installed::InstalledPage::new(cx)),
            updates: cx.new(|cx| updates::UpdatesPage::new(cx)),
            permissions: cx.new(|cx| permissions::PermissionsPage::new(cx)),
            remotes: cx.new(|cx| remotes::RemotesPage::new(cx)),
            runtimes: cx.new(|cx| runtimes::RuntimesPage::new(cx)),
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
            Page::Discover => self.discover.clone().into_any_element(),
            Page::Categories => self.categories.clone().into_any_element(),
            Page::Search => self.search.clone().into_any_element(),
            Page::Installed => self.installed.clone().into_any_element(),
            Page::Updates => self.updates.clone().into_any_element(),
            Page::Permissions => self.permissions.clone().into_any_element(),
            Page::Remotes => self.remotes.clone().into_any_element(),
            Page::Runtimes => self.runtimes.clone().into_any_element(),
        };

        div()
            .id(("page-content", self.active as usize))
            .size_full()
            .bg(rgb(theme::BG))
            .child(content)
    }
}
