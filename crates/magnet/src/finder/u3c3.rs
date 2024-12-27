use async_trait::async_trait;
use gpui::SharedString;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;

use super::{Finder, Item, Result};
use crate::select;

#[derive(Clone)]
pub struct U3C3 {
    client: Arc<Client>,
    item_sel: Selector,
    title_sel: Selector,
    size_sel: Selector,
    date_sel: Selector,
}

impl U3C3 {
    const BASE_URL: &'static str = "https://u3c3.com";

    pub fn new(client: Arc<Client>) -> Result<Self> {
        let item_sel = select!("tr.default");
        let title_sel = select!("td:nth-child(2) > a:nth-child(1)");
        let size_sel = select!("td:nth-child(4)");
        let date_sel = select!("td:nth-child(5)");

        Ok(Self {
            client,
            item_sel,
            title_sel,
            size_sel,
            date_sel,
        })
    }
}

#[async_trait]
impl Finder for U3C3 {
    async fn find(&self, key: SharedString) -> Result<Vec<Item>> {
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

        let mut items = Vec::new();
        for item in html.select(&self.item_sel).skip(2) {
            let title = item
                .select(&self.title_sel)
                .next()
                .and_then(|title| title.attr("title").map(|title| title.to_string()))
                .unwrap_or_default();

            let preview = item
                .select(&self.title_sel)
                .next()
                .and_then(|title| {
                    title
                        .attr("href")
                        .map(|href| format!("{}{}", Self::BASE_URL, href))
                })
                .unwrap_or_default();

            let size: String = item
                .select(&self.size_sel)
                .next()
                .map(|size| size.text().collect())
                .unwrap_or_default();

            let date: String = item
                .select(&self.date_sel)
                .next()
                .map(|date| date.text().collect())
                .unwrap_or_default();

            items.push(Item::new(title, size, date, preview));
        }

        Ok(items)
    }
}
