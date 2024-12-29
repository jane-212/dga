use scraper::Selector;

use super::Result;
use crate::select;

pub(crate) struct HomeSelectors {
    pub item: Selector,
    pub title: Selector,
    pub size: Selector,
    pub date: Selector,
}

impl HomeSelectors {
    pub fn new() -> Result<Self> {
        let item = select!("tr.default");
        let title = select!("td:nth-child(2) > a:nth-child(1)");
        let size = select!("td:nth-child(4)");
        let date = select!("td:nth-child(5)");

        Ok(Self {
            item,
            title,
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
        let title = select!("div.panel:nth-child(1) > div:nth-child(1) > h3:nth-child(1)");
        let size = select!("div.row:nth-child(3) > div:nth-child(2)");
        let date = select!("div.row:nth-child(1) > div:nth-child(4)");
        let magnet = select!(".card-footer-item");
        let images = select!("div.panel:nth-child(4) > div:nth-child(1) > img:nth-child(1)");

        Ok(Self {
            title,
            size,
            date,
            magnet,
            images,
        })
    }
}
