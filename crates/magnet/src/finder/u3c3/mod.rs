mod item;
mod selectors;

use std::sync::Arc;

use async_trait::async_trait;
use gpui::SharedString;
use item::{Data, Item, Preview};
use reqwest::Client;
use scraper::Html;
use selectors::{HomeSelectors, PreviewSelectors};

use super::{Finder, FoundItem, FoundPreview, Result};
use crate::{Date, Size};

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
            "GB" => number.map(|number| (number * PAD.powi(3)) as u64),
            "MB" => number.map(|number| (number * PAD.powi(2)) as u64),
            "KB" => number.map(|number| (number * PAD.powi(1)) as u64),
            "B" => number.map(|number| number as u64),
            _ => None,
        }
        .unwrap_or(0);

        Size::new(size)
    }

    fn parse_date(date: String) -> Date {
        Date::parse_date_time(date, "%Y-%m-%d %H:%M:%S")
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
                .and_then(|title| title.attr("title").map(String::from))
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

            let new_item = Item::builder()
                .title(title)
                .size(Self::parse_size(size))
                .date(Self::parse_date(date))
                .preview(preview)
                .build();
            items.push(Box::new(new_item));
        }

        Ok(items)
    }

    async fn load_preview(&self, url: SharedString) -> Result<Box<dyn FoundPreview>> {
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

        let size = Self::parse_size(size);
        let date = Self::parse_date(date);
        let data = Data::new(size, date, magnet);
        let preview = Preview::new(title, vec![Arc::new(data)], images);

        Ok(Box::new(preview))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_finder() -> U3C3 {
        U3C3::new(Client::new()).unwrap()
    }

    #[tokio::test]
    async fn find() {
        let finder = new_finder();
        let list = finder.find("stars-804".into()).await.unwrap();
        assert!(!list.is_empty());
    }

    #[tokio::test]
    async fn preview() {
        let finder = new_finder();
        let preview = finder
            .load_preview(
                format!(
                    "{}/view?id=73f25941f75cb6f8eebe727ae78a2c0c5dfcdb1a",
                    U3C3::BASE_URL
                )
                .into(),
            )
            .await
            .unwrap();
        assert!(!preview.title().is_empty());
        assert!(!preview.bounds().is_empty());
        assert!(!preview.images().is_empty());
    }
}
