mod preview;
mod search;

use gpui::{
    div, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext,
    WindowContext,
};
use icons::IconName;
use magnet::Magnet;
use preview::Preview;
use search::Search;
use ui::{notification::Notification, ContextModal};

pub struct Home {
    magnet: Magnet,
    search: View<Search>,
    preview: View<Preview>,
}

impl Home {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let magnet = Magnet::new().unwrap();
            let search = Search::new(cx);
            let preview = Preview::new(cx);
            cx.subscribe(&search, |home: &mut Self, search, event, cx| match event {
                search::SearchEvent::Search(key) => {
                    let magnet = home.magnet.clone();
                    let key = key.clone();
                    let find = cx
                        .background_executor()
                        .spawn(async move { magnet.find(key).await });
                    cx.spawn(|this, mut cx| async move {
                        let result = match find.await {
                            Ok(new_items) => search.update(&mut cx, |search, cx| {
                                search.set_items(new_items);
                                cx.notify();
                            }),
                            Err(e) => this.update(&mut cx, |_this, cx| {
                                cx.push_notification(
                                    Notification::new(e.to_string()).icon(IconName::CircleX),
                                );
                            }),
                        };
                        if let Err(e) = result {
                            eprintln!("{e:?}");
                        }
                    })
                    .detach();
                }
            })
            .detach();

            Self {
                magnet,
                search,
                preview,
            }
        })
    }
}

impl Render for Home {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .child(self.search.clone())
            .child(self.preview.clone())
    }
}
