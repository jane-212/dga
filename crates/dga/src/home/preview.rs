use std::{sync::Arc, time::Duration};

use gpui::{
    div, img, list, px, ClipboardItem, EventEmitter, IntoElement, ListAlignment, ListState,
    ParentElement, Render, SharedString, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use magnet::{Bound, FoundPreview};
use ui::{
    button::Button, indicator::Indicator, label::Label, notification::Notification,
    prelude::FluentBuilder, theme::ActiveTheme, ContextModal, Sizable, StyledExt,
};
use utils::LogErr;

pub struct Preview {
    is_loading: bool,
    list_items: Vec<ListItem>,
    list_state: ListState,
}

impl Preview {
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
                            this.render_item(ix, cx).into_any_element()
                        }),
                        None => div().into_any_element(),
                    },
                );

            Self {
                is_loading: false,
                list_items: Vec::new(),
                list_state,
            }
        })
    }

    fn render_magnet(
        &mut self,
        ix: usize,
        bound: Arc<dyn Bound>,
        added: bool,
        copied: bool,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        div().pt_2().child(
            div()
                .p_2()
                .bg(theme.background)
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(Label::new(bound.size()))
                        .child(Label::new(bound.date())),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .text_ellipsis()
                                .overflow_hidden()
                                .child(bound.magnet()),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    Button::new(("add-to-download", ix))
                                        .icon(if added {
                                            IconName::Check
                                        } else {
                                            IconName::Plus
                                        })
                                        .small()
                                        .on_click(cx.listener(move |this, _event, cx| {
                                            this.added(ix, cx);
                                        })),
                                )
                                .child(
                                    Button::new(("copy", ix))
                                        .icon(if copied {
                                            IconName::Check
                                        } else {
                                            IconName::Copy
                                        })
                                        .small()
                                        .on_click(cx.listener(move |this, _event, cx| {
                                            this.copy(ix, cx);
                                        })),
                                ),
                        ),
                ),
        )
    }

    fn render_item(&mut self, ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let item = &self.list_items[ix];
        match item {
            ListItem::Title(title) => div()
                .text_color(theme.primary)
                .text_xl()
                .font_bold()
                .child(title.clone())
                .into_any_element(),
            ListItem::Bound(bound, added, copied) => self
                .render_magnet(ix, bound.clone(), *added, *copied, cx)
                .into_any_element(),
            ListItem::Image(url) => div()
                .pt_2()
                .child(
                    img(url.clone())
                        .border_1()
                        .border_color(theme.border)
                        .rounded_md(),
                )
                .into_any_element(),
        }
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn load(&mut self) {
        self.is_loading = true;
    }

    pub fn loaded(&mut self, preview: Box<dyn FoundPreview>) {
        self.is_loading = false;
        self.load_items_from_preview(preview);
        self.list_state.reset(self.list_items.len());
    }

    fn load_items_from_preview(&mut self, preview: Box<dyn FoundPreview>) {
        let mut new_items = Vec::new();
        new_items.push(ListItem::Title(preview.title()));
        for bound in preview.bounds() {
            new_items.push(ListItem::Bound(bound, false, false));
        }
        for image in preview.images() {
            new_items.push(ListItem::Image(image));
        }

        self.list_items = new_items;
    }

    pub fn load_error(&mut self) {
        self.is_loading = false;
    }

    fn copy(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        if let ListItem::Bound(bound, _, copied) = &mut self.list_items[ix] {
            if *copied {
                return;
            }

            cx.write_to_clipboard(ClipboardItem::new_string(bound.magnet().to_string()));
            cx.push_notification(Notification::new("已复制").icon(IconName::Info));
            *copied = true;
            cx.notify();
            Self::delay_remove_copy_check(ix, cx);
        }
    }

    fn added(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        if let ListItem::Bound(bound, added, _) = &mut self.list_items[ix] {
            if *added {
                return;
            }

            cx.emit(PreviewEvent::AddToDownload(bound.magnet()));
            *added = true;
            cx.notify();
            Self::delay_remove_add_check(ix, cx);
        }
    }

    fn delay_remove_copy_check(ix: usize, cx: &mut ViewContext<Self>) {
        cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            this.update(&mut cx, |this, cx| {
                if let ListItem::Bound(_, _, copied) = &mut this.list_items[ix] {
                    *copied = false;
                    cx.notify();
                }
            })
            .log_err();
        })
        .detach();
    }

    fn delay_remove_add_check(ix: usize, cx: &mut ViewContext<Self>) {
        cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            this.update(&mut cx, |this, cx| {
                if let ListItem::Bound(_, added, _) = &mut this.list_items[ix] {
                    *added = false;
                    cx.notify();
                }
            })
            .log_err();
        })
        .detach();
    }

    fn render_content(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_2()
            .child(list(self.list_state.clone()).size_full())
    }
}

impl Render for Preview {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .size_full()
            .overflow_hidden()
            .bg(theme.secondary)
            .flex()
            .justify_center()
            .items_center()
            .when(self.is_loading, |this| {
                this.child(Indicator::new().icon(IconName::Loader).large())
            })
            .when(!self.is_loading, |this| this.child(self.render_content(cx)))
    }
}

pub enum PreviewEvent {
    AddToDownload(SharedString),
}

impl EventEmitter<PreviewEvent> for Preview {}

enum ListItem {
    Title(SharedString),
    Bound(Arc<dyn Bound>, bool, bool),
    Image(SharedString),
}
