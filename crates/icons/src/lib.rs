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
    Globe,
    User,
    Lock,
    FileX,
    Download,
    Upload,
    CirclePause,
    ListStart,
    ListEnd,
    CircleCheck,
    FileClock,
    HardDrive,
    FileCog,
    FileSearch,
    FileOutput,
    CircleHelp,
    LogOut,
    FastForward,
    ClipboardPlus,
    Plus,
    Trash2,
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
            IconName::Globe => "icons/globe.svg",
            IconName::User => "icons/user.svg",
            IconName::Lock => "icons/lock.svg",
            IconName::FileX => "icons/file-x.svg",
            IconName::Download => "icons/download.svg",
            IconName::Upload => "icons/upload.svg",
            IconName::CirclePause => "icons/circle-pause.svg",
            IconName::ListStart => "icons/list-start.svg",
            IconName::ListEnd => "icons/list-end.svg",
            IconName::CircleCheck => "icons/circle-check.svg",
            IconName::FileClock => "icons/file-clock.svg",
            IconName::HardDrive => "icons/hard-drive.svg",
            IconName::FileCog => "icons/file-cog.svg",
            IconName::FileSearch => "icons/file-search.svg",
            IconName::FileOutput => "icons/file-output.svg",
            IconName::CircleHelp => "icons/circle-help.svg",
            IconName::LogOut => "icons/log-out.svg",
            IconName::FastForward => "icons/fast-forward.svg",
            IconName::ClipboardPlus => "icons/clipboard-plus.svg",
            IconName::Plus => "icons/plus.svg",
            IconName::Trash2 => "icons/trash-2.svg",
        }
        .into()
    }
}

impl From<IconName> for Icon {
    fn from(value: IconName) -> Self {
        Icon::default().path(value.path())
    }
}
