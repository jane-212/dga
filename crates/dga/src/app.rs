use gpui::{
    div, px, ClickEvent, EventEmitter, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, RenderOnce, SharedString, Styled, View, ViewContext, VisualContext, WindowContext,
};
use icons::IconName;
use ui::button::{Button, ButtonVariants};
use ui::input::{InputEvent, TextInput};
use ui::prelude::FluentBuilder;
use ui::theme::ActiveTheme;
use ui::{
    theme::{Theme, ThemeMode},
    TitleBar, TITLE_BAR_HEIGHT,
};
use ui::{Icon, Root, Selectable, Sizable, StyledExt};

use super::download::Download;
use super::home::Home;
use crate::download::DownloadEvent;
use crate::home::HomeEvent;

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
        cx.on_window_should_close(|cx| {
            let bounds = cx.bounds();
            let width = bounds.size.width.0;
            let height = bounds.size.height.0;
            utils::write_window(width, height);

            true
        });
        cx.activate(true);

        cx.new_view(|cx| Root::new(app.into(), cx))
    }

    fn handle_app_event(this: View<Self>, event: &AppEvent, cx: &mut WindowContext) {
        if let AppEvent::ChangeTo(state) = event {
            this.update(cx, |app, cx| {
                if app.download.read(cx).has_login() {
                    match state {
                        AppState::Download => {
                            app.download.update(cx, |this, cx| {
                                this.resume(cx);
                            });
                        }
                        _ => {
                            app.download.update(cx, |this, cx| {
                                this.pause(cx);
                            });
                        }
                    }
                }

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
                .prefix(|cx| {
                    let theme = cx.theme();

                    div().pl_2().child(
                        Icon::new(IconName::Search)
                            .text_color(theme.primary)
                            .small(),
                    )
                })
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
            cx.subscribe(&home, |this, _home, event, cx| match event {
                HomeEvent::AddToDownload(new) => this.add_new(new.clone(), cx),
            })
            .detach();
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

    fn search(&mut self, input: View<TextInput>, cx: &mut ViewContext<Self>) {
        let text = input.read(cx).text();
        if text.is_empty() {
            return;
        }
        Self::clear_input(input, cx);
        cx.emit(AppEvent::Search(text));
    }

    fn clear_input(input: View<TextInput>, cx: &mut ViewContext<Self>) {
        input.update(cx, |input, cx| {
            input.set_text("", cx);
        });
    }

    fn change_color_mode(_event: &ClickEvent, cx: &mut WindowContext) {
        cx.stop_propagation();

        let mode = match cx.theme().mode.is_dark() {
            true => ThemeMode::Light,
            false => ThemeMode::Dark,
        };
        Theme::change(mode, cx);
    }

    fn open_home_page(_event: &ClickEvent, cx: &mut WindowContext) {
        cx.stop_propagation();
        cx.open_url("https://github.com/jane-212/dga");
    }

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

    fn render_title_start(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .justify_start()
            .items_center()
            .w_1_3()
            .px_2()
            .gap_1()
            .child(
                Button::new("home")
                    .icon(Icon::new(IconName::House).text_color(theme.primary))
                    .ghost()
                    .small()
                    .tooltip("首页")
                    .selected(matches!(self.state, AppState::Home))
                    .on_click(cx.listener(Self::set_to_home)),
            )
            .child(
                Button::new("download")
                    .icon(Icon::new(IconName::HardDriveDownload).text_color(theme.primary))
                    .ghost()
                    .small()
                    .tooltip("下载")
                    .selected(matches!(self.state, AppState::Download))
                    .on_click(cx.listener(Self::set_to_download)),
            )
    }

    fn set_to_home(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.emit(AppEvent::ChangeTo(AppState::Home));
    }

    fn set_to_download(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.emit(AppEvent::ChangeTo(AppState::Download));
    }

    fn logout_download(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.download.update(cx, |this, cx| {
            this.logout(cx);
        });
    }

    fn pause_all(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.download.update(cx, |_this, cx| {
            cx.emit(DownloadEvent::PauseAll);
        })
    }

    fn resume_all(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.download.update(cx, |_this, cx| {
            cx.emit(DownloadEvent::ResumeAll);
        })
    }

    fn add_new(&mut self, new: SharedString, cx: &mut ViewContext<Self>) {
        self.download.update(cx, |_this, cx| {
            cx.emit(DownloadEvent::AddNew(new));
        })
    }

    fn add_from_clipboard(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.download.update(cx, |_this, cx| {
            cx.emit(DownloadEvent::AddFromClipboard);
        })
    }

    fn render_title_middle(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let base = div().w_1_3();

        match self.state {
            AppState::Home => base
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .child(self.search_input.clone()),
            AppState::Download => match self.download.read(cx).has_login() {
                true => base
                    .flex()
                    .justify_center()
                    .items_center()
                    .px_2()
                    .gap_1()
                    .child(
                        Button::new("logout")
                            .icon(Icon::new(IconName::LogOut).text_color(ui::red_400()))
                            .ghost()
                            .small()
                            .tooltip("退出登录")
                            .on_click(cx.listener(Self::logout_download)),
                    )
                    .child(
                        Button::new("pause-all")
                            .icon(Icon::new(IconName::CirclePause).text_color(theme.primary))
                            .ghost()
                            .small()
                            .tooltip("全部暂停")
                            .on_click(cx.listener(Self::pause_all)),
                    )
                    .child(
                        Button::new("resume-all")
                            .icon(Icon::new(IconName::FastForward).text_color(theme.primary))
                            .ghost()
                            .small()
                            .tooltip("全部开始")
                            .on_click(cx.listener(Self::resume_all)),
                    )
                    .child(
                        Button::new("add-new")
                            .icon(Icon::new(IconName::ClipboardPlus).text_color(theme.primary))
                            .ghost()
                            .small()
                            .tooltip("从剪切板添加")
                            .on_click(cx.listener(Self::add_from_clipboard)),
                    ),
                false => base,
            },
            AppState::License(_) => base,
        }
    }

    fn theme_icon(theme: &Theme) -> IconName {
        match theme.mode {
            ThemeMode::Light => IconName::Sun,
            ThemeMode::Dark => IconName::Moon,
        }
    }

    fn render_title_end(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .justify_end()
            .items_center()
            .w_1_3()
            .px_2()
            .gap_1()
            .when(
                matches!(self.state, AppState::Download) && self.download.read(cx).has_login(),
                |this| {
                    this.child(
                        div()
                            .flex()
                            .items_center()
                            .text_sm()
                            .gap_1()
                            .child(
                                Icon::new(IconName::Download)
                                    .text_color(theme.primary)
                                    .small(),
                            )
                            .child(self.download.read(cx).total_download_speed()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .text_sm()
                            .gap_1()
                            .child(
                                Icon::new(IconName::Upload)
                                    .text_color(theme.primary)
                                    .small(),
                            )
                            .child(self.download.read(cx).total_upload_speed()),
                    )
                },
            )
            .child(
                Button::new("theme-mode")
                    .icon(Icon::new(Self::theme_icon(theme)).text_color(theme.primary))
                    .ghost()
                    .small()
                    .tooltip("主题")
                    .on_click(Self::change_color_mode),
            )
            .child(
                Button::new("license")
                    .icon(Icon::new(IconName::CopyRight).text_color(theme.primary))
                    .ghost()
                    .small()
                    .tooltip("开源协议")
                    .selected(matches!(self.state, AppState::License(_)))
                    .on_click(cx.listener(Self::switch_license)),
            )
            .child(
                Button::new("github")
                    .icon(Icon::new(IconName::Github).text_color(theme.primary))
                    .ghost()
                    .small()
                    .tooltip("打开主页")
                    .on_click(Self::open_home_page),
            )
    }

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
