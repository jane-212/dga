use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("初始化客户端失败")]
    BuildClient,
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),
    #[error("CSS选择器初始化错误: {0}")]
    Parse(&'static str),
    #[error("IO错误: {0}")]
    Tokio(#[from] tokio::task::JoinError),
    #[error("类型初始化错误")]
    TypeNotFound,
    #[error("Qbit错误: {0}")]
    QbitError(#[from] qbit_rs::Error),
    #[error("未知错误: {0}")]
    AnyError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
