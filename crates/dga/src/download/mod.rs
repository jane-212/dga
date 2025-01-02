use std::{
    sync::{Arc, Weak},
    time::Duration,
};

use chrono::DateTime;
use error::Result;
use gpui::{
    div, px, AsyncWindowContext, ClickEvent, Entity, EventEmitter, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, Pixels, Render, RenderOnce, SharedString, Styled, View,
    ViewContext, VisualContext, WeakView, WindowContext,
};
use icons::IconName;
use qbit_rs::{
    model::{AddTorrentArg, Credential, GetTorrentListArg, State, Torrent, TorrentSource},
    Qbit,
};
use runtime::RUNTIME;
use ui::{
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    input::TextInput,
    label::Label,
    notification::Notification,
    prelude::FluentBuilder,
    scroll::ScrollbarAxis,
    theme::ActiveTheme,
    ContextModal, Disableable, Icon, Sizable, StyledExt,
};
use url::Url;

use crate::LogErr;

pub struct Download {
    client: Option<Arc<Qbit>>,
    magnets: Vec<Magnet>,
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
        let download = cx.new_view(|cx| {
            let host = cx.new_view(|cx| {
                TextInput::new(cx)
                    .placeholder("地址")
                    .appearance(false)
                    .small()
                    .prefix(|_cx| div().pl_2().child(Icon::new(IconName::Globe).small()))
            });
            let username = cx.new_view(|cx| {
                TextInput::new(cx)
                    .placeholder("用户名")
                    .appearance(false)
                    .small()
                    .prefix(|_cx| div().pl_2().child(Icon::new(IconName::User).small()))
            });
            let password = cx.new_view(|cx| {
                let mut input = TextInput::new(cx)
                    .placeholder("密码")
                    .appearance(false)
                    .small()
                    .prefix(|_cx| div().pl_2().child(Icon::new(IconName::Lock).small()));
                input.set_masked(true, cx);

                input
            });
            let popup_check = PopupCheck::new(cx);
            cx.subscribe(&popup_check, |this: &mut Self, _delete_check, event, cx| {
                let CheckEvent::Confirm(hash, delete_file) = event;
                this.delete_one(hash.clone(), *delete_file, cx);
            })
            .detach();

            Self {
                client: None,
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

            match client.resume_torrents(hashes).await {
                Ok(_) => cx.update(|cx| {
                    cx.push_notification(Notification::new("已继续").icon(IconName::Info));
                }),
                Err(e) => cx.update(|cx| {
                    cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
                }),
            }
            .log_err();
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

            match client.pause_torrents(hashes).await {
                Ok(_) => cx.update(|cx| {
                    cx.push_notification(Notification::new("已暂停").icon(IconName::Info));
                }),
                Err(e) => cx.update(|cx| {
                    cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
                }),
            }
            .log_err();
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
        cx.notify();
    }

    pub fn has_login(&self) -> bool {
        self.client.is_some()
    }

    #[inline]
    fn render_login(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .rounded_lg()
            .shadow_sm()
            .bg(theme.secondary)
            .p_2()
            .child(
                div()
                    .w(px(200.0))
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(self.host.clone()),
            )
            .child(
                div()
                    .mt_2()
                    .w(px(200.0))
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(self.username.clone()),
            )
            .child(
                div()
                    .mt_2()
                    .w(px(200.0))
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .child(self.password.clone()),
            )
            .child(
                Button::new("login")
                    .label("登录")
                    .mt_2()
                    .disabled(self.is_login)
                    .loading(self.is_login)
                    .on_click(cx.listener(Self::handle_login)),
            )
    }

    #[inline]
    fn handle_login(&mut self, _event: &ClickEvent, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        self.is_login = true;
        cx.notify();
        cx.focus(&self.focus_handle);

        self.login(cx);
    }

    async fn login_task(client: Arc<Qbit>) -> Result<String> {
        let version = RUNTIME
            .spawn(async move { client.get_version().await })
            .await??;

        Ok(version)
    }

    fn update_magnets(&mut self, magnets: Vec<Torrent>) {
        self.magnets = magnets.into_iter().map(Magnet::from).collect();
    }

    async fn get_and_update(
        this: View<Self>,
        client: Arc<Qbit>,
        mut cx: AsyncWindowContext,
    ) -> Result<()> {
        let list = RUNTIME
            .spawn(async move {
                client
                    .get_torrent_list(
                        GetTorrentListArg::builder()
                            .sort("dlspeed".to_string())
                            .reverse(true)
                            .build(),
                    )
                    .await
            })
            .await??;
        cx.update(|cx| {
            this.update(cx, |this, cx| {
                this.update_magnets(list);
                cx.notify();
            })
        })?;

        Ok(())
    }

    fn start_update(this: WeakView<Self>, client: Weak<Qbit>, cx: AsyncWindowContext) {
        cx.spawn(|mut cx| async move {
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

                cx.background_executor().timer(Duration::from_secs(1)).await;
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

    const OTHER_WIDTH: Pixels = px(8. * 2. + 4. + 32. + 96. * 3. + 80. + 64. + 165. + 105. * 2.);

    #[inline]
    fn render_list(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let width = cx.viewport_size().width - Self::OTHER_WIDTH;

        div()
            .p_2()
            .scrollable(cx.view().entity_id(), ScrollbarAxis::Vertical)
            .child(
                div()
                    .flex()
                    .rounded_md()
                    .shadow_sm()
                    .mr_1()
                    .py_2()
                    .bg(theme.secondary)
                    .child(Label::new("").text_center().w_8())
                    .child(Label::new("名称").pr_1().w(width))
                    .child(Label::new("状态").pl_1().pr_1().w_24())
                    .child(Label::new("操作").pl_1().pr_1().w_24())
                    .child(Label::new("进度").pl_1().pr_1().w_20())
                    .child(Label::new("大小").pl_1().pr_1().w_24())
                    .child(Label::new("下载").pl_1().pr_1().w(px(105.0)))
                    .child(Label::new("上传").pl_1().pr_1().w(px(105.0)))
                    .child(Label::new("比率").pl_1().pr_1().w_16())
                    .child(Label::new("添加日期").pl_1().pr_1().w(px(165.0))),
            )
            .children(self.magnets.iter().enumerate().map(|(idx, magnet)| {
                div()
                    .flex()
                    .items_center()
                    .rounded_md()
                    .shadow_sm()
                    .mt_1()
                    .mr_1()
                    .py_2()
                    .bg(theme.secondary)
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .child(Icon::new(magnet.icon()).small())
                            .w_8(),
                    )
                    .child(
                        Label::new(magnet.name.clone())
                            .text_ellipsis()
                            .pr_1()
                            .w(width),
                    )
                    .child(Label::new(magnet.state.clone()).pl_1().pr_1().w_24())
                    .child(
                        div()
                            .flex()
                            .px_1()
                            .gap_1()
                            .child(
                                Button::new(("pause", idx))
                                    .icon(IconName::CirclePause)
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
                                Button::new(("resume", idx))
                                    .icon(IconName::FastForward)
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
                                Button::new(("delete", idx))
                                    .icon(IconName::Trash2)
                                    .small()
                                    .tooltip("删除")
                                    .on_click(cx.listener({
                                        let hash = magnet.hash.clone();
                                        let name = magnet.name.clone();
                                        move |this, event, cx| {
                                            this.delete_check(
                                                name.clone(),
                                                hash.clone(),
                                                event,
                                                cx,
                                            );
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
            }))
    }

    #[inline]
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
            match client.pause_torrents(&[hash.to_string()]).await {
                Ok(_) => cx.update(|cx| {
                    cx.push_notification(Notification::new("已暂停").icon(IconName::Info));
                }),
                Err(e) => cx.update(|cx| {
                    cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
                }),
            }
            .log_err();
        })
        .detach();
    }

    #[inline]
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

    #[inline]
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
                    match client.add_torrent(arg).await {
                        Ok(_) => cx.update(|cx| {
                            cx.push_notification(
                                Notification::new("添加成功").icon(IconName::Info),
                            );
                        }),
                        Err(e) => cx.update(|cx| {
                            cx.push_notification(
                                Notification::new(e.to_string()).icon(IconName::CircleX),
                            );
                        }),
                    }
                    .log_err();
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

    #[inline]
    fn add_new(&mut self, new: SharedString, cx: &mut ViewContext<Self>) {
        self.add_torrent(new.to_string(), cx);
    }

    #[inline]
    fn add_from_clipboard(&mut self, cx: &mut ViewContext<Self>) {
        let Some(new) = cx
            .read_from_clipboard()
            .and_then(|clipboard| clipboard.text())
        else {
            return;
        };
        self.add_torrent(new, cx);
    }

    #[inline]
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
            match client
                .delete_torrents(&[hash.to_string()], delete_file)
                .await
            {
                Ok(_) => cx.update(|cx| {
                    cx.push_notification(Notification::new("已删除").icon(IconName::Info));
                }),
                Err(e) => cx.update(|cx| {
                    cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
                }),
            }
            .log_err();
        })
        .detach();
    }

    #[inline]
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
            match client.resume_torrents(&[hash.to_string()]).await {
                Ok(_) => cx.update(|cx| {
                    cx.push_notification(Notification::new("已继续").icon(IconName::Info));
                }),
                Err(e) => cx.update(|cx| {
                    cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
                }),
            }
            .log_err();
        })
        .detach();
    }

    #[inline]
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
    fn format_timestamp(stamp: i64) -> SharedString {
        DateTime::from_timestamp(stamp, 0)
            .unwrap_or_default()
            .format("%Y/%m/%d %H:%M:%S")
            .to_string()
            .into()
    }

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
        Self::size_to_string(size).into()
    }

    fn human_read_speed(size: i64) -> SharedString {
        let size = Self::size_to_string(size);

        format!("{size}/s").into()
    }

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
        let add_on = value
            .added_on
            .map(Self::format_timestamp)
            .unwrap_or(UNKNOWN);
        let name = value.name.map(SharedString::from).unwrap_or(UNKNOWN);
        let size = value.size.map(Self::human_read_size).unwrap_or(UNKNOWN);
        let ratio = value
            .ratio
            .map(|ratio| format!("{ratio:.2}").into())
            .unwrap_or(UNKNOWN);
        let download = value.dlspeed.map(Self::human_read_speed).unwrap_or(UNKNOWN);
        let upload = value.upspeed.map(Self::human_read_speed).unwrap_or(UNKNOWN);

        Self {
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
