mod error;
mod finder;
mod item;

use error::{Error, Result};
use finder::Finder;
use gpui::SharedString;
pub use item::Item;
use reqwest::Client;
use std::sync::OnceLock;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct Magnet {
    finders: Vec<Box<dyn Finder>>,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

impl Magnet {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_e| Error::BuildClient)?;
        let finders = finder::all_finders(Arc::new(client))?;

        Ok(Self { finders })
    }

    pub async fn find(self, key: SharedString) -> Result<Vec<Item>> {
        let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
        let handle = runtime.spawn(async move {
            let mut tasks = Vec::new();
            for finder in self.finders {
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
}
