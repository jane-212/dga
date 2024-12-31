use gpui::SharedString;
use ui::{Icon, IconNamed};

pub enum IconName {
    Sun,
    Moon,
    Github,
    CopyRight,
    Search,
    CircleX,
    Info,
    Loader,
    Copy,
    Check,
    House,
    HardDriveDownload,
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
            IconName::Info => "icons/info.svg",
            IconName::Loader => "icons/loader.svg",
            IconName::Copy => "icons/copy.svg",
            IconName::Check => "icons/check.svg",
            IconName::House => "icons/house.svg",
            IconName::HardDriveDownload => "icons/hard-drive-download.svg",
        }
        .into()
    }
}

impl From<IconName> for Icon {
    fn from(value: IconName) -> Self {
        Icon::default().path(value.path())
    }
}
