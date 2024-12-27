use gpui::{
    div, px, EventEmitter, IntoElement, ParentElement, Render, SharedString, Styled, View,
    ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use magnet::Item;
use ui::{
    input::{InputEvent, TextInput},
    theme::ActiveTheme,
    Icon,
};

pub struct Search {
    search_input: View<TextInput>,
    items: Vec<Item>,
    is_loading: bool,
}

impl Search {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let search_input = cx.new_view(|cx| {
                TextInput::new(cx)
                    .placeholder("搜索")
                    .appearance(false)
                    .prefix(|_cx| div().pl_2().child(Icon::new(IconName::Search)))
            });
            cx.subscribe(&search_input, |this: &mut Self, input, event, cx| {
                if let InputEvent::PressEnter = event {
                    this.search(cx, input);
                }
            })
            .detach();

            Self {
                search_input,
                items: Vec::new(),
                is_loading: false,
            }
        })
    }

    #[inline]
    fn search(&mut self, cx: &mut ViewContext<Self>, input: View<TextInput>) {
        let search_text = input.read(cx).text();
        if search_text.is_empty() {
            return;
        }
        input.update(cx, |input, cx| {
            input.set_text("", cx);
        });
        cx.emit(SearchEvent::Search(search_text));
    }

    #[inline]
    pub fn set_items(&mut self, new_items: Vec<Item>) {
        self.items = new_items;
    }
}

impl Render for Search {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .w(px(500.0))
            .h_full()
            .child(
                div()
                    .m_2()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_lg()
                    .child(self.search_input.clone()),
            )
            .child(
                div().children(
                    self.items
                        .iter()
                        .map(|item| item.title.clone())
                        .collect::<Vec<SharedString>>(),
                ),
            )
    }
}

pub enum SearchEvent {
    Search(SharedString),
}

impl EventEmitter<SearchEvent> for Search {}
