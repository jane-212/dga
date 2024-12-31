use std::any::TypeId;
use std::sync::Arc;

use gpui::SharedString;

use super::U3C3;
use crate::{FoundItem, FoundPreview, Previewable};

pub struct Item {
    pub title: SharedString,
    pub size: SharedString,
    pub date: SharedString,
    pub preview: Url,
}

impl Item {
    pub fn new(
        title: impl Into<SharedString>,
        size: impl Into<SharedString>,
        date: impl Into<SharedString>,
        preview: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            size: size.into(),
            date: date.into(),
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

    fn size(&self) -> SharedString {
        self.size.clone()
    }

    fn date(&self) -> SharedString {
        self.date.clone()
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
    pub size: SharedString,
    pub date: SharedString,
    pub magnet: SharedString,
    pub images: Vec<SharedString>,
}

impl Preview {
    pub fn new(
        title: impl Into<SharedString>,
        size: impl Into<SharedString>,
        date: impl Into<SharedString>,
        magnet: impl Into<SharedString>,
        images: Vec<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            size: size.into(),
            date: date.into(),
            magnet: magnet.into(),
            images,
        }
    }
}

impl FoundPreview for Preview {
    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn size(&self) -> SharedString {
        self.size.clone()
    }

    fn date(&self) -> SharedString {
        self.date.clone()
    }

    fn magnet(&self) -> SharedString {
        self.magnet.clone()
    }

    fn images(&self) -> Vec<SharedString> {
        self.images.clone()
    }
}
