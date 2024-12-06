use crate::widget::opt_helpers;

use iced::{Element, Task};
use komorebi::{
    border_manager::ZOrder, BorderColours, BorderImplementation, BorderStyle, Colour, Rgb,
    StackbarLabel, StackbarMode, StaticConfig,
};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleSinglePicker(bool),
    ToggleMonoclePicker(bool),
    ToggleUnfocusedPicker(bool),
    ToggleFloatingPicker(bool),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    Border(bool),
    BorderWidth(i32),
    BorderOffset(i32),
    BorderStyle(BorderStyle),
    BorderImplementation(BorderImplementation),
    BorderZOrder(ZOrder),
    SingleColor(Option<iced::Color>),
    MonocleColor(Option<iced::Color>),
    UnfocusedColor(Option<iced::Color>),
    FloatingColor(Option<iced::Color>),
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
}

#[derive(Debug)]
pub struct BorderConfig<'a> {
    pub border: Option<&'a bool>,
    pub border_colours: Option<&'a BorderColours>,
    pub border_implementation: Option<&'a BorderImplementation>,
    pub border_style: Option<&'a BorderStyle>,
    pub border_width: Option<&'a i32>,
    pub border_offset: Option<&'a i32>,
    pub border_z_order: Option<&'a ZOrder>,
}

#[derive(Debug)]
pub struct BorderConfigMut<'a> {
    pub border: Option<&'a mut bool>,
    pub border_colours: Option<&'a mut BorderColours>,
    pub border_implementation: Option<&'a mut BorderImplementation>,
    pub border_style: Option<&'a mut BorderStyle>,
    pub border_width: Option<&'a mut i32>,
    pub border_offset: Option<&'a mut i32>,
    pub border_z_order: Option<&'a mut ZOrder>,
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
            Message::ConfigChange(change) => match change {
                ConfigChange::Border(_) => todo!(),
                ConfigChange::BorderWidth(_) => todo!(),
                ConfigChange::BorderOffset(_) => todo!(),
                ConfigChange::BorderStyle(border_style) => todo!(),
                ConfigChange::BorderImplementation(border_implementation) => todo!(),
                ConfigChange::BorderZOrder(zorder) => todo!(),
                ConfigChange::SingleColor(color) => todo!(),
                ConfigChange::MonocleColor(color) => todo!(),
                ConfigChange::UnfocusedColor(color) => todo!(),
                ConfigChange::FloatingColor(color) => todo!(),
            },
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
                    Some(""),
                    *config.border.unwrap_or(&true),
                    true,
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
                opt_helpers::choose_with_disable_default(
                    "Border Style",
                    Some(""),
                    vec![],
                    [BorderStyle::System, BorderStyle::Square, BorderStyle::Rounded],
                    Some(config.border_style.map_or(BorderStyle::System, |bs| *bs)),
                    |selected| {
                        Message::ConfigChange(ConfigChange::BorderStyle(
                            selected.unwrap_or(BorderStyle::System),
                        ))
                    },
                    Some(BorderStyle::System),
                    None,
                ),
                // opt_helpers::color(
                //     "Stackbar Background Color",
                //     Some("Tab background color. (default: '0x333333')"),
                //     self.show_background_picker,
                //     config
                //         .tabs
                //         .as_ref()
                //         .and_then(|t| t.background.map(into_color)),
                //     Some(iced::color!(0x333333)),
                //     Message::ToggleSinglePicker,
                //     |v| Message::ConfigChange(ConfigChange::SingleColor(v)),
                //     None,
                // ),
                // opt_helpers::color(
                //     "Stackbar Focused Text Color",
                //     Some("Focused Tab text color. (default: '0xFFFFFF')"),
                //     self.show_focused_text_picker,
                //     config
                //         .tabs
                //         .as_ref()
                //         .and_then(|t| t.focused_text.map(into_color)),
                //     Some(iced::color!(0xffffff)),
                //     Message::ToggleMonoclePicker,
                //     |v| Message::ConfigChange(ConfigChange::MonocleColor(v)),
                //     None,
                // ),
                // opt_helpers::color(
                //     "Stackbar Unfocused Text Color",
                //     Some("Unfocused Tab text color. (default: '0xB3B3B3')"),
                //     self.show_unfocused_text_picker,
                //     config
                //         .tabs
                //         .as_ref()
                //         .and_then(|t| t.unfocused_text.map(into_color)),
                //     Some(iced::color!(0xb3b3b3)),
                //     Message::ToggleUnfocusedPicker,
                //     |v| Message::ConfigChange(ConfigChange::UnfocusedColor(v)),
                //     None,
                // ),
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
        border_z_order: config.border_z_order.as_ref(),
    }
}

fn border_config_from_static_mut(config: &mut StaticConfig) -> BorderConfigMut {
    BorderConfigMut {
        border: config.border.as_mut(),
        border_colours: config.border_colours.as_mut(),
        border_implementation: config.border_implementation.as_mut(),
        border_style: config.border_style.as_mut(),
        border_width: config.border_width.as_mut(),
        border_offset: config.border_offset.as_mut(),
        border_z_order: config.border_z_order.as_mut(),
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
        border_z_order: None,
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
