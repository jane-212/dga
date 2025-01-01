mod item;
mod selectors;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use gpui::SharedString;
use item::{Item, Preview};
use reqwest::Client;
use scraper::Html;
use selectors::{HomeSelectors, PreviewSelectors};

use super::{Finder, Result};
use crate::{Date, FoundItem, FoundPreview, Size};

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

    fn parse_size(size: String) -> Size {
        let number = size
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .parse::<f64>()
            .ok();
        let signal: String = size
            .chars()
            .rev()
            .take_while(|c| !c.is_ascii_digit() && *c != '.')
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        const PAD: f64 = 1024.0;
        let size = match signal.to_uppercase().as_str() {
            "GB" => number.map(|number| (number * PAD.powi(3)) as u32),
            "MB" => number.map(|number| (number * PAD.powi(2)) as u32),
            "KB" => number.map(|number| (number * PAD.powi(1)) as u32),
            "B" => number.map(|number| number as u32),
            _ => None,
        }
        .unwrap_or(0);

        Size::new(size)
    }

    fn parse_date(date: String) -> Date {
        let date = NaiveDateTime::parse_from_str(date.trim(), "%Y-%m-%d %H:%M:%S")
            .map(|ndt| DateTime::from_naive_utc_and_offset(ndt, Utc))
            .ok()
            .unwrap_or_default();

        Date::new(date)
    }
}

#[async_trait]
impl Finder for U3C3 {
    async fn find(&self, key: SharedString) -> Result<Vec<Box<dyn FoundItem>>> {
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

        let mut items: Vec<Box<dyn FoundItem>> = Vec::new();
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

            items.push(Box::new(Item::new(
                title,
                Self::parse_size(size),
                Self::parse_date(date),
                preview,
            )));
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
        let preview = Preview::new(
            title,
            Self::parse_size(size),
            Self::parse_date(date),
            magnet,
            images,
        );

        Ok(Arc::new(preview))
    }
}
