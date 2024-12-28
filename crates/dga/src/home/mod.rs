mod preview;
mod search;

use gpui::{
    div, IntoElement, ParentElement, Render, SharedString, Styled, View, ViewContext,
    VisualContext, WeakView, WindowContext,
};
use icons::IconName;
use magnet::Magnet;
use preview::Preview;
use search::Search;
use ui::{notification::Notification, ContextModal};

use crate::{app::AppEvent, App};

pub struct Home {
    magnet: Magnet,
    search: View<Search>,
    search_loading: bool,
    preview: View<Preview>,
    preview_loading: bool,
}

impl Home {
    pub fn new(app: WeakView<App>, cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let magnet = Magnet::new().unwrap();
            let search = Search::new(cx);
            let preview = Preview::new(cx);
            cx.subscribe(&search, |this: &mut Self, _search, event, cx| match event {
                search::SearchEvent::Preview(url) => this.preview(url.clone(), cx),
            })
            .detach();
            if let Some(app) = app.upgrade() {
                let search = search.clone();
                cx.subscribe(&app, move |this, _app, event, cx| {
                    let search = search.clone();
                    if let AppEvent::Search(key) = event {
                        this.search(key.clone(), search, cx);
                    }
                })
                .detach();
            }

            Self {
                magnet,
                search,
                search_loading: false,
                preview,
                preview_loading: false,
            }
        })
    }

    fn preview(&mut self, preview: magnet::PreviewUrl, cx: &mut ViewContext<Self>) {
        if self.preview_loading {
            Self::notify_too_quick(cx);
            return;
        }
        self.preview_loading = true;
        let preview_view = self.preview.clone();
        preview_view.update(cx, |this, cx| {
            this.load();
            cx.notify();
        });
        let magnet = self.magnet.clone();
        let task = cx
            .background_executor()
            .spawn(async move { magnet.preview(preview).await });
        cx.spawn(|this, mut cx| async move {
            let result = match task.await {
                Ok(new_view) => preview_view.update(&mut cx, |this, cx| {
                    this.loaded(new_view);
                    cx.notify();
                }),
                Err(e) => preview_view.update(&mut cx, |this, cx| {
                    this.load_error();
                    cx.notify();
                    Self::notify_error(e.to_string(), cx);
                }),
            };
            if let Err(e) = result {
                eprintln!("{e:?}");
            }
            let result = this.update(&mut cx, |this, _cx| {
                this.preview_loading = false;
            });
            if let Err(e) = result {
                eprintln!("{e:?}");
            }
        })
        .detach();
    }

    #[inline]
    fn notify_too_quick<T: 'static>(cx: &mut ViewContext<T>) {
        cx.push_notification(Notification::new("太快啦").icon(IconName::Info));
    }

    #[inline]
    fn notify_error<T: 'static>(error: String, cx: &mut ViewContext<T>) {
        cx.push_notification(Notification::new(error).icon(IconName::CircleX));
    }

    fn search(&mut self, key: SharedString, search: View<Search>, cx: &mut ViewContext<Self>) {
        if self.search_loading {
            Self::notify_too_quick(cx);
            return;
        }
        self.search_loading = true;
        search.update(cx, |search, cx| {
            search.load();
            cx.notify();
        });
        let magnet = self.magnet.clone();
        let task = cx
            .background_executor()
            .spawn(async move { magnet.find(key).await });
        cx.spawn(|this, mut cx| async move {
            let result = match task.await {
                Ok(new_items) => search.update(&mut cx, |search, cx| {
                    search.loaded(new_items);
                    cx.notify();
                }),
                Err(e) => search.update(&mut cx, |search, cx| {
                    search.load_error();
                    cx.notify();
                    Self::notify_error(e.to_string(), cx);
                }),
            };
            if let Err(e) = result {
                eprintln!("{e:?}");
            }
            let result = this.update(&mut cx, |this, _cx| {
                this.search_loading = false;
            });
            if let Err(e) = result {
                eprintln!("{e:?}");
            }
        })
        .detach();
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
