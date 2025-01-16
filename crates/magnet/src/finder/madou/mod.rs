mod item;
mod selectors;

use std::sync::Arc;

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Datelike, Local};
use gpui::SharedString;
use item::{Data, Item, Preview};
use reqwest::Client;
use scraper::Html;
use selectors::{HomeSelectors, PreviewSelectors};

use super::{Finder, FoundItem, FoundPreview, Result};
use crate::{Date, Size};

pub struct Madou {
    client: Client,
    home_selectors: HomeSelectors,
    preview_selectors: PreviewSelectors,
}

impl Madou {
    const BASE_URL: &'static str = "https://hxx.533923.xyz";

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
            .take_while(|c| !c.is_ascii_digit() && *c != '.' && !c.is_whitespace())
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
        let now = Local::now();
        let (month, day) = if let Some((month_str, day_str)) = date.split_once('-') {
            (
                month_str.parse().unwrap_or(now.month()),
                day_str.parse().unwrap_or(now.day()),
            )
        } else {
            (now.month(), now.day())
        };

        Date::from_ymd(now.year() - 1, month, day)
    }

    fn base64_decode(origin: String) -> String {
        STANDARD
            .decode(origin)
            .ok()
            .and_then(|data| String::from_utf8(data).ok())
            .unwrap_or_default()
    }
}

#[async_trait]
impl Finder for Madou {
    async fn find(&self, key: SharedString) -> Result<Vec<Box<dyn FoundItem>>> {
        let url = format!("{}/search.php", Self::BASE_URL);
        let plain_text = self
            .client
            .post(url)
            .form(&[("keyword", &key)])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&plain_text);

        let mut items = Vec::new();
        for item in html.select(&self.home_selectors.item) {
            let title: String = item
                .select(&self.home_selectors.title)
                .next()
                .map(|title| title.text().collect())
                .and_then(|title: String| {
                    title
                        .split('\'')
                        .nth(1)
                        .map(|undecoded| undecoded.to_string())
                })
                .map(Self::base64_decode)
                .unwrap_or_default();

            let preview = item
                .select(&self.home_selectors.preview)
                .next()
                .and_then(|preview| {
                    preview
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
            items.push(Box::new(new_item) as Box<dyn FoundItem>);
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
            .and_then(|title: String| {
                title
                    .split('\'')
                    .nth(1)
                    .map(|undecoded| undecoded.to_string())
            })
            .map(Self::base64_decode)
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
            .and_then(|this| this.attr("href").map(|href| href.to_string()))
            .unwrap_or_default();
        let images: Vec<SharedString> = html
            .select(&self.preview_selectors.images)
            .flat_map(|this| this.attr("src"))
            .map(|item| item.to_string().into())
            .collect();

        let size = Self::parse_size(size);
        let date = Date::parse_date(date.trim(), "%Y-%m-%d");
        let data = Data::new(size, date, magnet);
        let preview = Preview::new(title, vec![Arc::new(data)], images);

        Ok(Box::new(preview))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_finder() -> Madou {
        Madou::new(Client::new()).unwrap()
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
            .load_preview(format!("{}/movie.php?id=11343687", Madou::BASE_URL).into())
            .await
            .unwrap();
        assert!(!preview.title().is_empty());
        assert!(!preview.bounds().is_empty());
        assert!(!preview.images().is_empty());
    }
}
