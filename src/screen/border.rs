use crate::widget::opt_helpers::description_text as t;
use crate::widget::opt_helpers;

use iced::{Element, Task};
use komorebi::{BorderColours, BorderImplementation, BorderStyle, Colour, Rgb, StaticConfig};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleSinglePicker(bool),
    ToggleMonoclePicker(bool),
    ToggleUnfocusedPicker(bool),
    ToggleFloatingPicker(bool),
    ToggleStackPicker(bool),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    Border(bool),
    BorderWidth(i32),
    BorderOffset(i32),
    BorderStyle(BorderStyle),
    BorderImplementation(BorderImplementation),
    SingleColor(Option<iced::Color>),
    MonocleColor(Option<iced::Color>),
    UnfocusedColor(Option<iced::Color>),
    FloatingColor(Option<iced::Color>),
    StackColor(Option<iced::Color>),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Debug, Default)]
pub struct Border {
    pub show_single_picker: bool,
    pub show_monocle_picker: bool,
    pub show_unfocused_picker: bool,
    pub show_floating_picker: bool,
    pub show_stack_picker: bool,
}

#[derive(Debug)]
pub struct BorderConfig<'a> {
    pub border: Option<&'a bool>,
    pub border_colours: Option<&'a BorderColours>,
    pub border_implementation: Option<&'a BorderImplementation>,
    pub border_style: Option<&'a BorderStyle>,
    pub border_width: Option<&'a i32>,
    pub border_offset: Option<&'a i32>,
}

#[derive(Debug)]
pub struct BorderConfigMut<'a> {
    pub border: &'a mut Option<bool>,
    pub border_colours: &'a mut Option<BorderColours>,
    pub border_implementation: &'a mut Option<BorderImplementation>,
    pub border_style: &'a mut Option<BorderStyle>,
    pub border_width: &'a mut Option<i32>,
    pub border_offset: &'a mut Option<i32>,
}

// impl Default for BorderConfig {
//     fn default() -> Self {
//         Self {
//             border: Some(false),
//             border_colours: Some(BorderColours {
//                 single: Some(Colour::Rgb(Rgb::new(66, 165, 245))),
//                 stack: Some(Colour::Rgb(Rgb::new(0, 165, 66))),
//                 monocle: Some(Colour::Rgb(Rgb::new(255, 51, 153))),
//                 floating: Some(Colour::Rgb(Rgb::new(245, 245, 165))),
//                 unfocused: Some(Colour::Rgb(Rgb::new(128, 128, 128))),
//             }),
//             border_implementation: Some(BorderImplementation::default()),
//             border_style: Some(BorderStyle::default()),
//             border_width: Some(8),
//             border_offset: Some(-1),
//             border_z_order: None,
//         }
//     }
// }

