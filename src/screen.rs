pub mod animation;
pub mod border;
pub mod general;
pub mod live_debug;
pub mod monitor;
pub mod monitors;
pub mod rule;
pub mod rules;
pub mod sidebar;
pub mod stackbar;
pub mod theme;
pub mod transparency;
pub mod workspace;

use crate::config::DEFAULT_CONFIG;

use std::fmt::{Display, Formatter};

use iced::{widget::value, Element};

pub struct Screens {
    pub general: general::General,
    pub monitors: monitors::Monitors,
    pub border: border::Border,
    pub stackbar: stackbar::Stackbar,
    pub transparency: transparency::Transparency,
    pub animation: animation::Animation,
    pub theme: theme::Theme,
    pub rules: rules::Rules,
    pub live_debug: live_debug::LiveDebug,
    pub settings: crate::settings::Settings,
    pub sidebar: sidebar::Sidebar,
}

impl Default for Screens {
    fn default() -> Self {
        Self {
            general: Default::default(),
            monitors: monitors::Monitors::new(&DEFAULT_CONFIG),
            border: Default::default(),
            stackbar: Default::default(),
            transparency: Default::default(),
            animation: Default::default(),
            theme: Default::default(),
            rules: Default::default(),
            live_debug: Default::default(),
            settings: Default::default(),
            sidebar: Default::default(),
        }
    }
}

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
