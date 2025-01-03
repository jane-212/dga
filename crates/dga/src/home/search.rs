use std::sync::Arc;

use gpui::{
    div, list, px, AnyElement, EventEmitter, InteractiveElement, IntoElement, ListAlignment,
    ListState, MouseButton, ParentElement, Render, Styled, View, ViewContext, VisualContext,
    WindowContext,
};
use icons::IconName;
use magnet::{FoundItem, Previewable};
use ui::{
    indicator::Indicator, label::Label, prelude::FluentBuilder, theme::ActiveTheme, Sizable,
    StyledExt,
};

pub struct Search {
    list_state: ListState,
    items: Vec<Box<dyn FoundItem>>,
    selected_item: Option<usize>,
    is_loading: bool,
}

impl Search {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let view = cx.view().downgrade();
            let list_state =
                ListState::new(
                    0,
                    ListAlignment::Top,
                    px(1000.0),
                    move |ix, cx| match view.upgrade() {
                        Some(view) => view.update(cx, |this: &mut Self, cx| {
                            this.render_list_item(ix, cx).into_any_element()
                        }),
                        None => div().into_any_element(),
                    },
                );

            Self {
                list_state,
                items: Vec::new(),
                selected_item: None,
                is_loading: false,
            }
        })
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn load(&mut self) {
        self.is_loading = true;
    }

    pub fn loaded(&mut self, new_items: Vec<Box<dyn FoundItem>>) {
        self.is_loading = false;
        self.items = new_items;
        self.list_state.reset(self.items.len());
        self.selected_item = None;
    }

    pub fn load_error(&mut self) {
        self.is_loading = false;
    }

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

    fn render_list_item(&mut self, ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let selected_item = self.selected_item;
        let item = &self.items[ix];
        let len = self.items.len();

        div()
            .when(ix != 0 && ix != len, |this| this.pt_1().pb_1())
            .when(ix == 0, |this| this.pt_2().pb_1())
            .when(ix == len - 1, |this| this.pt_1().pb_2())
            .px_2()
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
                        if selected == ix {
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
                                this.load_preview(selected_item, ix, url.clone(), cx);
                            }
                        }),
                    ),
            )
    }

    fn render_items(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(list(self.list_state.clone()).size_full())
    }

    fn render_loading() -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(Indicator::new().icon(IconName::Loader).large())
    }

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