impl Border {
    pub fn update(
        &mut self,
        message: Message,
        config: &mut StaticConfig,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigChange(change) => {
                let config = border_config_from_static_mut(config);
                match change {
                    ConfigChange::Border(enable) => {
                        *config.border = Some(enable);
                    }
                    ConfigChange::BorderWidth(width) => {
                        *config.border_width = Some(width);
                    }
                    ConfigChange::BorderOffset(offset) => {
                        *config.border_offset = Some(offset);
                    }
                    ConfigChange::BorderStyle(border_style) => {
                        *config.border_style = Some(border_style)
                    }
                    ConfigChange::BorderImplementation(border_implementation) => {
                        *config.border_implementation = Some(border_implementation)
                    }
                    ConfigChange::SingleColor(color) => {
                        if let Some(colours) = config.border_colours {
                            colours.single = color.map(from_color);
                        } else {
                            *config.border_colours = Some(BorderColours {
                                single: color.map(from_color),
                                stack: None,
                                monocle: None,
                                floating: None,
                                unfocused: None,
                            });
                        }
                        self.show_single_picker = false;
                    }
                    ConfigChange::MonocleColor(color) => {
                        if let Some(colours) = config.border_colours {
                            colours.monocle = color.map(from_color);
                        } else {
                            *config.border_colours = Some(BorderColours {
                                single: None,
                                stack: None,
                                monocle: color.map(from_color),
                                floating: None,
                                unfocused: None,
                            });
                        }
                        self.show_monocle_picker = false;
                    }
                    ConfigChange::UnfocusedColor(color) => {
                        if let Some(colours) = config.border_colours {
                            colours.unfocused = color.map(from_color);
                        } else {
                            *config.border_colours = Some(BorderColours {
                                single: None,
                                stack: None,
                                monocle: None,
                                floating: None,
                                unfocused: color.map(from_color),
                            });
                        }
                        self.show_unfocused_picker = false;
                    }
                    ConfigChange::FloatingColor(color) => {
                        if let Some(colours) = config.border_colours {
                            colours.floating = color.map(from_color);
                        } else {
                            *config.border_colours = Some(BorderColours {
                                single: None,
                                stack: None,
                                monocle: None,
                                floating: color.map(from_color),
                                unfocused: None,
                            });
                        }
                        self.show_floating_picker = false;
                    }
                    ConfigChange::StackColor(color) => {
                        if let Some(colours) = config.border_colours {
                            colours.stack = color.map(from_color);
                        } else {
                            *config.border_colours = Some(BorderColours {
                                single: None,
                                stack: color.map(from_color),
                                monocle: None,
                                floating: None,
                                unfocused: None,
                            });
                        }
                        self.show_stack_picker = false;
                    }
                }
            }
            Message::ToggleSinglePicker(show) => {
                self.show_single_picker = show;
            }
            Message::ToggleMonoclePicker(show) => {
                self.show_monocle_picker = show;
            }
            Message::ToggleUnfocusedPicker(show) => {
                self.show_unfocused_picker = show;
            }
            Message::ToggleFloatingPicker(show) => {
                self.show_floating_picker = show;
            }
            Message::ToggleStackPicker(show) => {
                self.show_stack_picker = show;
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: Option<&'a StaticConfig>) -> Element<'a, Message> {
        let config = if let Some(config) = config {
            border_config_from_static(config)
        } else {
            default_border_config()
        };
        opt_helpers::section_view(
            "Border:",
            [
                opt_helpers::toggle_with_disable_default_no_option(
                    "Enable Border",
                    Some("Display an active window border (default: false)"),
                    *config.border.unwrap_or(&false),
                    false,
                    |v| Message::ConfigChange(ConfigChange::Border(v)),
                    None,
                ),
                opt_helpers::number_with_disable_default(
                    "Border Width",
                    Some("Width of the window border. (default: 8)"),
                    *config.border_width.unwrap_or(&8),
                    8,
                    |value| Message::ConfigChange(ConfigChange::BorderWidth(value)),
                    None,
                ),
                opt_helpers::number_with_disable_default(
                    "Border Offset",
                    Some("Offset of the window border (default: -1)"),
                    *config.border_offset.unwrap_or(&-1),
                    -1,
                    |value| Message::ConfigChange(ConfigChange::BorderOffset(value)),
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Border Style",
                    Some("Active window border style (default: System)"),
                    vec![],
                    [
                        BorderStyle::System,
                        BorderStyle::Square,
                        BorderStyle::Rounded,
                    ],
                    Some(config.border_style.map_or(BorderStyle::System, |bs| *bs)),
                    |selected| {
                        Message::ConfigChange(ConfigChange::BorderStyle(
                            selected.unwrap_or(BorderStyle::System),
                        ))
                    },
                    Some(BorderStyle::System),
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Border Implementation",
                    Some("Active window border implementation (default: Komorebi)"),
                    vec![
                        t("Selected: 'Komorebi' -> Use the adjustable komorebi border implementation").into(),
                        t("Selected: 'Windows' -> Use the thin Windows accent border implementation").into(),
                    ],
                    [
                        BorderImplementation::Komorebi,
                        BorderImplementation::Windows
                    ],
                    Some(config.border_implementation.map_or(BorderImplementation::Komorebi, |bi| *bi)),
                    |selected| {
                        Message::ConfigChange(ConfigChange::BorderImplementation(
                            selected.unwrap_or(BorderImplementation::Komorebi),
                        ))
                    },
                    Some(BorderImplementation::Komorebi),
                    None,
                ),
                opt_helpers::color(
                    "Focused Window Border Color",
                    Some("Border colour when the container contains a single window and is focused"),
                    self.show_single_picker,
                    config
                        .border_colours
                        .as_ref()
                        .and_then(|bc| bc.single.map(into_color)),
                    Some(iced::color!(66, 165, 245)),
                    Message::ToggleSinglePicker,
                    |v| Message::ConfigChange(ConfigChange::SingleColor(v)),
                    None,
                ),
                opt_helpers::color(
                    "Unfocused Window Border Color",
                    Some("Border colour when the container is unfocused"),
                    self.show_unfocused_picker,
                    config
                        .border_colours
                        .as_ref()
                        .and_then(|bc| bc.unfocused.map(into_color)),
                    Some(iced::color!(128, 128, 128)),
                    Message::ToggleUnfocusedPicker,
                    |v| Message::ConfigChange(ConfigChange::UnfocusedColor(v)),
                    None,
                ),
                opt_helpers::color(
                    "Monocle Window Border Color",
                    Some("Border colour when the container is in monocle mode"),
                    self.show_monocle_picker,
                    config
                        .border_colours
                        .as_ref()
                        .and_then(|bc| bc.monocle.map(into_color)),
                    Some(iced::color!(255, 51, 153)),
                    Message::ToggleMonoclePicker,
                    |v| Message::ConfigChange(ConfigChange::MonocleColor(v)),
                    None,
                ),
                opt_helpers::color(
                    "Stack Window Border Color",
                    Some("Border colour when the container contains multiple windows and is focused"),
                    self.show_stack_picker,
                    config
                        .border_colours
                        .as_ref()
                        .and_then(|bc| bc.stack.map(into_color)),
                    Some(iced::color!(0, 165, 66)),
                    Message::ToggleStackPicker,
                    |v| Message::ConfigChange(ConfigChange::StackColor(v)),
                    None,
                ),
                opt_helpers::color(
                    "Floating Window Border Color",
                    Some("Border colour when the container is in floating mode and focused"),
                    self.show_floating_picker,
                    config
                        .border_colours
                        .as_ref()
                        .and_then(|bc| bc.floating.map(into_color)),
                    Some(iced::color!(245, 245, 165)),
                    Message::ToggleFloatingPicker,
                    |v| Message::ConfigChange(ConfigChange::FloatingColor(v)),
                    None,
                ),
            ],
        )
    }
}

fn border_config_from_static(config: &StaticConfig) -> BorderConfig {
    BorderConfig {
        border: config.border.as_ref(),
        border_colours: config.border_colours.as_ref(),
        border_implementation: config.border_implementation.as_ref(),
        border_style: config.border_style.as_ref(),
        border_width: config.border_width.as_ref(),
        border_offset: config.border_offset.as_ref(),
    }
}

fn border_config_from_static_mut(config: &mut StaticConfig) -> BorderConfigMut {
    BorderConfigMut {
        border: &mut config.border,
        border_colours: &mut config.border_colours,
        border_implementation: &mut config.border_implementation,
        border_style: &mut config.border_style,
        border_width: &mut config.border_width,
        border_offset: &mut config.border_offset,
    }
}

pub fn default_border_config() -> BorderConfig<'static> {
    BorderConfig {
        border: None,
        border_colours: None,
        border_implementation: None,
        border_style: None,
        border_width: None,
        border_offset: None,
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
