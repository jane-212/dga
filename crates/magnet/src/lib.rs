mod finder;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::{any::TypeId, cmp::Ordering};

use chrono::{DateTime, Utc};
use error::{Error, Result};
use finder::Finder;
use gpui::SharedString;
use reqwest::Client;
use runtime::RUNTIME;

pub struct Magnet {
    finders: HashMap<TypeId, Arc<dyn Finder>>,
}

impl Magnet {
    pub fn new() -> Result<Self> {
        let client = Self::default_http_client()?;
        let finders = finder::all_finders(client)?;

        Ok(Self { finders })
    }

    fn default_http_client() -> Result<Client> {
        Client::builder()
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_e| Error::BuildClient)
    }

    pub async fn find(&self, key: SharedString) -> Result<Vec<Box<dyn FoundItem>>> {
        let finders = self.finders.values().cloned().collect::<Vec<_>>();
        RUNTIME
            .spawn(async move {
                let mut tasks = Vec::new();
                for finder in finders {
                    let key = key.clone();
                    let task = async move { finder.find(key).await };
                    tasks.push(task);
                }

                let mut items = Vec::new();
                for task in tasks {
                    let new_items = task.await?;
                    items.extend(new_items);
                }

                items.sort_by(|a, b| match b.date().cmp(a.date()) {
                    Ordering::Equal => b.size().cmp(a.size()),
                    ori => ori,
                });
                Ok(items)
            })
            .await?
    }

    pub async fn preview(&self, url: Arc<dyn Previewable>) -> Result<Arc<dyn FoundPreview>> {
        let (id, url) = url.preview_url();
        match self.finders.get(&id) {
            Some(finder) => {
                let finder = finder.clone();
                RUNTIME
                    .spawn(async move { finder.load_preview(url).await })
                    .await?
            }
            None => Err(Error::TypeNotFound),
        }
    }
}

pub trait FoundItem: Send + Sync {
    fn title(&self) -> SharedString;
    fn size(&self) -> &Size;
    fn date(&self) -> &Date;
    fn url(&self) -> Arc<dyn Previewable>;
}

pub trait Previewable: Send + Sync + 'static {
    fn preview_url(&self) -> (TypeId, SharedString);
}

pub trait FoundPreview: Send + Sync {
    fn title(&self) -> SharedString;
    fn size(&self) -> &Size;
    fn date(&self) -> &Date;
    fn magnet(&self) -> SharedString;
    fn images(&self) -> Vec<SharedString>;
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Size {
    size: u32,
    format: SharedString,
}

impl From<&Size> for SharedString {
    fn from(value: &Size) -> Self {
        value.format.clone()
    }
}

impl Size {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            format: Self::to_format(size),
        }
    }

    fn to_format(size: u32) -> SharedString {
        let mut count = 0;
        let mut size = size as f64;
        while size > 1024.0 {
            size /= 1024.0;
            count += 1;
        }

        let signal = match count {
            1 => "MB",
            2 => "GB",
            _ => "KB",
        };

        format!("{:.2}{}", size, signal).into()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    date_time: DateTime<Utc>,
    format: SharedString,
}

impl From<&Date> for SharedString {
    fn from(value: &Date) -> Self {
        value.format.clone()
    }
}

impl Date {
    pub fn new(date_time: DateTime<Utc>) -> Self {
        Self {
            date_time,
            format: Self::to_format(date_time),
        }
    }

    fn to_format(date_time: DateTime<Utc>) -> SharedString {
        date_time.format("%Y-%m-%d %H:%M:%S").to_string().into()
    }
}
