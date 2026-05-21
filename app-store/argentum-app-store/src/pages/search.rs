//! Search page — local AppStream catalog, fuzzy match on name + summary.
//!
//! Without a working inline GPUI text-input primitive at the pinned commit,
//! we collect the query through a `zenity --entry` modal. The "Search…"
//! button opens the modal; the page renders the ranked results below. The
//! infrastructure for inline-input search remains intact (`rank()` is
//! unchanged) — only the input collection mechanism is modal for now.

use crate::pages::components::{app_card, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use crate::widgets::prompt;
use argentum_app_store_core::appstream::{self, AppMeta};
use argentum_app_store_core::flathub_api::{self as api, AppSummary};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub struct SearchData {
    pub catalog: Vec<AppMeta>,
    pub popular: Vec<AppSummary>,
}

pub struct SearchPage {
    state: PageState<SearchData>,
    query: String,
    pending: bool,
}

impl SearchPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, query: String::new(), pending: false };
        s.spawn_refresh(cx);
        s
    }

    fn spawn_refresh(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, async_cx| {
            let catalog = appstream::load_remote("flathub").await.unwrap_or_default();
            let popular = api::popular(30).await.unwrap_or_default();
            weak.update(async_cx, |this, cx| {
                this.state = PageState::Loaded {
                    data: SearchData { catalog, popular },
                    fetched_at: std::time::Instant::now(),
                };
                cx.notify();
            }).ok();
        })
        .detach();
    }

    fn open_search_prompt(&mut self, cx: &mut Context<Self>) {
        if self.pending {
            return;
        }
        self.pending = true;
        let initial = self.query.clone();
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let q = prompt::text("Search apps", "What are you looking for?", &initial)
                .await
                .unwrap_or(None);
            weak.update(async_cx, |this, cx| {
                this.pending = false;
                if let Some(q) = q {
                    this.query = q;
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn clear_query(&mut self, cx: &mut Context<Self>) {
        self.query.clear();
        cx.notify();
    }
}

impl Render for SearchPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.should_refresh(Page::Search.ttl()) {
            self.spawn_refresh(cx);
        }

        let body: AnyElement = match &self.state {
            PageState::Loaded { data, .. } => loaded_view(data, &self.query, cx).into_any_element(),
            PageState::Error(e) => skeleton::error_view(e).into_any_element(),
            _ => skeleton::rows(3, 100.).into_any_element(),
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(div().text_xl().pb_4().child("Search"))
            .child(search_bar(&self.query, self.pending, cx))
            .child(div().h(px(12.)))
            .child(body)
    }
}

fn search_bar(query: &str, pending: bool, cx: &mut Context<SearchPage>) -> impl IntoElement {
    let label = if pending {
        "Waiting for input…".to_string()
    } else if query.is_empty() {
        "Click to search…".to_string()
    } else {
        format!("Searching: {query}")
    };
    let mut row = div()
        .id("search-bar")
        .h(px(40.))
        .px_3()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .bg(rgb(theme::SURFACE))
        .rounded(px(8.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::SIDEBAR)))
        .on_click(cx.listener(|this, _e, _w, cx| this.open_search_prompt(cx)))
        .child(div().text_color(rgb(theme::TEXT_MUTED)).child(label))
        .child(div().text_color(rgb(theme::ACCENT)).child("⌕"));
    if !query.is_empty() {
        let clear = div()
            .id("clear-query")
            .px_3()
            .py_1()
            .ml_2()
            .text_color(rgb(theme::TEXT_MUTED))
            .cursor_pointer()
            .on_click(cx.listener(|this, _e, _w, cx| this.clear_query(cx)))
            .child("Clear");
        row = row.child(clear);
    }
    row
}

fn loaded_view(data: &SearchData, query: &str, cx: &mut Context<SearchPage>) -> impl IntoElement {
    if query.is_empty() {
        return div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Popular on Flathub"))
            .child(popular_grid(&data.popular, cx))
            .into_any_element();
    }
    let results = rank(&data.catalog, query);
    if results.is_empty() {
        return skeleton::empty_view(&format!("No matches for “{query}”.")).into_any_element();
    }
    let mut grid = div().flex().flex_row().flex_wrap().gap_3();
    for app in results.into_iter().take(30) {
        let data = app_card::AppCardData {
            app_id: app.app_id.clone(),
            name: app.name.clone(),
            summary: app.summary.clone(),
            icon_url: None,
        };
        let card = app_card::render::<SearchPage>(data, cx, |_this, _id, _cx| {
            tracing::info!("search result clicked — detail view follow-up");
        });
        grid = grid.child(div().w(px(220.)).child(card));
    }
    grid.into_any_element()
}

fn popular_grid(items: &[AppSummary], cx: &mut Context<SearchPage>) -> impl IntoElement {
    if items.is_empty() {
        return skeleton::empty_view("No Flathub data yet — open this page once you have network.")
            .into_any_element();
    }
    let mut grid = div().flex().flex_row().flex_wrap().gap_3();
    for item in items.iter().take(20) {
        let data = app_card::AppCardData {
            app_id: item.app_id.clone(),
            name: item.name.clone(),
            summary: item.summary.clone(),
            icon_url: item.icon.clone(),
        };
        let card = app_card::render::<SearchPage>(data, cx, |_this, _id, _cx| {
            tracing::info!("popular card clicked — detail view follow-up");
        });
        grid = grid.child(div().w(px(220.)).child(card));
    }
    grid.into_any_element()
}

pub fn rank<'a>(catalog: &'a [AppMeta], query: &str) -> Vec<&'a AppMeta> {
    let q = query.to_ascii_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(i32, &AppMeta)> = catalog
        .iter()
        .filter_map(|m| {
            let name = m.name.to_ascii_lowercase();
            let summary = m.summary.to_ascii_lowercase();
            let app_id = m.app_id.to_ascii_lowercase();
            let score = if name == q {
                100
            } else if name.starts_with(&q) {
                80
            } else if name.contains(&q) {
                60
            } else if app_id.contains(&q) {
                40
            } else if summary.contains(&q) {
                20
            } else if m.keywords.iter().any(|k| k.to_ascii_lowercase().contains(&q)) {
                10
            } else {
                0
            };
            if score > 0 { Some((score, m)) } else { None }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, m)| m).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str, name: &str, summary: &str) -> AppMeta {
        AppMeta {
            app_id: id.into(),
            name: name.into(),
            summary: summary.into(),
            ..Default::default()
        }
    }

    #[test]
    fn ranks_name_prefix_above_summary() {
        let catalog = vec![
            meta("a", "Other", "kcalc is a calculator"),
            meta("b", "KCalc", "calculator"),
        ];
        let r = rank(&catalog, "kcalc");
        assert_eq!(r[0].app_id, "b");
    }
}
