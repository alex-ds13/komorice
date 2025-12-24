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
pub mod wallpaper;
pub mod whkd;
pub mod workspace;

use std::fmt::{Display, Formatter};
use std::path::PathBuf;

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
pub struct Configuration {
    pub config_type: ConfigType,
    pub komorebi_state: ConfigState,
    pub whkd_state: ConfigState,
    pub has_loaded_komorebi: bool,
    pub has_loaded_whkd: bool,
    pub saved_new_komorebi: bool,
    pub saved_new_whkd: bool,
}

impl Configuration {
    pub fn path(&self) -> PathBuf {
        match self.config_type {
            ConfigType::Komorebi => match &self.komorebi_state {
                ConfigState::Active => crate::config::config_path(),
                ConfigState::Loaded(path_buf) | ConfigState::New(path_buf) => path_buf.clone(),
            },
            ConfigType::Whkd => match &self.whkd_state {
                ConfigState::Active => crate::whkd::config_path(),
                ConfigState::Loaded(path_buf) | ConfigState::New(path_buf) => path_buf.clone(),
            },
        }
    }

    pub fn state(&self, config_type: ConfigType) -> &ConfigState {
        match config_type {
            ConfigType::Komorebi => &self.komorebi_state,
            ConfigType::Whkd => &self.whkd_state,
        }
    }

    pub fn state_str(&self, config_type: ConfigType) -> &'static str {
        match config_type {
            ConfigType::Komorebi => self.komorebi_state.as_str(),
            ConfigType::Whkd => self.whkd_state.as_str(),
        }
    }

    pub fn has_loaded_or_is_new(&self, config_type: ConfigType) -> bool {
        match config_type {
            ConfigType::Komorebi => {
                self.has_loaded_komorebi || matches!(self.komorebi_state, ConfigState::New(_))
            }
            ConfigType::Whkd => {
                self.has_loaded_whkd || matches!(self.whkd_state, ConfigState::New(_))
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ConfigType {
    #[default]
    Komorebi,
    Whkd,
}

impl ConfigType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigType::Komorebi => "config",
            ConfigType::Whkd => "whkdrc",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            ConfigType::Komorebi => "Komorebi",
            ConfigType::Whkd => "Whkd",
        }
    }
}

impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title())
    }
}

#[derive(Debug, Default, Clone)]
pub enum ConfigState {
    #[default]
    Active,
    Loaded(PathBuf),
    New(PathBuf),
}

impl ConfigState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigState::Active => "active",
            ConfigState::Loaded(_) => "loaded",
            ConfigState::New(_) => "new",
        }
    }
}

/// A View object that can be returned by some screen which always includes the screen view element
/// and some optional modal.
pub struct View<'a, Message> {
    pub element: Element<'a, Message>,
    pub modal: Option<Modal<'a, Message>>,
}

impl<'a, Message> View<'a, Message> {
    /// Creates a new `View` with the given element.
    pub fn new(element: impl Into<Element<'a, Message>>) -> Self {
        Self {
            element: element.into(),
            modal: None,
        }
    }

    /// Adds a modal to the `View`, including a close message.
    pub fn modal(
        mut self,
        modal: Option<impl Into<Element<'a, Message>>>,
        close_message: Message,
    ) -> Self {
        self.modal = Some(Modal::new(modal, close_message));
        self
    }

    /// Applies a transformation to the produced message of the elements.
    pub fn map<MessageB, F>(self, f: F) -> View<'a, MessageB>
    where
        Message: 'a,
        MessageB: 'a,
        F: Fn(Message) -> MessageB + Clone + 'a,
    {
        View {
            element: self.element.map(f.clone()),
            modal: self.modal.map(|modal| modal.map(f.clone())),
        }
    }
}

impl<'a, Message, T> From<T> for View<'a, Message>
where
    T: Into<Element<'a, Message>>,
{
    fn from(value: T) -> Self {
        View::new(value.into())
    }
}

/// A Modal object that can be returned by some screen which always includes an optional modal element
/// and the close message.
pub struct Modal<'a, Message> {
    pub element: Option<Element<'a, Message>>,
    pub close_message: Message,
}

impl<'a, Message> Modal<'a, Message> {
    /// Creates a new `Modal` with the given optional element and close message.
    pub fn new(element: Option<impl Into<Element<'a, Message>>>, close_message: Message) -> Self {
        Self {
            element: element.map(Into::into),
            close_message,
        }
    }

    /// Applies a transformation to the produced message of the elements.
    pub fn map<MessageB, F>(self, f: F) -> Modal<'a, MessageB>
    where
        Message: 'a,
        MessageB: 'a,
        F: Fn(Message) -> MessageB + Clone + 'a,
    {
        Modal {
            element: self.element.map(|el| el.map(f.clone())),
            close_message: f(self.close_message),
        }
    }
}
