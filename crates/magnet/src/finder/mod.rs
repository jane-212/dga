mod u3c3;

use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use gpui::SharedString;
use reqwest::Client;
use std::sync::Arc;
use u3c3::U3C3;

use super::{Item, Result};

#[async_trait]
pub trait Finder: DynClone + Send + Sync {
    async fn find(&self, key: SharedString) -> Result<Vec<Item>>;
}

clone_trait_object!(Finder);

pub fn all_finders(client: Arc<Client>) -> Result<Vec<Box<dyn Finder>>> {
    let u3c3 = U3C3::new(client.clone())?;

    Ok(vec![Box::new(u3c3)])
}

#[macro_export]
macro_rules! select {
    ($path:expr) => {
        scraper::Selector::parse($path).map_err(|_e| $crate::Error::Parse($path))?
    };
}
