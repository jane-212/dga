mod item;
mod selectors;

use std::sync::Arc;

use async_trait::async_trait;
use gpui::SharedString;
use item::{Item, Preview};
use reqwest::Client;
use scraper::Html;
use selectors::{HomeSelectors, PreviewSelectors};

use super::{Finder, Result};
use crate::{FoundItem, FoundPreview};

pub struct U3C3 {
    client: Client,
    home_selectors: HomeSelectors,
    preview_selectors: PreviewSelectors,
}

impl U3C3 {
    const BASE_URL: &'static str = "https://u3c3.com";

    pub fn new(client: Client) -> Result<Self> {
        Ok(Self {
            client,
            home_selectors: HomeSelectors::new()?,
            preview_selectors: PreviewSelectors::new()?,
        })
    }
}

#[async_trait]
impl Finder for U3C3 {
    async fn find(&self, key: SharedString) -> Result<Vec<Arc<dyn FoundItem>>> {
        let url = Self::BASE_URL;
        let search2 = "eelja3lfe1a1".into();
        let plain_text = self
            .client
            .get(url)
            .query(&[("search", key), ("search2", search2)])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&plain_text);

        let mut items: Vec<Arc<dyn FoundItem>> = Vec::new();
        for item in html.select(&self.home_selectors.item).skip(2) {
            let title = item
                .select(&self.home_selectors.title)
                .next()
                .and_then(|title| title.attr("title").map(|title| title.to_string()))
                .unwrap_or_default();

            let preview = item
                .select(&self.home_selectors.title)
                .next()
                .and_then(|title| {
                    title
                        .attr("href")
                        .map(|href| format!("{}{}", Self::BASE_URL, href))
                })
                .unwrap_or_default();

            let size: String = item
                .select(&self.home_selectors.size)
                .next()
                .map(|size| size.text().collect())
                .unwrap_or_default();

            let date: String = item
                .select(&self.home_selectors.date)
                .next()
                .map(|date| date.text().collect())
                .unwrap_or_default();

            items.push(Arc::new(Item::new(title, size, date, preview)));
        }

        Ok(items)
    }

    async fn load_preview(&self, url: SharedString) -> Result<Arc<dyn FoundPreview>> {
        let text = self
            .client
            .get(url.to_string())
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);
        let title = html
            .select(&self.preview_selectors.title)
            .next()
            .map(|this| this.text().collect())
            .map(|this: String| this.trim().to_string())
            .unwrap_or_default();
        let size: String = html
            .select(&self.preview_selectors.size)
            .next()
            .map(|this| this.text().collect())
            .unwrap_or_default();
        let date: String = html
            .select(&self.preview_selectors.date)
            .next()
            .map(|this| this.text().collect())
            .unwrap_or_default();
        let magnet = html
            .select(&self.preview_selectors.magnet)
            .next()
            .and_then(|this| {
                this.attr("href")
                    .and_then(|this| this.find('&').map(|end| this[..end].to_string()))
            })
            .unwrap_or_default();
        let images: Vec<SharedString> = html
            .select(&self.preview_selectors.images)
            .next()
            .and_then(|this| {
                this.attr("src")
                    .map(|this| format!("{}{}", Self::BASE_URL, this))
            })
            .iter()
            .map(|item| item.into())
            .collect();
        let preview = Preview::new(title, size, date, magnet, images);

        Ok(Arc::new(preview))
    }
}
