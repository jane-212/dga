use std::sync::Arc;

use gpui::{
    div, px, AnyElement, Entity, EventEmitter, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use magnet::{FoundItem, Previewable};
use ui::{
    indicator::Indicator, label::Label, prelude::FluentBuilder, scroll::ScrollbarAxis,
    theme::ActiveTheme, Sizable, StyledExt,
};

pub struct Search {
    items: Vec<Box<dyn FoundItem>>,
    selected_item: Option<usize>,
    is_loading: bool,
}

impl Search {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self {
            items: Vec::new(),
            selected_item: None,
            is_loading: false,
        })
    }

    #[inline]
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    #[inline]
    pub fn load(&mut self) {
        self.is_loading = true;
    }

    #[inline]
    pub fn loaded(&mut self, new_items: Vec<Box<dyn FoundItem>>) {
        self.is_loading = false;
        self.items = new_items;
        self.selected_item = None;
    }

    #[inline]
    pub fn load_error(&mut self) {
        self.is_loading = false;
    }

    #[inline]
    fn load_preview(
        &mut self,
        selected_item: Option<usize>,
        idx: usize,
        url: Arc<dyn Previewable>,
        cx: &mut ViewContext<Self>,
    ) {
        if selected_item
            .map(|selected| selected == idx)
            .unwrap_or(false)
        {
            return;
        }

        self.selected_item = Some(idx);
        cx.emit(SearchEvent::Preview(url));
        cx.notify();
    }

    #[inline]
    fn render_items(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let selected_item = self.selected_item;
        let len = self.items.len();

        div()
            .size_full()
            .pl_2()
            .scrollable(cx.view().entity_id(), ScrollbarAxis::Vertical)
            .children(self.items.iter().enumerate().map(|(idx, item)| {
                div()
                    .when(idx != 0 && idx != len, |this| this.pt_1().pb_1())
                    .when(idx == 0, |this| this.pt_2().pb_1())
                    .when(idx == len - 1, |this| this.pt_1().pb_2())
                    .pr_3()
                    .child(
                        div()
                            .p_2()
                            .bg(theme.secondary)
                            .rounded_md()
                            .shadow_sm()
                            .border_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .font_bold()
                                    .text_lg()
                                    .text_color(theme.primary)
                                    .text_ellipsis()
                                    .child(item.title()),
                            )
                            .child(
                                div()
                                    .text_color(theme.secondary_foreground)
                                    .flex()
                                    .justify_between()
                                    .pt_2()
                                    .child(Label::new(item.size()))
                                    .child(Label::new(item.date())),
                            )
                            .when_some(selected_item, |this, selected| {
                                if selected == idx {
                                    this.shadow_none().border_color(theme.primary_active)
                                } else {
                                    this
                                }
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener({
                                    let url = item.url();
                                    move |this, _event, cx| {
                                        this.load_preview(selected_item, idx, url.clone(), cx);
                                    }
                                }),
                            ),
                    )
            }))
    }

    #[inline]
    fn render_loading() -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(Indicator::new().icon(IconName::Loader).large())
    }

    #[inline]
    fn render_content(&self, cx: &mut ViewContext<Self>) -> AnyElement {
        match self.is_loading {
            true => Self::render_loading().into_any_element(),
            false => self.render_items(cx).into_any_element(),
        }
    }
}

impl Render for Search {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let width = px(300.0);
        let theme = cx.theme();

        div()
            .w(width)
            .max_w(width)
            .min_w(width)
            .border_r_1()
            .border_color(theme.border)
            .child(self.render_content(cx))
    }
}

pub enum SearchEvent {
    Preview(Arc<dyn Previewable>),
}

impl EventEmitter<SearchEvent> for Search {}
