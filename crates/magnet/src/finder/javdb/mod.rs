mod item;
mod selectors;

use std::sync::Arc;

use async_trait::async_trait;
use gpui::SharedString;
use item::{Data, Item, Preview};
use reqwest::Client;
use scraper::Html;
use selectors::{HomeSelectors, PreviewSelectors};

use crate::{Bound, Date, Size};

use super::{Finder, FoundItem, FoundPreview, Result};

pub struct Javdb {
    client: Client,
    home_selectors: HomeSelectors,
    preview_selectors: PreviewSelectors,
}

impl Javdb {
    const BASE_URL: &'static str = "https://javdb.com";

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
}

#[async_trait]
impl Finder for Javdb {
    async fn find(&self, key: SharedString) -> Result<Vec<Box<dyn FoundItem>>> {
        let url = format!("{}/search", Self::BASE_URL);
        let text = self
            .client
            .get(url)
            .query(&[("q", key.to_string().as_str()), ("f", "all")])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        let mut items: Vec<Box<dyn FoundItem>> = Vec::new();
        for item in html.select(&self.home_selectors.item) {
            let url = item
                .select(&self.home_selectors.url)
                .next()
                .and_then(|url| {
                    url.attr("href")
                        .map(|url| format!("{}{}", Self::BASE_URL, url))
                })
                .unwrap_or_default();
            let title = item
                .select(&self.home_selectors.title)
                .next()
                .and_then(|this| this.attr("title").map(String::from))
                .unwrap_or_default();
            let id = item
                .select(&self.home_selectors.id)
                .next()
                .map(|id| id.text().collect::<String>())
                .unwrap_or_default();
            let date = item
                .select(&self.home_selectors.date)
                .next()
                .map(|date| date.text().collect::<String>())
                .unwrap_or_default();

            let date = Date::parse_date(date.trim(), "%Y-%m-%d");
            let new_item = Box::new(Item::new(title, id, date, url));
            items.push(new_item);
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
            .map(|title| title.text().collect::<String>())
            .unwrap_or_default();
        let images = html
            .select(&self.preview_selectors.samples)
            .flat_map(|sample| sample.attr("src").map(String::from).map(SharedString::from))
            .collect::<Vec<_>>();
        let mut bounds: Vec<_> = html
            .select(&self.preview_selectors.items)
            .map(|item| {
                let date = item
                    .select(&self.preview_selectors.date)
                    .next()
                    .map(|title| title.text().collect::<String>())
                    .unwrap_or_default();
                let size = item
                    .select(&self.preview_selectors.size)
                    .next()
                    .map(|value| value.text().collect::<String>())
                    .unwrap_or_default();
                let size = size
                    .split_once(',')
                    .map(|(size, _)| size.trim().to_string())
                    .unwrap_or(size.trim().to_string());
                let size = Self::parse_size(size);
                let magnet = item
                    .select(&self.preview_selectors.url)
                    .next()
                    .and_then(|url| {
                        url.attr("href")
                            .and_then(|href| href.split_once('&').map(|(url, _)| url.to_string()))
                    })
                    .unwrap_or_default();

                Arc::new(Data::new(
                    size,
                    Date::parse_date(date.trim(), "%Y-%m-%d"),
                    magnet,
                )) as Arc<dyn Bound>
            })
            .collect();
        bounds.sort_by(|a, b| b.date().cmp(a.date()));

        Ok(Box::new(Preview::new(title, bounds, images)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_finder() -> Javdb {
        Javdb::new(Client::new()).unwrap()
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
            .load_preview("https://javdb.com/v/qDYQzM".into())
            .await
            .unwrap();
        assert!(!preview.title().is_empty());
        assert!(!preview.bounds().is_empty());
        assert!(!preview.images().is_empty());
    }
}
