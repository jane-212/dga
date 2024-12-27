use gpui::{
    div, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext,
    WindowContext,
};
use ui::theme::ActiveTheme;

pub struct Preview;

impl Preview {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self)
    }
}

impl Render for Preview {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div().size_full().bg(theme.secondary).child("Preview")
    }
}
