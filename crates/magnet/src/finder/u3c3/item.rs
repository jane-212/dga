use std::any::TypeId;
use std::sync::Arc;

use gpui::SharedString;

use super::U3C3;
use crate::{Date, FoundItem, FoundPreview, Previewable, Size};

pub struct Item {
    pub title: SharedString,
    pub size: Size,
    pub date: Date,
    pub preview: Url,
}

impl Item {
    pub fn new(
        title: impl Into<SharedString>,
        size: Size,
        date: Date,
        preview: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            size,
            date,
            preview: Url(preview.into()),
        }
    }
}

impl FoundItem for Item {
    fn url(&self) -> Arc<dyn Previewable> {
        Arc::new(self.preview.clone())
    }

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn size(&self) -> &Size {
        &self.size
    }

    fn date(&self) -> &Date {
        &self.date
    }
}

#[derive(Clone)]
pub struct Url(SharedString);

impl Previewable for Url {
    fn preview_url(&self) -> (TypeId, SharedString) {
        (TypeId::of::<U3C3>(), self.0.clone())
    }
}

pub struct Preview {
    pub title: SharedString,
    pub size: Size,
    pub date: Date,
    pub magnet: SharedString,
    pub images: Vec<SharedString>,
}

impl Preview {
    pub fn new(
        title: impl Into<SharedString>,
        size: Size,
        date: Date,
        magnet: impl Into<SharedString>,
        images: Vec<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            size,
            date,
            magnet: magnet.into(),
            images,
        }
    }
}

impl FoundPreview for Preview {
    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn size(&self) -> &Size {
        &self.size
    }

    fn date(&self) -> &Date {
        &self.date
    }

    fn magnet(&self) -> SharedString {
        self.magnet.clone()
    }

    fn images(&self) -> Vec<SharedString> {
        self.images.clone()
    }
}
