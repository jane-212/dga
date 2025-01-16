mod javdb;
mod madou;
mod u3c3;

use std::{any::TypeId, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use gpui::SharedString;
use javdb::Javdb;
use madou::Madou;
use reqwest::Client;
use u3c3::U3C3;

use super::Result;
use crate::{FoundItem, FoundPreview};

#[async_trait]
pub trait Finder: Send + Sync {
    async fn find(&self, key: SharedString) -> Result<Vec<Box<dyn FoundItem>>>;
    async fn load_preview(&self, url: SharedString) -> Result<Box<dyn FoundPreview>>;
}

pub fn all_finders(client: Client) -> Result<HashMap<TypeId, Arc<dyn Finder>>> {
    let mut finders = HashMap::new();

    let u3c3 = U3C3::new(client.clone())?;
    finders.insert(TypeId::of::<U3C3>(), Arc::new(u3c3) as Arc<dyn Finder>);

    let javdb = Javdb::new(client.clone())?;
    finders.insert(TypeId::of::<Javdb>(), Arc::new(javdb));

    let madou = Madou::new(client.clone())?;
    finders.insert(TypeId::of::<Madou>(), Arc::new(madou));

    Ok(finders)
}

#[macro_export]
macro_rules! select {
    ($path:expr) => {
        scraper::Selector::parse($path).map_err(|_e| $crate::Error::Parse($path))?
    };
}
