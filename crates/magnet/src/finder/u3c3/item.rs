use std::any::TypeId;
use std::sync::Arc;

use gpui::SharedString;

use super::U3C3;
use crate::{Bound, Date, FoundItem, FoundPreview, Previewable, Size};

pub struct Item {
    title: SharedString,
    size: Size,
    date: Date,
    preview: Url,
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
    title: SharedString,
    bounds: Vec<Arc<dyn Bound>>,
    images: Vec<SharedString>,
}

pub struct Data {
    size: Size,
    date: Date,
    magnet: SharedString,
}

impl Data {
    pub fn new(size: Size, date: Date, magnet: impl Into<SharedString>) -> Self {
        Self {
            size,
            date,
            magnet: magnet.into(),
        }
    }
}

impl Bound for Data {
    fn size(&self) -> &Size {
        &self.size
    }

    fn date(&self) -> &Date {
        &self.date
    }

    fn magnet(&self) -> SharedString {
        self.magnet.clone()
    }
}

impl Preview {
    pub fn new(
        title: impl Into<SharedString>,
        bounds: Vec<Arc<dyn Bound>>,
        images: Vec<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            bounds,
            images,
        }
    }
}

impl FoundPreview for Preview {
    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn images(&self) -> Vec<SharedString> {
        self.images.clone()
    }

    fn bounds(&self) -> Vec<Arc<dyn Bound>> {
        self.bounds.clone()
    }
}
