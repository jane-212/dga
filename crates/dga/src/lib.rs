mod app;
mod download;
mod home;

pub use app::App;

trait LogErr<E> {
    fn log_err(&self);
}

impl<E: 'static + std::fmt::Debug> LogErr<E> for std::result::Result<(), E> {
    fn log_err(&self) {
        if let Err(e) = self {
            eprintln!("{e:?}");
        }
    }
}
