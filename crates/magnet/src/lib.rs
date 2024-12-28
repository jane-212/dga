mod error;
mod finder;
mod item;

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::{sync::Arc, time::Duration};

use error::{Error, Result};
use finder::Finder;
use gpui::SharedString;
pub use item::{Item, Preview, PreviewUrl};
use reqwest::Client;
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct Magnet {
    finders: HashMap<TypeId, Arc<dyn Finder>>,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}

impl Magnet {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_e| Error::BuildClient)?;
        let finders = finder::all_finders(client)?;

        Ok(Self { finders })
    }

    pub async fn find(self, key: SharedString) -> Result<Vec<Item>> {
        let runtime = runtime();
        let handle = runtime.spawn(async move {
            let mut tasks = Vec::new();
            for finder in self.finders.values() {
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
        });

        handle.await?
    }

    pub async fn preview(self, preview: PreviewUrl) -> Result<Preview> {
        let runtime = runtime();
        let handle = runtime.spawn(async move {
            let id = finder::finder_id(&preview);
            let url = preview.url();
            match self.finders.get(&id) {
                Some(finder) => finder.load_preview(url).await,
                None => Err(Error::TypeNotFound),
            }
        });

        handle.await?
    }
}
