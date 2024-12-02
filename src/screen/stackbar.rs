use crate::widget::opt_helpers;

use iced::{Element, Task};
use komorebi::{Colour, Rgb, StackbarConfig, StackbarLabel, StackbarMode, TabsConfig};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    Height(i32),
    Label(StackbarLabel),
    Mode(StackbarMode),
    Width(i32),
    FontFamily(String),
    FontSize(i32),
    BackgroundColor(Colour),
    FocusedTextColor(Colour),
    UnfocusedTextColor(Colour),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Debug, Default)]
pub struct Stackbar {}

// pub trait StackbarScreen {
//     fn update(&mut self, message: Message) -> (Action, Task<Message>);
//
//     fn view(&self) -> Element<'_, Message>;
// }
//
// impl StackbarScreen for StackbarConfig {
// }

impl Stackbar {
    pub fn update(
        &mut self,
        config: &mut StackbarConfig,
        message: Message,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::Height(height) => {
                    config.height = Some(height);
                }
                ConfigChange::Width(width) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.width = Some(width);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.width = Some(width);
                    }
                }
                ConfigChange::Label(label) => {
                    config.label = Some(label);
                }
                ConfigChange::Mode(mode) => {
                    config.mode = Some(mode);
                }
                ConfigChange::FontSize(size) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.font_size = Some(size);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.font_size = Some(size);
                    }
                }
                ConfigChange::FontFamily(font_name) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.font_family = Some(font_name);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.font_family = Some(font_name);
                    }
                }
                ConfigChange::BackgroundColor(color) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.background = Some(color);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.background = Some(color);
                    }
                }
                ConfigChange::FocusedTextColor(color) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.focused_text = Some(color);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.focused_text = Some(color);
                    }
                }
                ConfigChange::UnfocusedTextColor(color) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.unfocused_text = Some(color);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.unfocused_text = Some(color);
                    }
                }
            },
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: Option<&'a StackbarConfig>) -> Element<'a, Message> {
        let config = if let Some(config) = config {
            config
        } else {
            default_stackbar_config_ref()
        };
        opt_helpers::section_view(
            "Stackbar:",
            [
                opt_helpers::number(
                    "Stackbar Height",
                    Some(""),
                    config.height.unwrap_or(40),
                    |value| Message::ConfigChange(ConfigChange::Height(value)),
                ),
                opt_helpers::choose(
                    "Stackbar Label",
                    Some(""),
                    [StackbarLabel::Process, StackbarLabel::Title],
                    Some(config.label.unwrap_or(StackbarLabel::Process)),
                    |selected| Message::ConfigChange(ConfigChange::Label(selected)),
                ),
                opt_helpers::choose(
                    "Stackbar Mode",
                    Some(""),
                    [
                        StackbarMode::OnStack,
                        StackbarMode::Always,
                        StackbarMode::Never,
                    ],
                    Some(config.mode.unwrap_or(StackbarMode::OnStack)),
                    |selected| Message::ConfigChange(ConfigChange::Mode(selected)),
                ),
                opt_helpers::number(
                    "Stackbar Tabs Width",
                    Some(""),
                    config.tabs.as_ref().and_then(|t| t.width).unwrap_or(200),
                    |value| Message::ConfigChange(ConfigChange::Width(value)),
                ),
                opt_helpers::input(
                    "Stackbar Font Family",
                    Some(""),
                    "",
                    config
                        .tabs
                        .as_ref()
                        .map(|t| t.font_family.as_ref().map_or("", |v| v))
                        .unwrap_or(""),
                    |value| Message::ConfigChange(ConfigChange::FontFamily(value)),
                    None,
                ),
                opt_helpers::number(
                    "Stackbar Font Size",
                    Some(""),
                    config.tabs.as_ref().and_then(|t| t.font_size).unwrap_or(0),
                    |value| Message::ConfigChange(ConfigChange::FontSize(value)),
                ),
            ],
        )
    }
}

pub fn default_stackbar_config() -> StackbarConfig {
    StackbarConfig {
        height: None,
        label: None,
        mode: None,
        tabs: None,
    }
}

pub fn default_stackbar_config_ref() -> &'static StackbarConfig {
    &StackbarConfig {
        height: None,
        label: None,
        mode: None,
        tabs: None,
    }
}

pub fn default_tabs_config() -> TabsConfig {
    TabsConfig {
        width: Some(200),
        focused_text: Some(Colour::Rgb(Rgb::from(16777215))),
        unfocused_text: Some(Colour::Rgb(Rgb::from(11776947))),
        background: Some(Colour::Rgb(Rgb::from(3355443))),
        font_family: None,
        font_size: Some(0),
    }
}
