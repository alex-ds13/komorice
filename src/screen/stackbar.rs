use crate::widget::opt_helpers;

use iced::{
    widget::{column, container, horizontal_space, row, text},
    Center, Element, Fill,
    Length::Fixed,
    Task,
};
use komorebi::{Colour, Rgb, StackbarConfig, StackbarLabel, StackbarMode, TabsConfig};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleBackgroundPicker(bool),
    ToggleFocusedTextPicker(bool),
    ToggleUnfocusedTextPicker(bool),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    Height(i32),
    Label(StackbarLabel),
    Mode(StackbarMode),
    Width(i32),
    FontFamily(String),
    FontSize(i32),
    BackgroundColor(Option<iced::Color>),
    FocusedTextColor(Option<iced::Color>),
    UnfocusedTextColor(Option<iced::Color>),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Debug, Default)]
pub struct Stackbar {
    pub show_background_picker: bool,
    pub show_focused_text_picker: bool,
    pub show_unfocused_text_picker: bool,
}

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
        message: Message,
        config: &mut StackbarConfig,
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
                        config.tabs = Some(tabs);
                    }
                }
                ConfigChange::Label(label) => {
                    config.label = Some(label);
                }
                ConfigChange::Mode(mode) => {
                    config.mode = Some(mode);
                }
                ConfigChange::FontSize(size) => {
                    let size = if size <= 0 { None } else { Some(size) };
                    if let Some(tabs) = &mut config.tabs {
                        tabs.font_size = size;
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.font_size = size;
                        config.tabs = Some(tabs);
                    }
                }
                ConfigChange::FontFamily(font_name) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.font_family = Some(font_name);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.font_family = Some(font_name);
                        config.tabs = Some(tabs);
                    }
                }
                ConfigChange::BackgroundColor(color) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.background = color.map(from_color);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.background = color.map(from_color);
                        config.tabs = Some(tabs);
                    }
                    self.show_background_picker = false;
                }
                ConfigChange::FocusedTextColor(color) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.focused_text = color.map(from_color);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.focused_text = color.map(from_color);
                        config.tabs = Some(tabs);
                    }
                    self.show_focused_text_picker = false;
                }
                ConfigChange::UnfocusedTextColor(color) => {
                    if let Some(tabs) = &mut config.tabs {
                        tabs.unfocused_text = color.map(from_color);
                    } else {
                        let mut tabs = default_tabs_config();
                        tabs.unfocused_text = color.map(from_color);
                        config.tabs = Some(tabs);
                    }
                    self.show_unfocused_text_picker = false;
                }
            },
            Message::ToggleBackgroundPicker(show) => {
                self.show_background_picker = show;
            }
            Message::ToggleFocusedTextPicker(show) => {
                self.show_focused_text_picker = show;
            }
            Message::ToggleUnfocusedTextPicker(show) => {
                self.show_unfocused_text_picker = show;
            }
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
                opt_helpers::number_with_disable_default(
                    "Stackbar Height",
                    Some("Stackbar height. (default: 40)"),
                    config.height.unwrap_or(40),
                    40,
                    |value| Message::ConfigChange(ConfigChange::Height(value)),
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Stackbar Label",
                    Some("Stackbar label. (default: \"Process\")"),
                    vec![],
                    [StackbarLabel::Process, StackbarLabel::Title],
                    Some(config.label.unwrap_or(StackbarLabel::Process)),
                    |selected| {
                        Message::ConfigChange(ConfigChange::Label(
                            selected.unwrap_or(StackbarLabel::Process),
                        ))
                    },
                    Some(StackbarLabel::Process),
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Stackbar Mode",
                    Some("Stackbar mode. (default \"OnStack\")"),
                    vec![],
                    [
                        StackbarMode::OnStack,
                        StackbarMode::Always,
                        StackbarMode::Never,
                    ],
                    Some(config.mode.unwrap_or(StackbarMode::OnStack)),
                    |selected| {
                        Message::ConfigChange(ConfigChange::Mode(
                            selected.unwrap_or(StackbarMode::OnStack),
                        ))
                    },
                    Some(StackbarMode::OnStack),
                    None,
                ),
                opt_helpers::number_with_disable_default(
                    "Stackbar Tabs Width",
                    Some("Tabs width. (default: 200)"),
                    config.tabs.as_ref().and_then(|t| t.width).unwrap_or(200),
                    200,
                    |value| Message::ConfigChange(ConfigChange::Width(value)),
                    None,
                ),
                opt_helpers::input_with_disable_default(
                    "Stackbar Font Family",
                    Some("Tabs font family name. (default: empty string)\n\nWhen empty use system font."),
                    "",
                    config
                        .tabs
                        .as_ref()
                        .map(|t| t.font_family.as_ref().map_or("", |v| v))
                        .unwrap_or(""),
                    String::from(""),
                    |value| Message::ConfigChange(ConfigChange::FontFamily(value)),
                    None,
                    None,
                ),
                opt_helpers::number_with_disable_default(
                    "Stackbar Font Size",
                    Some("Tabs font size. (default: '0')\n\nWhen '0' use system font size."),
                    config.tabs.as_ref().and_then(|t| t.font_size).unwrap_or(0),
                    0,
                    |value| Message::ConfigChange(ConfigChange::FontSize(value)),
                    None,
                ),
                opt_helpers::color(
                    "Stackbar Background Color",
                    Some("Tab background color. (default: '0x333333')"),
                    self.show_background_picker,
                    config
                        .tabs
                        .as_ref()
                        .and_then(|t| t.background.map(into_color)),
                    Some(iced::color!(0x333333)),
                    Message::ToggleBackgroundPicker,
                    |v| Message::ConfigChange(ConfigChange::BackgroundColor(v)),
                    None,
                ),
                opt_helpers::color(
                    "Stackbar Focused Text Color",
                    Some("Focused Tab text color. (default: '0xFFFFFF')"),
                    self.show_focused_text_picker,
                    config
                        .tabs
                        .as_ref()
                        .and_then(|t| t.focused_text.map(into_color)),
                    Some(iced::color!(0xffffff)),
                    Message::ToggleFocusedTextPicker,
                    |v| Message::ConfigChange(ConfigChange::FocusedTextColor(v)),
                    None,
                ),
                opt_helpers::color(
                    "Stackbar Unfocused Text Color",
                    Some("Unfocused Tab text color. (default: '0xB3B3B3')"),
                    self.show_unfocused_text_picker,
                    config
                        .tabs
                        .as_ref()
                        .and_then(|t| t.unfocused_text.map(into_color)),
                    Some(iced::color!(0xb3b3b3)),
                    Message::ToggleUnfocusedTextPicker,
                    |v| Message::ConfigChange(ConfigChange::UnfocusedTextColor(v)),
                    None,
                ),
                tabs_demo(config),
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
        width: None,
        focused_text: None,
        unfocused_text: None,
        background: None,
        font_family: None,
        font_size: None,
    }
}

