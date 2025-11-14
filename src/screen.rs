pub mod animation;
pub mod border;
pub mod general;
pub mod home;
pub mod live_debug;
pub mod monitor;
pub mod monitors;
pub mod rule;
pub mod rules;
pub mod sidebar;
pub mod stackbar;
pub mod theme;
pub mod transparency;
pub mod whkd;
pub mod workspace;

use std::fmt::{Display, Formatter};

use iced::{Element, widget::value};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Screen {
    #[default]
    Home,
    General,
    Monitors,
    Border,
    Stackbar,
    Transparency,
    Animations,
    Theme,
    Rules,
    LiveDebug,
    Settings,
    Whkd,
    WhkdBinding,
}

impl Display for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::Home => write!(f, "Home"),
            Screen::General => write!(f, "General"),
            Screen::Monitors => write!(f, "Monitors"),
            Screen::Border => write!(f, "Border"),
            Screen::Stackbar => write!(f, "Stackbar"),
            Screen::Transparency => write!(f, "Transparency"),
            Screen::Animations => write!(f, "Animations"),
            Screen::Theme => write!(f, "Theme"),
            Screen::Rules => write!(f, "Rules"),
            Screen::LiveDebug => write!(f, "Live Debug"),
            Screen::Settings => write!(f, "Settings"),
            Screen::Whkd => write!(f, "Whkd"),
            Screen::WhkdBinding => write!(f, "Bindings"),
        }
    }
}

impl<Message> From<Screen> for Element<'_, Message> {
    fn from(screen: Screen) -> Self {
        value(screen).into()
    }
}

impl<Message> From<&Screen> for Element<'_, Message> {
    fn from(screen: &Screen) -> Self {
        value(screen).into()
    }
}

#[derive(Debug, Default, Clone)]
pub enum ConfigType {
    #[default]
    Komorebi,
    Whkd,
}

impl ConfigType {
    pub fn file_str(&self) -> &'static str {
        match self {
            ConfigType::Komorebi => "config",
            ConfigType::Whkd => "whkdrc",
        }
    }
}

impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigType::Komorebi => write!(f, "Komorebi"),
            ConfigType::Whkd => write!(f, "Whkd"),
        }
    }
}
