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
pub mod whkd_sidebar;
pub mod workspace;

use std::fmt::{Display, Formatter};

use iced::{
    widget::{value, vertical_space},
    Element, Task,
};

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
            Screen::WhkdBinding => write!(f, "Whkd Binding"),
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

#[derive(Clone, Debug)]
pub enum SidebarMessage {
    SelectScreen(Screen),
}

#[derive(Clone, Debug)]
pub enum SidebarAction {
    None,
    UpdateMainScreen(Screen),
}

#[derive(Debug, Default, Clone)]
pub enum Sidebar {
    #[default]
    None,
    Config(sidebar::Sidebar),
    Whkd(whkd_sidebar::WhkdSidebar),
}

impl Sidebar {
    pub fn update(&mut self, message: SidebarMessage) -> (SidebarAction, Task<SidebarMessage>) {
        match self {
            Sidebar::None => (SidebarAction::None, Task::none()),
            Sidebar::Config(sidebar) => sidebar.update(message),
            Sidebar::Whkd(whkd_sidebar) => whkd_sidebar.update(message),
        }
    }

    pub fn view(&self) -> Element<SidebarMessage> {
        match self {
            Sidebar::None => vertical_space().into(),
            Sidebar::Config(sidebar) => sidebar.view(),
            Sidebar::Whkd(whkd_sidebar) => whkd_sidebar.view(),
        }
    }
}
