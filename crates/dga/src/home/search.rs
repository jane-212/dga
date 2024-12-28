use gpui::{
    div, px, Div, Entity, EventEmitter, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use magnet::{Item, PreviewUrl};
use ui::{
    indicator::Indicator, label::Label, prelude::FluentBuilder, scroll::ScrollbarAxis,
    theme::ActiveTheme, Icon, Sizable, StyledExt,
};

pub struct Search {
    items: Vec<Item>,
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
    pub fn load(&mut self) {
        self.is_loading = true;
    }

    #[inline]
    pub fn loaded(&mut self, new_items: Vec<Item>) {
        self.is_loading = false;
        self.items = new_items;
        self.selected_item = None;
    }

    #[inline]
    pub fn load_error(&mut self) {
        self.is_loading = false;
    }

    #[inline]
    fn render_label(label: Label, icon: IconName) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_sm()
            .font_light()
            .child(Icon::new(icon))
            .child(label)
    }

    #[inline]
    fn render_items(&self, cx: &mut ViewContext<Self>) -> Div {
        let theme = cx.theme();
        let selected_item = self.selected_item;
        let len = self.items.len();

        div().size_full().pl_2().child(
            div().size_full().overflow_hidden().rounded_md().child(
                div()
                    .scrollable(cx.view().entity_id(), ScrollbarAxis::Vertical)
                    .children(self.items.iter().enumerate().map(|(idx, item)| {
                        let item = item.clone();

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
                                            .child(item.title),
                                    )
                                    .child(
                                        div()
                                            .text_color(theme.secondary_foreground)
                                            .flex()
                                            .justify_between()
                                            .pt_2()
                                            .child(Self::render_label(
                                                Label::new(item.size),
                                                IconName::Weight,
                                            ))
                                            .child(Self::render_label(
                                                Label::new(item.date),
                                                IconName::Calendar,
                                            )),
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
                                        cx.listener(move |this, _event, cx| {
                                            if selected_item
                                                .map(|selected| selected == idx)
                                                .unwrap_or(false)
                                            {
                                                return;
                                            }

                                            this.selected_item = Some(idx);
                                            cx.emit(SearchEvent::Preview(item.preview.clone()));
                                            cx.notify();
                                        }),
                                    ),
                            )
                    })),
            ),
        )
    }

    #[inline]
    fn render_loading() -> Div {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(Indicator::new().icon(IconName::Loader).large())
    }

    #[inline]
    fn render_content(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        match self.is_loading {
            true => Self::render_loading(),
            false => {
                if self.items.is_empty() {
                    div()
                        .size_full()
                        .flex()
                        .justify_center()
                        .items_center()
                        .child(Icon::new(IconName::Candy).large())
                } else {
                    self.render_items(cx)
                }
            }
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
    Preview(PreviewUrl),
}

impl EventEmitter<SearchEvent> for Search {}
