use error::Error;
use gpui::AsyncWindowContext;
use icons::IconName;
use runtime::RUNTIME;
use std::future::Future;
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
