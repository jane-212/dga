use std::fs;
use std::future::Future;
use std::path::PathBuf;

use error::Error;
use gpui::{px, size, AsyncWindowContext, Pixels, Size};
use icons::IconName;
use runtime::RUNTIME;
use ui::{notification::Notification, ContextModal};

pub async fn handle_qbit_operation<F, Fut>(
    operation: F,
    success_msg: &'static str,
    cx: &mut AsyncWindowContext,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), Error>> + Send + 'static,
{
    let res = RUNTIME.spawn(operation()).await;
    match res {
        Ok(res) => match res {
            Ok(_) => cx.update(|cx| {
                cx.push_notification(Notification::new(success_msg).icon(IconName::Info));
            }),
            Err(e) => cx.update(|cx| {
                cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
            }),
        },
        Err(e) => cx.update(|cx| {
            cx.push_notification(Notification::new(e.to_string()).icon(IconName::CircleX));
        }),
    }
    .log_err();
}

pub async fn handle_tokio_spawn<F, Fut, R>(spawn: F) -> Result<R, Error>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<R, Error>> + Send + 'static,
    R: Send + 'static,
{
    let res = RUNTIME.spawn(spawn()).await??;

    Ok(res)
}

pub trait LogErr<E> {
    fn log_err(&self);
}

impl<E: 'static + std::fmt::Debug> LogErr<E> for std::result::Result<(), E> {
    fn log_err(&self) {
        if let Err(e) = self {
            eprintln!("{e:?}");
        }
    }
}

fn data_dir() -> PathBuf {
    let username = whoami::username();
    #[cfg(target_os = "macos")]
    let user_dir = PathBuf::from("/Users").join(username);
    #[cfg(target_os = "linux")]
    let user_dir = PathBuf::from("/home").join(username);
    #[cfg(target_os = "windows")]
    let user_dir = PathBuf::from("C:\\Users").join(username);

    user_dir.join(".cache").join("github.jane-212.dga")
}

pub fn read_window() -> Option<Size<Pixels>> {
    let window_file = data_dir().join("window");
    match fs::read_to_string(window_file) {
        Ok(content) => {
            let mut lines = content.lines();
            let width = lines.next().and_then(|line| line.parse().ok());
            let height = lines.next().and_then(|line| line.parse().ok());

            match (width, height) {
                (Some(width), Some(height)) => Some(size(px(width), px(height))),
                _ => None,
            }
        }
        Err(_) => None,
    }
}

pub fn write_window(width: f32, height: f32) {
    let data_dir = data_dir();
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).log_err();
    }
    fs::write(data_dir.join("window"), format!("{width}\n{height}\n")).log_err();
}
