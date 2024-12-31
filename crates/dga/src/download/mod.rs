use gpui::{
    div, IntoElement, ParentElement, Render, View, ViewContext, VisualContext, WindowContext,
};

pub struct Download {}

impl Download {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self {})
    }
}

impl Render for Download {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child("Download")
    }
}
