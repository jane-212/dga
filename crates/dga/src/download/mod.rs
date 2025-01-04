use std::{
    sync::{Arc, Weak},
    time::Duration,
};

use chrono::DateTime;
use error::Result;
use gpui::{
    div, list, px, AsyncWindowContext, ClickEvent, EventEmitter, FocusHandle, InteractiveElement,
    IntoElement, ListAlignment, ListState, ParentElement, Pixels, Render, RenderOnce, SharedString,
    Styled, View, ViewContext, VisualContext, WeakView, WindowContext,
};
use icons::IconName;
use qbit_rs::{
    model::{AddTorrentArg, Credential, GetTorrentListArg, State, Torrent, TorrentSource},
    Qbit,
};
use ui::{
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    input::TextInput,
    label::Label,
    notification::Notification,
    prelude::FluentBuilder,
    theme::ActiveTheme,
    ContextModal, Disableable, Icon, Sizable, StyledExt,
};
use url::Url;
use utils::LogErr;

pub struct Download {
    list_state: ListState,
    client: Option<Arc<Qbit>>,
    magnets: Vec<Magnet>,
    total_speed: (SharedString, SharedString),
    host: View<TextInput>,
    username: View<TextInput>,
    password: View<TextInput>,
    popup_check: View<PopupCheck>,
    is_login: bool,
    is_pause: bool,
    focus_handle: FocusHandle,
}

pub enum DownloadEvent {
    PauseAll,
    ResumeAll,
    AddFromClipboard,
    AddNew(SharedString),
}

impl EventEmitter<DownloadEvent> for Download {}

impl Download {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        let download =
            cx.new_view(|cx| {
                let (host, username, password) = utils::read_login_info();
                let host = cx.new_view(|cx| {
                    let mut input = TextInput::new(cx)
                        .placeholder("地址")
                        .appearance(false)
                        .small()
                        .prefix(|cx| {
                            let theme = cx.theme();

                            div()
                                .pl_2()
                                .child(Icon::new(IconName::Globe).text_color(theme.primary).small())
                        });
                    if let Some(host) = host {
                        input.set_text(host, cx);
                    }

                    input
                });
                let username = cx.new_view(|cx| {
                    let mut input = TextInput::new(cx)
                        .placeholder("用户名")
                        .appearance(false)
                        .small()
                        .prefix(|cx| {
                            let theme = cx.theme();
                            div()
                                .pl_2()
                                .child(Icon::new(IconName::User).text_color(theme.primary).small())
                        });
                    if let Some(username) = username {
                        input.set_text(username, cx);
                    }

                    input
                });
                let password = cx.new_view(|cx| {
                    let mut input = TextInput::new(cx)
                        .placeholder("密码")
                        .appearance(false)
                        .small()
                        .prefix(|cx| {
                            let theme = cx.theme();
                            div()
                                .pl_2()
                                .child(Icon::new(IconName::Lock).text_color(theme.primary).small())
                        });
                    input.set_masked(true, cx);
                    if let Some(password) = password {
                        input.set_text(password, cx);
                    }

                    input
                });
                let popup_check = PopupCheck::new(cx);
                cx.subscribe(&popup_check, |this: &mut Self, _delete_check, event, cx| {
                    let CheckEvent::Confirm(hash, delete_file) = event;
                    this.delete_one(hash.clone(), *delete_file, cx);
                })
                .detach();
                cx.on_release(|this, _window_handle, cx| {
                    let host = this.host.read(cx).text();
                    let username = this.username.read(cx).text();
                    let password = this.password.read(cx).text();

                    utils::write_login_info(host, username, password);
                })
                .detach();
                let view = cx.view().downgrade();
                let list_state =
                    ListState::new(1, ListAlignment::Top, px(1000.0), move |ix, cx| match view
                        .upgrade()
                    {
                        Some(view) => view.update(cx, |this, cx| {
                            if ix == 0 {
                                Self::render_banner(cx).into_any_element()
                            } else {
                                let magnet = &this.magnets[ix - 1];
                                Self::render_list_item(magnet, ix - 1, cx).into_any_element()
                            }
                        }),
                        None => div().into_any_element(),
                    });

                Self {
                    list_state,
                    client: None,
                    total_speed: ("0 B/s".into(), "0 B/s".into()),
                    magnets: Vec::new(),
                    popup_check,
                    host,
                    username,
                    password,
                    is_login: false,
                    is_pause: false,
                    focus_handle: cx.focus_handle(),
                }
            });
        cx.subscribe(&download, Self::handle_event).detach();

