use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("初始化客户端失败")]
    BuildClient,
    #[error("网络错误")]
    Network(#[from] reqwest::Error),
    #[error("CSS选择器初始化错误: {}", _0)]
    Parse(&'static str),
    #[error("运行时初始化错误")]
    Tokio(#[from] tokio::task::JoinError),
    #[error("类型初始化错误")]
    TypeNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
