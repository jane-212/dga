mod error;
mod finder;

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use error::Error;
pub use error::Result;
use finder::Finder;
use gpui::SharedString;
use lazy_static::lazy_static;
use reqwest::Client;
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct Magnet {
    finders: HashMap<TypeId, Arc<dyn Finder>>,
}

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
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

    pub async fn find(&self, key: SharedString) -> Result<Vec<Arc<dyn FoundItem>>> {
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
    fn size(&self) -> SharedString;
    fn date(&self) -> SharedString;
    fn url(&self) -> Arc<dyn Previewable>;
}

pub trait Previewable: Send + Sync + 'static {
    fn preview_url(&self) -> (TypeId, SharedString);
}

pub trait FoundPreview: Send + Sync {
    fn title(&self) -> SharedString;
    fn size(&self) -> SharedString;
    fn date(&self) -> SharedString;
    fn magnet(&self) -> SharedString;
    fn images(&self) -> Vec<SharedString>;
}
