use scraper::Selector;

use super::Result;
use crate::select;

pub(crate) struct HomeSelectors {
    pub item: Selector,
    pub url: Selector,
    pub title: Selector,
    pub id: Selector,
    pub date: Selector,
}

impl HomeSelectors {
    pub fn new() -> Result<Self> {
        let item = select!("body > section > div > div.movie-list.h.cols-4.vcols-8 > div");
        let url = select!("a");
        let title = select!("a");
        let id = select!("a > div.video-title > strong");
        let date = select!("a > div.meta");

        Ok(Self {
            item,
            url,
            title,
            id,
            date,
        })
    }
}

pub(crate) struct PreviewSelectors {
    pub title: Selector,
    pub samples: Selector,
    pub items: Selector,
    pub date: Selector,
    pub size: Selector,
    pub url: Selector,
}

impl PreviewSelectors {
    pub fn new() -> Result<Self> {
        let title = select!("body > section > div > div.video-detail > h2 > strong.current-title");
        let samples =
            select!("body > section > div > div.video-detail > div:nth-child(3) > div > article > div > div > a > img");
        let items = select!("#magnets-content > div");
        let date = select!("div.date.column > span");
        let size = select!("div.magnet-name.column.is-four-fifths > a > span.meta");
        let url = select!("div.magnet-name.column.is-four-fifths > a");

        Ok(Self {
            title,
            samples,
            items,
            date,
            size,
            url,
        })
    }
}
