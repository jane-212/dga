mod finder;

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use error::{Error, Result};
use finder::Finder;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use gpui::SharedString;
use reqwest::Client;
use runtime::RUNTIME;

pub struct Magnet {
    matcher: SkimMatcherV2,
    finders: HashMap<TypeId, Arc<dyn Finder>>,
}

impl Magnet {
    pub fn new() -> Result<Self> {
        let client = Self::default_http_client()?;
        let finders = finder::all_finders(client)?;
        let matcher = SkimMatcherV2::default().smart_case();

        Ok(Self { matcher, finders })
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
        let mut tasks = Vec::new();
        for finder in finders {
            let key = key.clone();
            let task = RUNTIME.spawn(async move { finder.find(key).await });
            tasks.push(task);
        }

        let mut items = Vec::new();
        for task in tasks {
            let new_items = task.await??;
            items.extend(new_items);
        }

        items.sort_by_key(|item| {
            self.matcher
                .fuzzy_match(&item.title(), &key)
                .unwrap_or_default()
        });
        items.reverse();
        Ok(items)
    }

    pub async fn preview(&self, url: Arc<dyn Previewable>) -> Result<Box<dyn FoundPreview>> {
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
    fn url(&self) -> Arc<dyn Previewable>;
    fn first(&self) -> SharedString;
    fn last(&self) -> SharedString;
}

pub trait Previewable: Send + Sync + 'static {
    fn preview_url(&self) -> (TypeId, SharedString);
}

pub trait FoundPreview: Send + Sync {
    fn title(&self) -> SharedString;
    fn bounds(&self) -> Vec<Arc<dyn Bound>>;
    fn images(&self) -> Vec<SharedString>;
}

pub trait Bound: Send + Sync {
    fn size(&self) -> &Size;
    fn date(&self) -> &Date;
    fn magnet(&self) -> SharedString;
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Size {
    size: u64,
    format: SharedString,
}

impl From<&Size> for SharedString {
    fn from(value: &Size) -> Self {
        value.format.clone()
    }
}

impl Size {
    pub fn new(size: u64) -> Self {
        Self {
            size,
            format: Self::to_format(size),
        }
    }

    fn to_format(size: u64) -> SharedString {
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

        format!("{:.2} {}", size, signal).into()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    date_time: DateTime<Local>,
    format: SharedString,
}

impl From<&Date> for SharedString {
    fn from(value: &Date) -> Self {
        value.format.clone()
    }
}

impl Date {
    fn new(date_time: DateTime<Local>) -> Self {
        Self {
            date_time,
            format: Self::to_format(date_time),
        }
    }

    fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .map(|date| date.and_time(NaiveTime::MIN))
            .map(Self::convert_date_time_to_local)
            .unwrap_or_default();

        Self::new(date)
    }

    fn parse_date(date: impl AsRef<str>, format: impl AsRef<str>) -> Self {
        let date = date.as_ref();
        let format = format.as_ref();

        let date = NaiveDate::parse_from_str(date, format)
            .map(|date| date.and_time(NaiveTime::MIN))
            .map(Self::convert_date_time_to_local)
            .unwrap_or_default();

        Self::new(date)
    }

    fn convert_date_time_to_local(date_time: NaiveDateTime) -> DateTime<Local> {
        Local.from_utc_datetime(&date_time)
    }

    fn parse_date_time(date: impl AsRef<str>, format: impl AsRef<str>) -> Self {
        let date = date.as_ref();
        let format = format.as_ref();
        let date = NaiveDateTime::parse_from_str(date, format)
            .map(Self::convert_date_time_to_local)
            .unwrap_or_default();

        Self::new(date)
    }

    fn to_format(date_time: DateTime<Local>) -> SharedString {
        date_time.format("%Y-%m-%d %H:%M:%S").to_string().into()
    }
}
