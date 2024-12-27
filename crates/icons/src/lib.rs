use gpui::SharedString;
use ui::{Icon, IconNamed};

pub enum IconName {
    Sun,
    Moon,
    Github,
    CopyRight,
    Search,
    CircleX,
}

impl IconNamed for IconName {
    fn path(&self) -> SharedString {
        match self {
            IconName::Sun => "icons/sun.svg",
            IconName::Moon => "icons/moon.svg",
            IconName::Github => "icons/github.svg",
            IconName::CopyRight => "icons/copyright.svg",
            IconName::Search => "icons/search.svg",
            IconName::CircleX => "icons/circle-x.svg",
        }
        .into()
    }
}

impl From<IconName> for Icon {
    fn from(value: IconName) -> Self {
        Icon::default().path(value.path())
    }
}