fn from_color(color: iced::Color) -> Colour {
    let rgba8 = color.into_rgba8();
    Colour::Rgb(Rgb {
        r: rgba8[0] as u32,
        g: rgba8[1] as u32,
        b: rgba8[2] as u32,
    })
}

fn into_color(colour: Colour) -> iced::Color {
    match colour {
        Colour::Rgb(rgb) => iced::Color::from_rgb8(rgb.r as u8, rgb.g as u8, rgb.b as u8),
        Colour::Hex(hex) => {
            let rgb = Rgb::from(hex);
            iced::Color::from_rgb8(rgb.r as u8, rgb.g as u8, rgb.b as u8)
        }
    }
}

fn tabs_demo(config: &StackbarConfig) -> Element<Message> {
    let background = config
        .tabs
        .as_ref()
        .and_then(|t| t.background.map(into_color))
        .unwrap_or(iced::color!(0x333333));

    let focused_text = config
        .tabs
        .as_ref()
        .and_then(|t| t.focused_text.map(into_color))
        .unwrap_or(iced::color!(0xffffff));

    let unfocused_text = config
        .tabs
        .as_ref()
        .and_then(|t| t.unfocused_text.map(into_color))
        .unwrap_or(iced::color!(0xb3b3b3));

    let font_size = config
        .tabs
        .as_ref()
        .and_then(|t| t.font_size.map(|fs| fs as u16))
        .unwrap_or(12);

    let tabs_width = config
        .tabs
        .as_ref()
        .and_then(|t| t.width.map(|w| w as f32))
        .unwrap_or(200.0);

    let tabs_height = config.height.map(|h| h as f32).unwrap_or(40.0);

    column![
        row!["Tabs Look:", horizontal_space()],
        row![
            container(text("Focused Tab").size(font_size))
                .width(Fixed(tabs_width))
                .height(Fixed(tabs_height))
                .align_x(Center)
                .align_y(Center)
                .style(move |t| container::Style {
                    background: Some(background.into()),
                    text_color: Some(focused_text),
                    border: iced::Border {
                        radius: (tabs_height / 2.0).into(),
                        ..container::rounded_box(t).border
                    },
                    ..container::rounded_box(t)
                }),
            container(text("Unfocused Tab").size(font_size))
                .width(Fixed(tabs_width))
                .height(Fixed(tabs_height))
                .align_x(Center)
                .align_y(Center)
                .style(move |t| container::Style {
                    background: Some(background.into()),
                    text_color: Some(unfocused_text),
                    border: iced::Border {
                        radius: (tabs_height / 2.0).into(),
                        ..container::rounded_box(t).border
                    },
                    ..container::rounded_box(t)
                }),
        ]
        .spacing(10),
    ]
    .padding(5)
    .width(Fill)
    .align_x(Center)
    .spacing(10)
    .into()
}
