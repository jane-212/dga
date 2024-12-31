mod u3c3;

use std::{any::TypeId, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use gpui::SharedString;
use reqwest::Client;
use u3c3::U3C3;

use super::Result;
use crate::{FoundItem, FoundPreview};

#[async_trait]
pub trait Finder: Send + Sync {
    async fn find(&self, key: SharedString) -> Result<Vec<Box<dyn FoundItem>>>;
    async fn load_preview(&self, url: SharedString) -> Result<Arc<dyn FoundPreview>>;
}

fn cast(u3c3: U3C3) -> Arc<dyn Finder> {
    Arc::new(u3c3)
}

pub fn all_finders(client: Client) -> Result<HashMap<TypeId, Arc<dyn Finder>>> {
    let mut finders = HashMap::new();
    let u3c3 = U3C3::new(client.clone())?;
    finders.insert(TypeId::of::<U3C3>(), cast(u3c3));

    Ok(finders)
}

#[macro_export]
macro_rules! select {
    ($path:expr) => {
        scraper::Selector::parse($path).map_err(|_e| $crate::Error::Parse($path))?
    };
}
