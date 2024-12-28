use gpui::SharedString;

#[derive(Clone)]
pub struct Item {
    pub title: SharedString,
    pub size: SharedString,
    pub date: SharedString,
    pub preview: PreviewUrl,
}

impl Item {
    pub fn new(
        title: impl Into<SharedString>,
        size: impl Into<SharedString>,
        date: impl Into<SharedString>,
        preview: PreviewUrl,
    ) -> Self {
        Self {
            title: title.into(),
            size: size.into(),
            date: date.into(),
            preview,
        }
    }
}

#[derive(Clone)]
pub enum PreviewUrl {
    U3C3(SharedString),
}

impl PreviewUrl {
    pub fn u3c3(url: impl Into<SharedString>) -> Self {
        Self::U3C3(url.into())
    }

    pub fn url(self) -> SharedString {
        match self {
            PreviewUrl::U3C3(url) => url,
        }
    }
}

#[derive(Clone)]
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
