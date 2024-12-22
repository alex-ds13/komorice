pub mod border;
pub mod general;
pub mod monitor;
pub mod monitors;
pub mod rule;
pub mod rules;
pub mod sidebar;
pub mod stackbar;
pub mod transparency;
pub mod workspace;

use std::fmt::{Display, Formatter};

use iced::{widget::value, Element};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Screen {
    #[default]
    Home,
    General,
    Monitors,
    Border,
    Stackbar,
    Transparency,
    Rules,
    Debug,
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
            Screen::Rules => write!(f, "Rules"),
            Screen::Debug => write!(f, "Debug"),
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
