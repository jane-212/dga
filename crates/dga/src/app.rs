use gpui::{
    div, px, ClickEvent, EventEmitter, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use ui::button::{Button, ButtonVariants};
use ui::prelude::FluentBuilder;
use ui::theme::ActiveTheme;
use ui::{
    theme::{Theme, ThemeMode},
    TitleBar,
};
use ui::{Root, Selectable, Sizable};

use super::home::Home;

pub struct App {
    focus_handle: FocusHandle,
    state: AppState,
    home: View<Home>,
}

#[derive(Clone, PartialEq)]
pub enum AppState {
    Home,
    License,
}

pub enum AppEvent {
    ChangeTo(AppState),
}

impl EventEmitter<AppEvent> for App {}

impl App {
    pub fn root(cx: &mut WindowContext) -> View<Root> {
        let app = cx.new_view(|cx| {
            #[cfg(not(target_os = "linux"))]
            cx.observe_window_appearance(|_, cx| {
                Theme::sync_system_appearance(cx);
            })
            .detach();
            cx.on_release(|_app, _window_handle, cx| {
                cx.quit();
            })
            .detach();
            let home = Home::new(cx);
            let focus_handle = cx.focus_handle();
            cx.focus(&focus_handle);

            Self {
                focus_handle,
                state: AppState::Home,
                home,
            }
        });
        cx.subscribe(&app, |app, event, cx| {
            let AppEvent::ChangeTo(state) = event;
            app.update(cx, |app, cx| {
                app.state = state.clone();
                cx.notify();
            });
        })
        .detach();

        cx.activate(true);

        cx.new_view(|cx| Root::new(app.into(), cx))
    }

    #[inline]
    fn change_color_mode(_event: &ClickEvent, cx: &mut WindowContext) {
        cx.stop_propagation();
        let mode = match cx.theme().mode.is_dark() {
            true => ThemeMode::Light,
            false => ThemeMode::Dark,
        };

        Theme::change(mode, cx);
    }

    #[inline]
    fn open_home_page(_event: &ClickEvent, cx: &mut WindowContext) {
        cx.stop_propagation();
        cx.open_url("https://github.com/jane-212/dga");
    }

    #[inline]
    fn switch_license(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        match self.state {
            AppState::Home => cx.emit(AppEvent::ChangeTo(AppState::License)),
            AppState::License => cx.emit(AppEvent::ChangeTo(AppState::Home)),
        }
    }

    #[inline]
    fn render_license(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div().size_full().flex().justify_center().child(
            div()
                .w(px(700.0))
                .min_w(px(700.0))
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .items_center()
                        .rounded_lg()
                        .shadow_sm()
                        .p_2()
                        .mt_2()
                        .bg(theme.secondary)
                        .text_sm()
                        .child("软件开源协议")
                        .border_1()
                        .border_color(theme.border),
                )
                .child(
                    div()
                        .rounded_lg()
                        .shadow_sm()
                        .p_2()
                        .mt_2()
                        .bg(theme.secondary)
                        .text_sm()
                        .child(include_str!("../../../LICENSE"))
                        .border_1()
                        .border_color(theme.border),
                ),
        )
    }

    #[inline]
    fn render_main(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        match self.state {
            AppState::Home => self.home.clone().into_any_element(),
            AppState::License => self.render_license(cx).into_any_element(),
        }
    }

    #[inline]
    fn render_title_bar(&mut self, cx: &mut ViewContext<Self>) -> TitleBar {
        TitleBar::new().child(div()).child(
            div()
                .flex()
                .justify_end()
                .items_center()
                .px_2()
                .gap_1()
                .child(
                    Button::new("theme-mode")
                        .map(|this| {
                            if cx.theme().mode.is_dark() {
                                this.icon(IconName::Moon)
                            } else {
                                this.icon(IconName::Sun)
                            }
                        })
                        .ghost()
                        .small()
                        .on_click(Self::change_color_mode),
                )
                .child(
                    Button::new("license")
                        .icon(IconName::CopyRight)
                        .ghost()
                        .small()
                        .selected(self.state == AppState::License)
                        .on_click(cx.listener(Self::switch_license)),
                )
                .child(
                    Button::new("github")
                        .icon(IconName::Github)
                        .ghost()
                        .small()
                        .on_click(Self::open_home_page),
                ),
        )
    }
}

impl Render for App {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let notification_layer = Root::render_notification_layer(cx);
        let title_bar = self.render_title_bar(cx);
        let main = self.render_main(cx);
        let theme = cx.theme();

        div()
            .size_full()
            .font_family(".SystemUIFont")
            .track_focus(&self.focus_handle)
            .text_color(theme.foreground)
            .bg(theme.background)
            .child(title_bar)
            .child(main)
            .child(div().absolute().top_8().children(notification_layer))
    }
}
