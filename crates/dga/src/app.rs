use gpui::{
    div, px, ClickEvent, EventEmitter, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, RenderOnce, SharedString, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use ui::button::{Button, ButtonVariants};
use ui::input::{InputEvent, TextInput};
use ui::theme::ActiveTheme;
use ui::{
    theme::{Theme, ThemeMode},
    TitleBar, TITLE_BAR_HEIGHT,
};
use ui::{Icon, Root, Selectable, Sizable, StyledExt};

use super::download::Download;
use super::home::Home;

pub struct App {
    focus_handle: FocusHandle,
    state: AppState,
    home: View<Home>,
    download: View<Download>,
    search_input: View<TextInput>,
}

#[derive(Clone, PartialEq)]
pub enum AppState {
    Home,
    Download,
    License(FromState),
}

#[derive(Clone, PartialEq)]
pub enum FromState {
    Home,
    Download,
}

impl From<&FromState> for AppState {
    fn from(value: &FromState) -> Self {
        match value {
            FromState::Home => AppState::Home,
            FromState::Download => AppState::Download,
        }
    }
}

pub enum AppEvent {
    ChangeTo(AppState),
    Search(SharedString),
}

impl EventEmitter<AppEvent> for App {}

impl App {
    pub fn root(cx: &mut WindowContext) -> View<Root> {
        let app = Self::new(cx);
        cx.subscribe(&app, Self::handle_app_event).detach();
        cx.activate(true);

        cx.new_view(|cx| Root::new(app.into(), cx))
    }

    fn handle_app_event(this: View<Self>, event: &AppEvent, cx: &mut WindowContext) {
        if let AppEvent::ChangeTo(state) = event {
            this.update(cx, |app, cx| {
                app.state = state.clone();
                cx.notify();
            });
        }
    }

    fn new_search_input(cx: &mut ViewContext<Self>) -> View<TextInput> {
        let input = cx.new_view(|cx| {
            TextInput::new(cx)
                .placeholder("搜索")
                .appearance(false)
                .xsmall()
                .prefix(|_cx| div().pl_2().child(Icon::new(IconName::Search).small()))
        });
        cx.subscribe(&input, |this: &mut Self, input, event, cx| {
            if let InputEvent::PressEnter = event {
                this.search(input, cx);
            }
        })
        .detach();

        input
    }

    fn new_home(cx: &mut ViewContext<Self>) -> View<Home> {
        let app = cx.view().downgrade();
        Home::new(app, cx)
    }

    fn new_focus_handle(cx: &mut ViewContext<Self>) -> FocusHandle {
        let handle = cx.focus_handle();
        cx.focus(&handle);

        handle
    }

    fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            #[cfg(not(target_os = "linux"))]
            cx.observe_window_appearance(|_, cx| {
                Theme::sync_system_appearance(cx);
            })
            .detach();
            cx.on_release(|_app, _window_handle, cx| {
                cx.quit();
            })
            .detach();

            let search_input = Self::new_search_input(cx);
            let home = Self::new_home(cx);
            let download = Download::new(cx);
            let focus_handle = Self::new_focus_handle(cx);

            Self {
                search_input,
                focus_handle,
                state: AppState::Home,
                home,
                download,
            }
        })
    }

    #[inline]
    fn search(&mut self, input: View<TextInput>, cx: &mut ViewContext<Self>) {
        let text = input.read(cx).text();
        if text.is_empty() {
            return;
        }
        Self::clear_input(input, cx);
        cx.emit(AppEvent::Search(text));
    }

    #[inline]
    fn clear_input(input: View<TextInput>, cx: &mut ViewContext<Self>) {
        input.update(cx, |input, cx| {
            input.set_text("", cx);
        });
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
            AppState::Home => cx.emit(AppEvent::ChangeTo(AppState::License(FromState::Home))),
            AppState::Download => {
                cx.emit(AppEvent::ChangeTo(AppState::License(FromState::Download)))
            }
            AppState::License(ref to) => cx.emit(AppEvent::ChangeTo(to.into())),
        }
    }

    #[inline]
    fn render_main(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let total_height = cx.viewport_size().height;
        let height = total_height - TITLE_BAR_HEIGHT;
        let base = div().w_full().h(height);

        match self.state {
            AppState::Home => base.child(self.home.clone()),
            AppState::Download => base.child(self.download.clone()),
            AppState::License(_) => base.child(License),
        }
    }

    #[inline]
    fn render_title_start(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .justify_end()
            .items_center()
            .px_2()
            .gap_1()
            .child(
                Button::new("home")
                    .icon(IconName::House)
                    .ghost()
                    .small()
                    .selected(matches!(self.state, AppState::Home))
                    .on_click(cx.listener(Self::set_to_home)),
            )
            .child(
                Button::new("download")
                    .icon(IconName::HardDriveDownload)
                    .ghost()
                    .small()
                    .selected(matches!(self.state, AppState::Download))
                    .on_click(cx.listener(Self::set_to_download)),
            )
    }

    #[inline]
    fn set_to_home(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.emit(AppEvent::ChangeTo(AppState::Home));
    }

    #[inline]
    fn set_to_download(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.emit(AppEvent::ChangeTo(AppState::Download));
    }

    #[inline]
    fn render_title_middle(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        match self.state {
            AppState::Home => div()
                .w(px(200.0))
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .child(self.search_input.clone()),
            AppState::Download => div(),
            AppState::License(_) => div(),
        }
    }

    #[inline]
    fn theme_icon(cx: &mut ViewContext<Self>) -> IconName {
        match cx.theme().mode {
            ThemeMode::Light => IconName::Sun,
            ThemeMode::Dark => IconName::Moon,
        }
    }

    #[inline]
    fn render_title_end(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .justify_end()
            .items_center()
            .px_2()
            .gap_1()
            .child(
                Button::new("theme-mode")
                    .icon(Self::theme_icon(cx))
                    .ghost()
                    .small()
                    .on_click(Self::change_color_mode),
            )
            .child(
                Button::new("license")
                    .icon(IconName::CopyRight)
                    .ghost()
                    .small()
                    .selected(matches!(self.state, AppState::License(_)))
                    .on_click(cx.listener(Self::switch_license)),
            )
            .child(
                Button::new("github")
                    .icon(IconName::Github)
                    .ghost()
                    .small()
                    .on_click(Self::open_home_page),
            )
    }

    #[inline]
    fn render_title_bar(&mut self, cx: &mut ViewContext<Self>) -> TitleBar {
        let title_start = self.render_title_start(cx);
        let title_middle = self.render_title_middle(cx);
        let title_end = self.render_title_end(cx);

        TitleBar::new()
            .child(title_start)
            .child(title_middle)
            .child(title_end)
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

#[derive(IntoElement)]
struct License;

impl RenderOnce for License {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .w(px(700.0))
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .items_center()
                            .rounded_lg()
                            .shadow_sm()
                            .p_2()
                            .bg(theme.secondary)
                            .text_sm()
                            .font_bold()
                            .text_color(theme.primary)
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
}
