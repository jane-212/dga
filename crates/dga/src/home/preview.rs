use std::time::Duration;

use gpui::{
    div, img, ClipboardItem, Div, Entity, IntoElement, ParentElement, Render, Styled, View,
    ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use ui::{
    button::Button, indicator::Indicator, label::Label, prelude::FluentBuilder,
    scroll::ScrollbarAxis, theme::ActiveTheme, Sizable, StyledExt,
};

use crate::LogErr;

pub struct Preview {
    is_loading: bool,
    copied: bool,
    view: Option<magnet::Preview>,
}

impl Preview {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self {
            is_loading: false,
            copied: false,
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
    pub fn loaded(&mut self, preview: magnet::Preview) {
        self.is_loading = false;
        self.copied = false;
        self.view = Some(preview);
    }

    #[inline]
    pub fn load_error(&mut self) {
        self.is_loading = false;
    }

    #[inline]
    fn copy(&mut self, magnet: String, cx: &mut ViewContext<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(magnet));
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
    fn render_content(&self, preview: magnet::Preview, cx: &mut ViewContext<Self>) -> Div {
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
                                .child(preview.title),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .mt_2()
                                .text_color(theme.secondary_foreground)
                                .child(Label::new(preview.size))
                                .child(Label::new(preview.date)),
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
                                        .child(preview.magnet.clone()),
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
                                            move |this, _event, cx| {
                                                if !copied {
                                                    this.copy(preview.magnet.to_string(), cx);
                                                }
                                            }
                                        })),
                                ),
                        )
                        .child(div().mt_2().children(preview.images.into_iter().map(|url| {
                            img(url).border_1().border_color(theme.border).rounded_md()
                        }))),
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
