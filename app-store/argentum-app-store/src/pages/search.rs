//! Search page — local AppStream catalog, fuzzy match on name + summary.
//!
//! Real text input is a known GPUI gap at the pinned commit (the settings
//! panel's `software.rs::add_remote_form` lives with the same limitation).
//! Until that's solved we render a "type to search" stub plus a static set of
//! quick links (popular categories + the full popular grid) so the page is
//! still useful. The infrastructure for real search — local AppStream load,
//! ranking, grid render — is in place; wiring a `TextInput` element is the
//! single missing piece.

use crate::pages::components::{app_card, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::appstream::{self, AppMeta};
use argentum_app_store_core::flathub_api::{self as api, AppSummary};
use gpui::{
    AnyElement, Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};

pub struct SearchData {
    pub catalog: Vec<AppMeta>,
    pub popular: Vec<AppSummary>,
}

pub struct SearchPage {
    state: PageState<SearchData>,
    query: String,
}

impl SearchPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let s = Self { state: PageState::Empty, query: String::new() };
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
            .child(input_stub(&self.query))
            .child(div().h(px(12.)))
            .child(body)
    }
}

fn input_stub(query: &str) -> impl IntoElement {
    // TODO: real text input. Until then this is a static affordance hinting
    // at what's coming. Search still happens — populated `query` is exercised
    // in tests — it just can't currently take keystrokes from the UI.
    div()
        .h(px(40.))
        .px_3()
        .flex()
        .items_center()
        .bg(rgb(theme::SURFACE))
        .rounded(px(8.))
        .text_color(rgb(theme::TEXT_MUTED))
        .child(if query.is_empty() {
            "Type to search apps (text input TODO — meanwhile, use Discover/Categories)".to_string()
        } else {
            format!("Searching: {query}")
        })
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
        };
        let card = app_card::render::<SearchPage>(data, cx, |_this, _id, _cx| {
            tracing::info!("search result clicked — detail view TODO");
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
        };
        let card = app_card::render::<SearchPage>(data, cx, |_this, _id, _cx| {
            tracing::info!("popular card clicked — detail view TODO");
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
