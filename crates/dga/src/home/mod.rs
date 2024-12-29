mod preview;
mod search;

use gpui::{
    div, IntoElement, ParentElement, Render, SharedString, Styled, Task, View, ViewContext,
    VisualContext, WeakView, WindowContext,
};
use icons::IconName;
use magnet::{Item, Magnet};
use preview::Preview;
use search::Search;
use ui::{notification::Notification, ContextModal};

use crate::{app::AppEvent, App, LogErr};

pub struct Home {
    magnet: Magnet,
    search: View<Search>,
    preview: View<Preview>,
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
                preview,
            }
        })
    }

    fn check_and_set_preview_to_loading(&self, cx: &mut ViewContext<Self>) -> bool {
        let is_loading = self.preview.read(cx).is_loading();
        match is_loading {
            true => {
                Self::notify_too_quick(cx);
            }
            false => {
                self.preview.update(cx, |this, cx| {
                    this.load();
                    cx.notify();
                });
            }
        }

        is_loading
    }

    #[inline]
    fn preview_task(
        &self,
        preview: magnet::PreviewUrl,
        cx: &mut ViewContext<Self>,
    ) -> Task<magnet::Result<magnet::Preview>> {
        let magnet = self.magnet.clone();
        cx.background_executor()
            .spawn(async move { magnet.preview(preview).await })
    }

    fn preview(&mut self, preview: magnet::PreviewUrl, cx: &mut ViewContext<Self>) {
        if self.check_and_set_preview_to_loading(cx) {
            return;
        }

        let task = self.preview_task(preview, cx);
        let preview_view = self.preview.clone();
        cx.spawn(|_this, mut cx| async move {
            match task.await {
                Ok(new_view) => preview_view.update(&mut cx, |this, cx| {
                    this.loaded(new_view);
                    cx.notify();
                }),
                Err(e) => preview_view.update(&mut cx, |this, cx| {
                    this.load_error();
                    cx.notify();
                    Self::notify_error(e.to_string(), cx);
                }),
            }
            .log_err();
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

    fn check_and_set_search_to_loading(&self, cx: &mut ViewContext<Self>) -> bool {
        let is_loading = self.search.read(cx).is_loading();
        match is_loading {
            true => {
                Self::notify_too_quick(cx);
            }
            false => {
                self.search.update(cx, |this, cx| {
                    this.load();
                    cx.notify();
                });
            }
        }

        is_loading
    }

    #[inline]
    fn search_task(
        &self,
        key: SharedString,
        cx: &mut ViewContext<Self>,
    ) -> Task<magnet::Result<Vec<Item>>> {
        let magnet = self.magnet.clone();
        cx.background_executor()
            .spawn(async move { magnet.find(key).await })
    }

    fn search(&mut self, key: SharedString, search: View<Search>, cx: &mut ViewContext<Self>) {
        if self.check_and_set_search_to_loading(cx) {
            return;
        }

        let task = self.search_task(key, cx);
        cx.spawn(|_this, mut cx| async move {
            match task.await {
                Ok(new_items) => search.update(&mut cx, |search, cx| {
                    search.loaded(new_items);
                    cx.notify();
                }),
                Err(e) => search.update(&mut cx, |search, cx| {
                    search.load_error();
                    cx.notify();
                    Self::notify_error(e.to_string(), cx);
                }),
            }
            .log_err();
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
