use scraper::Selector;

use super::Result;
use crate::select;

pub(crate) struct HomeSelectors {
    pub item: Selector,
    pub title: Selector,
    pub preview: Selector,
    pub size: Selector,
    pub date: Selector,
}

impl HomeSelectors {
    pub fn new() -> Result<Self> {
        let item = select!("tr.default");
        let title = select!("td:nth-child(2) > a > span");
        let preview = select!("td:nth-child(2) > a");
        let size = select!("td:nth-child(3)");
        let date = select!("td:nth-child(1)");

        Ok(Self {
            item,
            title,
            preview,
            size,
            date,
        })
    }
}

pub(crate) struct PreviewSelectors {
    pub title: Selector,
    pub size: Selector,
    pub date: Selector,
    pub magnet: Selector,
    pub images: Selector,
}

impl PreviewSelectors {
    pub fn new() -> Result<Self> {
        let title = select!("body > div:nth-child(5) > div:nth-child(1) > div.panel-heading > h3");
        let size = select!("body > div:nth-child(5) > div:nth-child(1) > div.panel-body > div:nth-child(2) > div:nth-child(2)");
        let date = select!("body > div:nth-child(5) > div:nth-child(1) > div.panel-body > div:nth-child(1) > div:nth-child(2)");
        let magnet = select!("body > div:nth-child(5) > div.download > div > a:nth-child(2)");
        let images = select!("#torrent-description > div > img");

        Ok(Self {
            title,
            size,
            date,
            magnet,
            images,
        })
    }
}
