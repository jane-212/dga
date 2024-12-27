use gpui::SharedString;

pub struct Item {
    pub title: SharedString,
    pub size: SharedString,
    pub date: SharedString,
    pub preview: SharedString,
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
            preview: preview.into(),
        }
    }
}
