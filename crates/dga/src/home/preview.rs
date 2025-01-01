use std::sync::Arc;
use std::time::Duration;

use gpui::{
    div, img, ClipboardItem, Div, Entity, EventEmitter, IntoElement, ParentElement, Render,
    SharedString, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use magnet::FoundPreview;
use ui::{
    button::Button, indicator::Indicator, label::Label, notification::Notification,
    prelude::FluentBuilder, scroll::ScrollbarAxis, theme::ActiveTheme, ContextModal, Sizable,
    StyledExt,
};

use crate::LogErr;

pub struct Preview {
    is_loading: bool,
    copied: bool,
    added: bool,
    view: Option<Arc<dyn FoundPreview>>,
}

impl Preview {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self {
            is_loading: false,
            copied: false,
            added: false,
            view: None,
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
    pub fn loaded(&mut self, preview: Arc<dyn FoundPreview>) {
        self.is_loading = false;
        self.copied = false;
        self.added = false;
        self.view = Some(preview);
    }

    #[inline]
    pub fn load_error(&mut self) {
        self.is_loading = false;
    }

    #[inline]
    fn copy(&mut self, magnet: String, cx: &mut ViewContext<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(magnet));
        cx.push_notification(Notification::new("已复制").icon(IconName::Info));
        self.copied = true;
        cx.notify();
        Self::delay_remove_copy_check(cx);
    }

    #[inline]
    fn delay_remove_copy_check(cx: &mut ViewContext<Self>) {
        cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            this.update(&mut cx, |this, cx| {
                this.copied = false;
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    #[inline]
    fn delay_remove_add_check(cx: &mut ViewContext<Self>) {
        cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            this.update(&mut cx, |this, cx| {
                this.added = false;
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    #[inline]
    fn render_content(&self, preview: Arc<dyn FoundPreview>, cx: &mut ViewContext<Self>) -> Div {
        let theme = cx.theme();

        div().size_full().child(
            div()
                .scrollable(cx.view().entity_id(), ScrollbarAxis::Vertical)
                .child(
                    div()
                        .p_2()
                        .pr_3()
                        .child(
                            div()
                                .text_color(theme.primary)
                                .text_xl()
                                .font_bold()
                                .child(preview.title()),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .mt_2()
                                .text_color(theme.secondary_foreground)
                                .child(Label::new(preview.size()))
                                .child(Label::new(preview.date())),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .p_2()
                                .mt_2()
                                .bg(theme.background)
                                .border_1()
                                .border_color(theme.border)
                                .rounded_lg()
                                .child(
                                    div()
                                        .text_ellipsis()
                                        .overflow_hidden()
                                        .child(preview.magnet()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(
                                            Button::new("add-to-download")
                                                .icon(if self.added {
                                                    IconName::Check
                                                } else {
                                                    IconName::Plus
                                                })
                                                .small()
                                                .on_click(cx.listener({
                                                    let magnet = preview.magnet();
                                                    move |this, _event, cx| {
                                                        if !this.added {
                                                            cx.emit(PreviewEvent::AddToDownload(
                                                                magnet.clone(),
                                                            ));
                                                            this.added = true;
                                                            cx.notify();
                                                            Self::delay_remove_add_check(cx);
                                                        }
                                                    }
                                                })),
                                        )
                                        .child(
                                            Button::new("copy")
                                                .icon(if self.copied {
                                                    IconName::Check
                                                } else {
                                                    IconName::Copy
                                                })
                                                .small()
                                                .on_click(cx.listener({
                                                    let copied = self.copied;
                                                    let magnet = preview.magnet();
                                                    move |this, _event, cx| {
                                                        if !copied {
                                                            this.copy(magnet.to_string(), cx);
                                                        }
                                                    }
                                                })),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .mt_2()
                                .children(preview.images().into_iter().map(|url| {
                                    img(url).border_1().border_color(theme.border).rounded_md()
                                })),
                        ),
                ),
        )
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
            .when(!self.is_loading, |this| {
                this.when_some(self.view.clone(), |this, view| {
                    this.child(self.render_content(view, cx))
                })
            })
    }
}

pub enum PreviewEvent {
    AddToDownload(SharedString),
}

impl EventEmitter<PreviewEvent> for Preview {}