        download
    }

    pub fn total_download_speed(&self) -> SharedString {
        self.total_speed.0.clone()
    }

    pub fn total_upload_speed(&self) -> SharedString {
        self.total_speed.1.clone()
    }

    fn handle_event(this: View<Self>, event: &DownloadEvent, cx: &mut WindowContext) {
        match event {
            DownloadEvent::PauseAll => cx
                .spawn(|cx| Self::pause_all(this.downgrade(), cx))
                .detach(),
            DownloadEvent::ResumeAll => cx
                .spawn(|cx| Self::resume_all(this.downgrade(), cx))
                .detach(),
            DownloadEvent::AddFromClipboard => {
                this.update(cx, |this, cx| {
                    this.add_from_clipboard(cx);
                });
            }
            DownloadEvent::AddNew(new) => {
                this.update(cx, |this, cx| {
                    this.add_new(new.clone(), cx);
                });
            }
        }
    }

    async fn resume_all(this: WeakView<Self>, mut cx: AsyncWindowContext) {
        if let Some(this) = this.upgrade() {
            let Some(client) = cx
                .update(|cx| this.read(cx).client.clone())
                .ok()
                .and_then(|inner| inner)
            else {
                return;
            };
            let Ok(hashes) = cx.update(|cx| {
                this.read(cx)
                    .magnets
                    .iter()
                    .flat_map(|magnet| magnet.hash.as_ref().map(ToString::to_string))
                    .collect::<Vec<_>>()
            }) else {
                return;
            };

            utils::handle_qbit_operation(
                || async move {
                    client.resume_torrents(hashes).await?;
                    Ok(())
                },
                "已继续",
                &mut cx,
            )
            .await;
        }
    }

    async fn pause_all(this: WeakView<Self>, mut cx: AsyncWindowContext) {
        if let Some(this) = this.upgrade() {
            let Some(client) = cx
                .update(|cx| this.read(cx).client.clone())
                .ok()
                .and_then(|inner| inner)
            else {
                return;
            };
            let Ok(hashes) = cx.update(|cx| {
                this.read(cx)
                    .magnets
                    .iter()
                    .flat_map(|magnet| magnet.hash.as_ref().map(ToString::to_string))
                    .collect::<Vec<_>>()
            }) else {
                return;
            };

            utils::handle_qbit_operation(
                || async move {
                    client.pause_torrents(hashes).await?;
                    Ok(())
                },
                "已暂停",
                &mut cx,
            )
            .await;
        }
    }

    pub fn pause(&mut self, cx: &mut ViewContext<Self>) {
        self.is_pause = true;
        cx.notify();
    }

    pub fn resume(&mut self, cx: &mut ViewContext<Self>) {
        self.is_pause = false;
        cx.notify();
    }

    pub fn logout(&mut self, cx: &mut ViewContext<Self>) {
        self.client = None;
        self.magnets.clear();
        self.list_state.reset(0);
        self.total_speed = ("0 B/s".into(), "0 B/s".into());
        cx.notify();
    }

    pub fn has_login(&self) -> bool {
        self.client.is_some()
    }

    fn render_login(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .rounded_lg()
            .shadow_sm()
            .bg(theme.secondary)
            .p_4()
            .child(
                div()
                    .w_72()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(self.host.clone()),
            )
            .child(
                div()
                    .mt_4()
                    .w_72()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(self.username.clone()),
            )
            .child(
                div()
                    .mt_4()
                    .w_72()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(self.password.clone()),
            )
            .child(
                div().flex().justify_center().mt_4().child(
                    Button::new("login")
                        .label("登录")
                        .w_32()
                        .font_bold()
                        .text_color(theme.primary)
                        .disabled(self.is_login)
                        .loading(self.is_login)
                        .on_click(cx.listener(Self::handle_login)),
                ),
            )
    }

    fn handle_login(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        self.is_login = true;
        cx.notify();
        cx.focus(&self.focus_handle);

        self.login(cx);
    }

    async fn login_task(client: Arc<Qbit>) -> Result<String> {
        let version = utils::handle_tokio_spawn(|| async move {
            let version = client.get_version().await?;

            Ok(version)
        })
        .await?;

        Ok(version)
    }

    async fn get_and_update(
        this: View<Self>,
        client: Arc<Qbit>,
        mut cx: AsyncWindowContext,
    ) -> Result<()> {
        let list = utils::handle_tokio_spawn(|| async move {
            let list = client
                .get_torrent_list(
                    GetTorrentListArg::builder()
                        .sort("dlspeed".to_string())
                        .reverse(true)
                        .build(),
                )
                .await?;

            Ok(list)
        })
        .await?;
        let magnets = list.into_iter().map(Magnet::from).collect::<Vec<_>>();
        let total_download_speed = magnets.iter().map(|magnet| magnet.total.0).sum::<i64>();
        let total_download_speed = human_read_speed(total_download_speed);
        let total_upload_speed = magnets.iter().map(|magnet| magnet.total.1).sum::<i64>();
        let total_upload_speed = human_read_speed(total_upload_speed);
        cx.update(|cx| {
            this.update(cx, |this, cx| {
                this.total_speed = (total_download_speed, total_upload_speed);
                this.magnets = magnets;
                let offset = this.list_state.logical_scroll_top();
                this.list_state.reset(this.magnets.len() + 1);
                this.list_state.scroll_to(offset);
                cx.notify();
            })
        })?;

        Ok(())
    }

    fn start_update(this: WeakView<Self>, client: Weak<Qbit>, cx: AsyncWindowContext) {
        cx.spawn({
            let this = this.clone();
            let client = client.clone();
            |mut cx| async move {
                loop {
                    match (this.upgrade(), client.upgrade()) {
                        (Some(this), Some(client)) => {
                            let Ok(is_pause) = cx.update(|cx| this.read(cx).is_pause) else {
                                break;
                            };

                            if !is_pause {
                                Self::get_and_update(this, client, cx.clone())
                                    .await
                                    .log_err();
                            }
                        }
                        _ => break,
                    }

                    cx.background_executor()
                        .timer(Duration::from_millis(1_500))
                        .await;
                }
            }
        })
        .detach();
    }

    fn login(&mut self, cx: &mut ViewContext<Self>) {
        let host = self.host.read(cx).text();
        let host = match Url::parse(&host) {
            Ok(host) => host,
            Err(e) => {
                self.is_login = false;
                cx.push_notification(
                    Notification::new(format!("请填写完整url: {e}")).icon(IconName::CircleX),
                );
                return;
            }
        };
        let username = self.username.read(cx).text();
        let password = self.password.read(cx).text();

        let credential = Credential::new(username, password);
        let client = Qbit::new(host, credential);
        let client = Arc::new(client);

        let client_clone = client.clone();
        let task = cx
            .background_executor()
            .spawn(async { Self::login_task(client_clone).await });

        cx.spawn(|this, mut async_cx| async move {
            match task.await {
                Ok(v) => {
                    let client_weak = Arc::downgrade(&client);
                    let res = async_cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            this.client = Some(client.clone());
                            this.is_login = false;
                            cx.notify();
                        })
                        .log_err();
                        cx.push_notification(
                            Notification::new(format!("Qbittorrent版本: {v}")).icon(IconName::Info),
                        )
                    });
                    Self::start_update(this, client_weak, async_cx);

                    res
                }
                Err(e) => async_cx.update(|cx| {
                    this.update(cx, |this, cx| {
                        this.is_login = false;
                        cx.notify();
                    })
                    .log_err();
                    cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX))
                }),
            }
            .log_err();
        })
        .detach();
    }

    fn render_banner(cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let width = cx.viewport_size().width - Self::OTHER_WIDTH;

        div()
            .flex()
            .py_2()
            .bg(theme.secondary)
            .font_bold()
            .child(Label::new("").text_center().w_8())
            .child(Label::new("名称").text_color(theme.primary).pr_1().w(width))
            .child(
                Label::new("状态")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w_24(),
            )
            .child(
                Label::new("操作")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w_24(),
            )
            .child(
                Label::new("进度")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w_20(),
            )
            .child(
                Label::new("大小")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w_24(),
            )
            .child(
                Label::new("下载")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w(px(105.0)),
            )
            .child(
                Label::new("上传")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w(px(105.0)),
            )
            .child(
                Label::new("比率")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w_16(),
            )
            .child(
                Label::new("添加日期")
                    .text_color(theme.primary)
                    .pl_1()
                    .pr_1()
                    .w(px(165.0)),
            )
    }

    fn render_list_item(
        magnet: &Magnet,
        ix: usize,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let width = cx.viewport_size().width - Self::OTHER_WIDTH;

        div()
            .flex()
            .items_center()
            .py_2()
            .h_12()
            .overflow_hidden()
            .bg(theme.secondary)
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .justify_center()
                    .child(Icon::new(magnet.icon()).text_color(theme.primary))
                    .w_8(),
            )
            .child(Label::new(magnet.name.clone()).pr_1().w(width))
            .child(Label::new(magnet.state.clone()).pl_1().pr_1().w_24())
            .child(
                div()
                    .flex()
                    .px_1()
                    .gap_1()
                    .child(
                        Button::new(("pause", ix))
                            .icon(Icon::new(IconName::CirclePause).text_color(theme.primary))
                            .small()
                            .tooltip("暂停")
                            .on_click(cx.listener({
                                let hash = magnet.hash.clone();
                                move |this, event, cx| {
                                    this.pause_one(hash.clone(), event, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(("resume", ix))
                            .icon(Icon::new(IconName::FastForward).text_color(theme.primary))
                            .small()
                            .tooltip("开始")
                            .on_click(cx.listener({
                                let hash = magnet.hash.clone();
                                move |this, event, cx| {
                                    this.resume_one(hash.clone(), event, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(("delete", ix))
                            .icon(Icon::new(IconName::Trash2).text_color(ui::red_400()))
                            .small()
                            .tooltip("删除")
                            .on_click(cx.listener({
                                let hash = magnet.hash.clone();
                                let name = magnet.name.clone();
                                move |this, event, cx| {
                                    this.delete_check(name.clone(), hash.clone(), event, cx);
                                }
                            })),
                    )
                    .w_24(),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(Progress::new(magnet.progress))
                    .px_1()
                    .w_20(),
            )
            .child(Label::new(magnet.size.clone()).px_1().w_24())
            .child(Label::new(magnet.download.clone()).px_1().w(px(105.0)))
            .child(Label::new(magnet.upload.clone()).px_1().w(px(105.0)))
            .child(Label::new(magnet.ratio.clone()).px_1().w_16())
            .child(Label::new(magnet.add_on.clone()).px_1().w(px(165.0)))
    }

    const OTHER_WIDTH: Pixels = px(32. + 96. * 3. + 80. + 64. + 165. + 105. * 2.);

    fn render_list(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(list(self.list_state.clone()).size_full())
    }

    fn pause_one(
        &mut self,
        hash: Option<SharedString>,
        _event: &ClickEvent,
        cx: &mut ViewContext<Self>,
    ) {
        let Some(hash) = hash else {
            return;
        };
        let Some(ref client) = self.client else {
            return;
        };
        let client = client.clone();
        cx.spawn(|_this, mut cx| async move {
            utils::handle_qbit_operation(
                || async move {
                    client.pause_torrents(&[hash.to_string()]).await?;
                    Ok(())
                },
                "已暂停",
                &mut cx,
            )
            .await;
        })
        .detach();
    }

    fn delete_check(
        &mut self,
        name: SharedString,
        hash: Option<SharedString>,
        _event: &ClickEvent,
        cx: &mut ViewContext<Self>,
    ) {
        self.popup_check.update(cx, |this, cx| {
            this.check_delete(name, hash);
            cx.notify();
        });
    }

    fn add_torrent(&mut self, new: String, cx: &mut ViewContext<Self>) {
        let Some(ref client) = self.client else {
            cx.push_notification(Notification::new("请先登录").icon(IconName::Info));
            return;
        };
        let client = client.clone();
        cx.spawn(|_this, mut cx| async move {
            match new.parse() {
                Ok(urls) => {
                    let arg = AddTorrentArg::builder()
                        .source(TorrentSource::Urls { urls })
                        .build();

                    utils::handle_qbit_operation(
                        || async move {
                            client.add_torrent(arg).await?;
                            Ok(())
                        },
                        "添加成功",
                        &mut cx,
                    )
                    .await;
                }
                Err(e) => cx
                    .update(|cx| {
                        cx.push_notification(
                            Notification::new(e.to_string()).icon(IconName::CircleX),
                        );
                    })
                    .log_err(),
            }
        })
        .detach();
    }

    fn add_new(&mut self, new: SharedString, cx: &mut ViewContext<Self>) {
        self.add_torrent(new.to_string(), cx);
    }

    fn add_from_clipboard(&mut self, cx: &mut ViewContext<Self>) {
        let Some(new) = cx
            .read_from_clipboard()
            .and_then(|clipboard| clipboard.text())
        else {
            return;
        };
        self.add_torrent(new, cx);
    }

    fn delete_one(
        &mut self,
        hash: Option<SharedString>,
        delete_file: bool,
        cx: &mut ViewContext<Self>,
    ) {
        let Some(hash) = hash else {
            return;
        };
        let Some(ref client) = self.client else {
            return;
        };
        let client = client.clone();
        cx.spawn(|_this, mut cx| async move {
            utils::handle_qbit_operation(
                || async move {
                    client
                        .delete_torrents(&[hash.to_string()], delete_file)
                        .await?;
                    Ok(())
                },
                "已删除",
                &mut cx,
            )
            .await;
        })
        .detach();
    }

    fn resume_one(
        &mut self,
        hash: Option<SharedString>,
        _event: &ClickEvent,
        cx: &mut ViewContext<Self>,
    ) {
        let Some(hash) = hash else {
            return;
        };
        let Some(ref client) = self.client else {
            return;
        };
        let client = client.clone();
        cx.spawn(|_this, mut cx| async move {
            utils::handle_qbit_operation(
                || async move {
                    client.resume_torrents(&[hash.to_string()]).await?;
                    Ok(())
                },
                "已继续",
                &mut cx,
            )
            .await;
        })
        .detach();
    }

    fn render_main(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let base = div().size_full().flex().justify_center().items_center();

        match self.client {
            Some(_) => base.child(self.render_list(cx)),
            None => base.child(self.render_login(cx)),
        }
        .child(self.popup_check.clone())
    }
}

impl Render for Download {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.render_main(cx)
    }
}

struct Magnet {
    total: (i64, i64),
    hash: Option<SharedString>,
    icon_state: State,
    state: SharedString,
    add_on: SharedString,
    name: SharedString,
    progress: f32,
    size: SharedString,
    ratio: SharedString,
    download: SharedString,
    upload: SharedString,
}

impl Magnet {
    fn map_state(state: &State) -> SharedString {
        match state {
            State::Error => "失败",
            State::MissingFiles => "缺少文件",
            State::Uploading => "上传",
            State::PausedUP => "完成",
            State::QueuedUP => "等待上传",
            State::StalledUP => "等待",
            State::CheckingUP => "正在检查",
            State::ForcedUP => "强制上传",
            State::Allocating => "预留空间",
            State::Downloading => "正在下载",
            State::MetaDL => "下载元数据",
            State::PausedDL => "暂停",
            State::QueuedDL => "等待",
            State::StalledDL => "等待",
            State::CheckingDL => "正在检查",
            State::ForcedDL => "强制下载",
            State::CheckingResumeData => "校验元数据",
            State::Moving => "文件被移动",
            State::Unknown => "未知",
        }
        .into()
    }

    fn icon(&self) -> IconName {
        match self.icon_state {
            State::Error => IconName::CircleX,
            State::MissingFiles => IconName::FileX,
            State::Uploading => IconName::Upload,
            State::PausedUP => IconName::CircleCheck,
            State::QueuedUP => IconName::ListStart,
            State::StalledUP => IconName::ListStart,
            State::CheckingUP => IconName::FileClock,
            State::ForcedUP => IconName::Upload,
            State::Allocating => IconName::HardDrive,
            State::Downloading => IconName::Download,
            State::MetaDL => IconName::FileCog,
            State::PausedDL => IconName::CirclePause,
            State::QueuedDL => IconName::ListEnd,
            State::StalledDL => IconName::ListEnd,
            State::CheckingDL => IconName::FileClock,
            State::ForcedDL => IconName::Download,
            State::CheckingResumeData => IconName::FileSearch,
            State::Moving => IconName::FileOutput,
            State::Unknown => IconName::CircleHelp,
        }
    }
}

impl From<Torrent> for Magnet {
    fn from(value: Torrent) -> Self {
        const UNKNOWN: SharedString = SharedString::new_static("unknown");
        let state = value.state.as_ref().map(Self::map_state).unwrap_or(UNKNOWN);
        let icon_state = value.state.unwrap_or(State::Unknown);
        let add_on = value.added_on.map(format_timestamp).unwrap_or(UNKNOWN);
        let name = value.name.map(SharedString::from).unwrap_or(UNKNOWN);
        let size = value.size.map(human_read_size).unwrap_or(UNKNOWN);
        let ratio = value
            .ratio
            .map(|ratio| format!("{ratio:.2}").into())
            .unwrap_or(UNKNOWN);
        let download = value.dlspeed.map(human_read_speed).unwrap_or(UNKNOWN);
        let upload = value.upspeed.map(human_read_speed).unwrap_or(UNKNOWN);

        Self {
            total: (
                value.dlspeed.unwrap_or_default(),
                value.upspeed.unwrap_or_default(),
            ),
            hash: value.hash.map(SharedString::from),
            icon_state,
            state,
            add_on,
            name,
            progress: value.progress.unwrap_or_default() as f32,
            size,
            ratio,
            download,
            upload,
        }
    }
}

#[derive(IntoElement)]
struct Progress {
    value: f32,
    height: Pixels,
    width: Pixels,
}

impl Progress {
    fn new(value: f32) -> Self {
        Self {
            value,
            height: px(10.0),
            width: px(72.),
        }
    }
}

impl RenderOnce for Progress {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();
        let width = self.value.clamp(0., 1.) * self.width;

        div()
            .h(self.height)
            .w(self.width)
            .rounded_sm()
            .bg(theme.progress_bar.opacity(0.2))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .h_full()
                    .w(width)
                    .rounded_sm()
                    .bg(theme.progress_bar),
            )
    }
}

struct PopupCheck {
    hash: Option<SharedString>,
    name: Option<SharedString>,
    delete_file: bool,
}

impl PopupCheck {
    fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self {
            hash: None,
            name: None,
            delete_file: false,
        })
    }

    fn check_delete(&mut self, name: SharedString, hash: Option<SharedString>) {
        self.name = Some(name);
        self.hash = hash;
    }

    fn confirm_delete(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        let hash = self.hash.take();
        self.name = None;
        let delete_file = self.delete_file;
        self.delete_file = false;
        cx.emit(CheckEvent::Confirm(hash, delete_file));
    }

    fn cancel(&mut self, _event: &ClickEvent, _cx: &mut ViewContext<Self>) {
        self.hash = None;
        self.name = None;
        self.delete_file = false;
    }

    fn toggle_check(&mut self, check: &bool, cx: &mut ViewContext<Self>) {
        self.delete_file = *check;
        cx.notify();
    }
}

impl Render for PopupCheck {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .absolute()
            .top_0()
            .left_0()
            .when_some(self.name.clone(), |this, name| {
                this.size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(theme.background.opacity(0.95))
                    .child(
                        div()
                            .bg(theme.secondary)
                            .rounded_lg()
                            .shadow_sm()
                            .border_1()
                            .border_color(theme.border)
                            .w_64()
                            .p_4()
                            .child("是否删除")
                            .child(name)
                            .child(
                                div().py_4().pl_2().child(
                                    Checkbox::new("delete-file")
                                        .label("同时删除文件")
                                        .checked(self.delete_file)
                                        .on_click(cx.listener(Self::toggle_check)),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .child(
                                        Button::new("cancel")
                                            .icon(IconName::CircleX)
                                            .label("取消")
                                            .on_click(cx.listener(Self::cancel)),
                                    )
                                    .child(
                                        Button::new("check")
                                            .icon(IconName::Check)
                                            .label("确认")
                                            .primary()
                                            .on_click(cx.listener(Self::confirm_delete)),
                                    ),
                            ),
                    )
            })
            .on_any_mouse_down(|_event, cx| {
                cx.stop_propagation();
            })
            .on_mouse_move(|_event, cx| {
                cx.stop_propagation();
            })
            .on_scroll_wheel(|_event, cx| {
                cx.stop_propagation();
            })
    }
}

enum CheckEvent {
    Confirm(Option<SharedString>, bool),
}

impl EventEmitter<CheckEvent> for PopupCheck {}

fn size_to_string(size: i64) -> String {
    let mut count = 0;
    let mut size = size as f64;
    while size >= 1024.0 {
        size /= 1024.0;
        count += 1;
    }

    let signal = match count {
        0 => "B",
        1 => "KB",
        2 => "MB",
        3 => "GB",
        4 => "TB",
        _ => "PB",
    };

    if size < 0.01 {
        format!("0 {signal}")
    } else {
        format!("{size:.2} {signal}")
    }
}

fn human_read_size(size: i64) -> SharedString {
    size_to_string(size).into()
}

fn human_read_speed(size: i64) -> SharedString {
    let size = size_to_string(size);

    format!("{size}/s").into()
}

fn format_timestamp(stamp: i64) -> SharedString {
    DateTime::from_timestamp(stamp, 0)
        .unwrap_or_default()
        .naive_local()
        .format("%Y/%m/%d %H:%M:%S")
        .to_string()
        .into()
}
