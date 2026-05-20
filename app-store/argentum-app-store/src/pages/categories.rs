//! Categories page — list of standard Flathub categories. Selecting one
//! fetches `collection/category/<name>/30` and renders a grid.

use crate::pages::components::{app_card, skeleton};
use crate::pages::{Page, PageState};
use crate::theme;
use argentum_app_store_core::flathub_api::{self as api, AppSummary};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

pub enum View {
    Index,
    Category { name: String, state: PageState<Vec<AppSummary>> },
}

pub struct CategoriesPage {
    view: View,
}

impl CategoriesPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { view: View::Index }
    }

    fn open(&mut self, name: String, cx: &mut Context<Self>) {
        self.view = View::Category { name: name.clone(), state: PageState::Loading };
        cx.notify();
        cx.spawn(async move |weak, async_cx| {
            let result = api::category(&name, 30).await;
            weak.update(async_cx, |this, cx| {
                if let View::Category { state, .. } = &mut this.view {
                    *state = match result {
                        Ok(d) => PageState::Loaded { data: d, fetched_at: std::time::Instant::now() },
                        Err(e) => PageState::Error(e.to_string()),
                    };
                    cx.notify();
                }
            }).ok();
        })
        .detach();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.view = View::Index;
        cx.notify();
    }
}

impl Render for CategoriesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Refresh stale category grids.
        if let View::Category { state, .. } = &self.view {
            if state.should_refresh(Page::Categories.ttl()) {
                if let View::Category { name, .. } = &self.view {
                    let name = name.clone();
                    cx.spawn(async move |weak, async_cx| {
                        let result = api::category(&name, 30).await;
                        weak.update(async_cx, |this, cx| {
                            if let View::Category { state, .. } = &mut this.view {
                                *state = match result {
                                    Ok(d) => PageState::Loaded {
                                        data: d,
                                        fetched_at: std::time::Instant::now(),
                                    },
                                    Err(e) => PageState::Error(e.to_string()),
                                };
                                cx.notify();
                            }
                        })
                        .ok();
                    })
                    .detach();
                }
            }
        }

        let body: AnyElement = match &self.view {
            View::Index => index_view(cx).into_any_element(),
            View::Category { state, .. } => match state {
                PageState::Loaded { data, .. } if data.is_empty() => {
                    skeleton::empty_view("No apps in this category yet.").into_any_element()
                }
                PageState::Loaded { data, .. } => grid(data, cx).into_any_element(),
                PageState::Error(e) => skeleton::error_view(e).into_any_element(),
                _ => skeleton::rows(3, 100.).into_any_element(),
            },
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .child(header(&self.view, cx))
            .child(body)
    }
}

fn header(view: &View, cx: &mut Context<CategoriesPage>) -> AnyElement {
    match view {
        View::Index => div().text_xl().pb_4().child("Categories").into_any_element(),
        View::Category { name, .. } => div()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .pb_4()
            .child(
                div()
                    .id("back-categories")
                    .px_3()
                    .py_1()
                    .bg(rgb(theme::SURFACE))
                    .rounded(px(6.))
                    .text_color(rgb(theme::TEXT_MUTED))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _e, _w, cx| this.back(cx)))
                    .child("← Categories"),
            )
            .child(div().text_xl().text_color(rgb(theme::TEXT)).child(name.clone()))
            .into_any_element(),
    }
}

fn index_view(cx: &mut Context<CategoriesPage>) -> impl IntoElement {
    let mut col = div().flex().flex_col().gap_2();
    for cat in api::standard_categories() {
        let cat_owned = cat.to_string();
        col = col.child(
            div()
                .id(SharedString::from(format!("category-row:{cat}")))
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
                .on_click(cx.listener(move |this, _e, _w, cx| this.open(cat_owned.clone(), cx)))
                .child(div().text_color(rgb(theme::TEXT)).child(cat.to_string()))
                .child(div().text_color(rgb(theme::TEXT_MUTED)).child("→")),
        );
    }
    col
}

fn grid(items: &[AppSummary], cx: &mut Context<CategoriesPage>) -> impl IntoElement {
    let mut grid = div().flex().flex_row().flex_wrap().gap_3();
    for item in items.iter().take(30) {
        let data = app_card::AppCardData {
            app_id: item.app_id.clone(),
            name: item.name.clone(),
            summary: item.summary.clone(),
        };
        let card = app_card::render::<CategoriesPage>(data, cx, |_this, _id, _cx| {
            // TODO: route into a detail page once Categories owns its own
            // detail view (or routes back to Discover with the deep link).
            tracing::info!("category card clicked — detail view TODO");
        });
        grid = grid.child(div().w(px(220.)).child(card));
    }
    grid
}
