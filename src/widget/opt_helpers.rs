#![allow(dead_code)]
use super::{
    color_picker::{color_picker, HexString},
    expandable::Expandable,
    icons, number_input, opt_button as opt_button_internal, ICONS,
};

use crate::{widget, BOLD_FONT, EMOJI_FONT};

use std::fmt::Display;
use std::str::FromStr;

use iced::{
    border, padding,
    widget::{
        button, checkbox, column, combo_box, container, horizontal_rule, mouse_area, pick_list,
        row, scrollable, text, toggler, Button, Column, Container, Row, Text,
    },
    Background, Center, Color, Element, Fill,
};
use num_traits::{Bounded, Num, NumAssignOps};

pub struct DisableArgs<'a, Message> {
    pub disable: bool,
    pub label: Option<&'a str>,
    pub on_toggle: fn(bool) -> Message,
}

pub fn opt_box_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();
    let background = if palette.is_dark {
        Some(palette.background.weak.color.scale_alpha(0.35).into())
    } else {
        Some(iced::Color::BLACK.scale_alpha(0.01).into())
    };
    let border = if palette.is_dark {
        iced::border::rounded(5)
    } else {
        iced::border::rounded(5)
            .width(1)
            .color(palette.background.weak.color)
    };
    container::Style {
        background,
        border,
        ..container::Style::default()
    }
}

pub fn opt_box_style_top(theme: &iced::Theme) -> container::Style {
    container::Style {
        border: iced::Border {
            radius: iced::border::top(5),
            ..opt_box_style(theme).border
        },
        ..opt_box_style(theme)
    }
}

pub fn opt_box_style_bottom(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();
    let background = if palette.is_dark {
        Some(palette.background.weak.color.scale_alpha(0.15).into())
    } else {
        Some(iced::Color::BLACK.scale_alpha(0.05).into())
    };
    container::Style {
        background,
        border: iced::Border {
            radius: iced::border::bottom(5),
            ..opt_box_style(theme).border
        },
        ..opt_box_style(theme)
    }
}

pub fn opt_box<'a, Message: 'a>(
    element: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    container(element).padding(10).style(opt_box_style)
}

pub fn reset_button<'a, Message>(message: Message) -> Button<'a, Message> {
    button(icons::back().size(13).style(|t: &iced::Theme| text::Style {
        color: Some(t.extended_palette().primary.strong.color),
    }))
    .on_press(message)
    .padding(padding::all(2.5))
    .style(|t, s| {
        if matches!(s, button::Status::Hovered) {
            button::secondary(t, button::Status::Active)
        } else {
            button::text(t, s)
        }
    })
}

pub fn disable_checkbox<'a, Message: Clone + 'a>(
    disable_args: Option<&DisableArgs<'a, Message>>,
) -> Option<Element<'a, Message>> {
    disable_args.map(|args| {
        mouse_area(
            row![
                text(args.label.unwrap_or_default()),
                checkbox("", args.disable)
                    .spacing(0)
                    .on_toggle(args.on_toggle)
            ]
            .spacing(10),
        )
        .on_press((args.on_toggle)(!args.disable))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
    })
}

pub fn to_description_text(t: Text) -> Text {
    t.style(|t: &iced::Theme| {
        let palette = t.extended_palette();
        let color = if palette.is_dark {
            Some(palette.secondary.strong.color)
        } else {
            Some(palette.background.base.text.scale_alpha(0.75))
        };
        text::Style { color }
    })
    .size(13)
    .wrapping(text::Wrapping::WordOrGlyph)
}

pub fn description_text(s: &str) -> Text {
    to_description_text(text(s))
}

///Creates a column with a label element and a description
///
///If `Some(description)` is given, it adds the description below the name.
pub fn label_element_with_description<'a, Message: 'a>(
    label_el: impl Into<Element<'a, Message>> + 'a,
    description: Option<&'a str>,
) -> Element<'a, Message> {
    column![label_el.into()]
        .push_maybe(description.map(description_text))
        .width(Fill)
        .spacing(5)
        .into()
}

///Creates a column with a label and a description
///
///If `Some(description)` is given, it adds the description below the name.
pub fn label_with_description<'a, Message: 'a>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
) -> Element<'a, Message> {
    label_element_with_description(widget::label(name), description)
}

///Creates a `button` with `name` as label and with a custom element as the button itself.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn opt_custom_button<'a, Message: Clone + 'a, F, I>(
    name: impl text::IntoFragment<'a>,
    description: Option<&'a str>,
    on_press: Message,
    element: F,
) -> Element<'a, Message>
where
    F: Fn(bool) -> I + 'a,
    I: Into<Element<'a, Message>> + 'a,
{
    opt_button_internal::OptButton::with(name, description, on_press, element).into()
}

// fn h<Message: Clone>(hovered: bool) -> impl Into<Element<'_, Message>> {}

///Creates a `button` with `name` as label.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn opt_button<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    on_press: Message,
) -> Element<'a, Message> {
    opt_custom_button(name, description, on_press.clone(), move |hovered| {
        button(text("›").font(*EMOJI_FONT).size(25))
            .on_press(on_press.clone())
            .padding(padding::left(10).right(10))
            .style(move |t, s| {
                if hovered {
                    button::secondary(t, button::Status::Active)
                } else {
                    button::text(t, s)
                }
            })
    })
}

///Creates a `button` with `name` as label with "Delete", "Move Up" and "Move Down" buttons.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn opt_button_add_move<'a, Message: Clone + 'a>(
    name: impl text::IntoFragment<'a>,
    description: Option<&'a str>,
    show_delete: bool,
    show_up: bool,
    show_down: bool,
    on_press: Message,
    on_delete: Message,
    on_add_up: Message,
    on_add_down: Message,
    on_move_up: Message,
    on_move_down: Message,
) -> Element<'a, Message> {
    let on_press_c = on_press.clone();
    let element = move |hovered| {
        let right_button = container(
            button(text("›").font(*EMOJI_FONT).size(25))
                .on_press(on_press_c.clone())
                .padding(padding::left(10).right(10))
                .style(move |t, s| {
                    if hovered {
                        button::secondary(t, button::Status::Active)
                    } else {
                        button::text(t, s)
                    }
                }),
        )
        .padding(padding::left(5));

        let add_buttons = Column::new()
            .push(
                button(row![text("+").size(10), icons::level_up().size(10)].spacing(2.5))
                    .on_press(on_add_up.clone())
                    .padding(padding::left(5).right(5)),
            )
            .push(
                button(row![text("+").size(10), icons::level_down().size(10)].spacing(2.5))
                    .on_press(on_add_down.clone())
                    .padding(padding::left(5).right(5)),
            )
            .spacing(2.5);

        let delete_button = Column::new().push_maybe(
            show_delete.then_some(
                button(icons::delete().size(18))
                    .on_press(on_delete.clone())
                    .padding(padding::left(5).right(5))
                    .style(button::danger),
            ),
        );

        let move_buttons = Column::new()
            .push_maybe(
                show_up.then_some(
                    button(icons::up_chevron().size(10))
                        .on_press(on_move_up.clone())
                        .style(button::secondary)
                        .padding(padding::left(5).right(5)),
                ),
            )
            .push_maybe(
                show_down.then_some(
                    button(icons::down_chevron().size(10))
                        .on_press(on_move_down.clone())
                        .style(button::secondary)
                        .padding(padding::left(5).right(5)),
                ),
            )
            .spacing(2.5);

        let align_buttons = |el| {
            container(el)
                .align_y(match (show_up, show_down) {
                    (true, false) => iced::Top,
                    (false, true) => iced::Bottom,
                    _ => iced::Center.into(),
                })
                .height(Fill)
        };

        let element = row![
            align_buttons(add_buttons),
            delete_button,
            align_buttons(move_buttons),
            right_button
        ]
        .spacing(10)
        .height(iced::Shrink)
        .align_y(Center);

        element
    };

    opt_custom_button(name, description, on_press.clone(), element)
}

// #[allow(clippy::too_many_arguments)]
// pub fn opt_button_add_move<'a, Message: Clone + 'a>(
//     name: &'a str,
//     description: Option<&'a str>,
//     _hovered: bool,
//     show_delete: bool,
//     show_up: bool,
//     show_down: bool,
//     on_press: Message,
//     on_delete: Message,
//     on_add_up: Message,
//     on_add_down: Message,
//     on_move_up: Message,
//     on_move_down: Message,
//     _on_hover: impl Fn(bool) -> Message,
// ) -> Element<'a, Message> {
//     let on_press_c = on_press.clone();
//     let element = move |hovered| {
//         let right_button = container(
//             button(text("›").font(*EMOJI_FONT).size(25))
//                 .on_press(on_press.clone())
//                 .padding(padding::left(10).right(10))
//                 .style(move |t, s| {
//                     if hovered {
//                         button::secondary(t, button::Status::Active)
//                     } else {
//                         button::text(t, s)
//                     }
//                 }),
//         )
//         .padding(padding::left(5));

//         let add_buttons = Column::new()
//             .push(
//                 button(row![text("+").size(10), icons::level_up().size(10)].spacing(2.5))
//                     .on_press(on_add_up.clone())
//                     .padding(padding::left(5).right(5)),
//             )
//             .push(
//                 button(row![text("+").size(10), icons::level_down().size(10)].spacing(2.5))
//                     .on_press(on_add_down.clone())
//                     .padding(padding::left(5).right(5)),
//             )
//             .spacing(2.5);

//         let delete_button = Column::new().push_maybe(
//             show_delete.then_some(
//                 button(icons::delete().size(18))
//                     .on_press(on_delete.clone())
//                     .padding(padding::left(5).right(5))
//                     .style(button::danger),
//             ),
//         );

//         let move_buttons = Column::new()
//             .push_maybe(
//                 show_up.then_some(
//                     button(icons::up_chevron().size(10))
//                         .on_press(on_move_up.clone())
//                         .style(button::secondary)
//                         .padding(padding::left(5).right(5)),
//                 ),
//             )
//             .push_maybe(
//                 show_down.then_some(
//                     button(icons::down_chevron().size(10))
//                         .on_press(on_move_down.clone())
//                         .style(button::secondary)
//                         .padding(padding::left(5).right(5)),
//                 ),
//             )
//             .spacing(2.5);

//         let align_buttons = |el| {
//             container(el)
//                 .align_y(match (show_up, show_down) {
//                     (true, false) => iced::Top,
//                     (false, true) => iced::Bottom,
//                     _ => iced::Center.into(),
//                 })
//                 .height(Fill)
//         };

//         row![
//             align_buttons(add_buttons),
//             delete_button,
//             align_buttons(move_buttons),
//             right_button
//         ]
//         .spacing(10)
//         .height(iced::Shrink)
//         .align_y(Center)
//     };

//     opt_custom_button(name, description, on_press_c, _on_hover, element)
// }

///Creates a row with a label with `name` and a `text_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn input<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    placeholder: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
) -> Element<'a, Message> {
    let element = row![
        label_with_description(name, description),
        widget::input(placeholder, value, on_change, on_submit),
    ]
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `text_input` and a disable checkbox which allows
///toggling the input on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn input_with_disable<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    placeholder: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + Clone + 'a,
    on_submit: Option<Message>,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_input_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_change.clone());
    let element = row![label_with_description(name, description),]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            widget::input(placeholder, value, on_change, on_submit).on_input_maybe(on_input_maybe),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `text_input` and a disable checkbox which allows
///toggling the input on/off.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn input_with_disable_default<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    placeholder: &'a str,
    value: &'a str,
    default_value: String,
    on_change: impl Fn(String) -> Message + Clone + 'a,
    on_submit: Option<Message>,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let is_dirty = value != default_value && !should_disable;
    let label = if is_dirty {
        row![name, reset_button(on_change(default_value))]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let on_input_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_change.clone());
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            widget::input(placeholder, value, on_change, on_submit).on_input_maybe(on_input_maybe),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name` and a `number_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number_simple<'a, T, Message: Clone + 'a>(
    value: T,
    on_change: impl Fn(T) -> Message + Clone + 'a,
) -> number_input::NumberInput<'a, T, Message>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Default + Bounded + 'a,
{
    number_input("", value).on_input(on_change)
}

///Creates a row with a label with `name` and a `number_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number<'a, T, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: T,
    on_change: impl Fn(T) -> Message + Clone + 'a,
) -> Element<'a, Message>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Default + Bounded + 'a,
{
    let element = row![
        label_with_description(name, description),
        number_simple(value, on_change),
    ]
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `number_input` and a disable checkbox which allows
///toggling the number input on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number_with_disable<'a, T, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: T,
    on_change: impl Fn(T) -> Message + Clone + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Copy + Default + Bounded + 'a,
{
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let on_change = move |v| {
        if should_disable {
            on_change(value)
        } else {
            on_change(v)
        }
    };
    let element = row![label_with_description(name, description),]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(number_input("", value).on_input_maybe((!should_disable).then_some(on_change)))
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `number_input` and a disable checkbox which allows
///toggling the number input on/off. It also adds a button in front of the label in case the value
///is diferent from `default_value` to send a message with the default value.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number_with_disable_default<'a, T, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: T,
    default_value: T,
    on_change: impl Fn(T) -> Message + Clone + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Copy + Clone + Default + Bounded + 'a,
{
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let is_dirty = value != default_value && !should_disable;
    let label = if is_dirty {
        row![name, reset_button(on_change(default_value))]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let on_change = move |v| {
        if should_disable {
            on_change(value)
        } else {
            on_change(v)
        }
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(number_input("", value).on_input_maybe((!should_disable).then_some(on_change)))
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `number_input` and a disable checkbox which allows
///toggling the number input on/off. It also adds a button in front of the label in case the value
///is diferent from `default_value` to send a message with the default value.
///
///This version of `number` uses values as `Option`s to allow the default value to be `None`.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number_with_disable_default_option<'a, T, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: Option<T>,
    default_value: Option<T>,
    on_change: impl Fn(Option<T>) -> Message + Clone + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Copy + Clone + Default + Bounded + 'a,
{
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let default_value_internal = default_value.unwrap_or_default();
    let value_internal = value.unwrap_or(default_value_internal);
    let is_dirty = ((value_internal != default_value_internal)
        || (default_value.is_none() && value.is_some()))
        && !should_disable;
    let label = if is_dirty {
        row![name, reset_button(on_change(default_value))]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let on_change = move |v| {
        if should_disable {
            on_change(None)
        } else {
            on_change(Some(v))
        }
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            number_input("", value_internal).on_input_maybe((!should_disable).then_some(on_change)),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `number_input` and a disable checkbox which allows
///toggling the number input on/off. It also adds a button in front of the label in case the value
///is diferent from `default_value` to send a message with the default value.
///
///This version of `number` uses values as `Option`s to allow the default value to be `None`.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn number_with_disable_default_option_bounded<'a, T, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: Option<T>,
    max: T,
    min: T,
    default_value: Option<T>,
    on_change: impl Fn(Option<T>) -> Message + Clone + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Copy + Clone + Default + Bounded + 'a,
{
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let default_value_internal = default_value.unwrap_or_default();
    let value_internal = value.unwrap_or(default_value_internal);
    let is_dirty = ((value_internal != default_value_internal)
        || (default_value.is_none() && value.is_some()))
        && !should_disable;
    let label = if is_dirty {
        row![name, reset_button(on_change(default_value))]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let on_change = move |v| {
        if should_disable {
            on_change(None)
        } else {
            on_change(Some(v))
        }
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            number_input("", value_internal)
                .max(max)
                .min(min)
                .on_input_maybe((!should_disable).then_some(on_change)),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name` and a `color_picker`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn color_picker_simple<'a, Message: Clone + 'a, F>(
    show_picker: bool,
    color: Color,
    underlay: Element<'a, Message>,
    on_cancel: Message,
    on_submit: F,
) -> Element<'a, Message>
where
    F: Fn(Color) -> Message + 'a,
{
    color_picker(show_picker, color, underlay, on_cancel, on_submit).into()
}

#[allow(clippy::too_many_arguments)]
pub fn color<'a, Message: Clone + 'a, F>(
    name: &'a str,
    description: Option<&'a str>,
    show_picker: bool,
    color: Option<Color>,
    default_color: Option<Color>,
    on_toggle: impl Fn(bool) -> Message,
    on_submit: F,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    F: Fn(Option<Color>) -> Message + 'a,
{
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let default_color_internal = default_color.unwrap_or_default();
    let color_internal = color.unwrap_or(default_color_internal);
    let is_dirty = ((color_internal != default_color_internal)
        || (default_color.is_none() && color.is_some()))
        && !should_disable;
    let label = if is_dirty {
        row![name, reset_button(on_submit(default_color))]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let on_press = if should_disable {
        None
    } else {
        Some(on_toggle(true))
    };
    let on_submit_internal = move |v| on_submit(Some(v));
    let underlay = button(text(color_internal.as_hex_string()))
        .on_press_maybe(on_press)
        .style(move |t, s| button::Style {
            background: Some(color_internal.into()),
            text_color: if color_internal.r.max(color_internal.g.max(color_internal.b)) < 0.5 {
                Color::WHITE
            } else {
                Color::BLACK
            },
            ..button::secondary(t, s)
        })
        .into();
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(color_picker_simple(
            show_picker,
            color_internal,
            underlay,
            on_toggle(false),
            on_submit_internal,
        ))
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a `checkbox` with `name` as label
///
///If `Some(description)` is given, it adds the description below the label.
pub fn bool<'a, Message: 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let element = row![
        label_with_description(name, description),
        checkbox(if value { "On" } else { "Off" }, value).on_toggle(on_toggle),
    ]
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates a `checkbox` with `name` as label and a disable checkbox which allows
///toggling the bool on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn bool_with_disable<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_toggle_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_toggle);
    let element = row![label_with_description(name, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(checkbox(if value { "On" } else { "Off" }, value).on_toggle_maybe(on_toggle_maybe))
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a `toggler` with `name` as label
///
///If `Some(description)` is given, it adds the description below the label.
pub fn toggle<'a, Message: 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let element = row![
        label_with_description(name, description),
        toggler(value)
            .label(if value { "On" } else { "Off" })
            .on_toggle(on_toggle),
    ]
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates a `toggler` with `name` as label and a disable checkbox which allows
///toggling the toggler on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn toggle_with_disable<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_toggle_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_toggle);
    let element = row![label_with_description(name, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            toggler(value)
                .label(if value { "On" } else { "Off" })
                .on_toggle_maybe(on_toggle_maybe),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a `toggler` with `name` as label and a disable checkbox which allows
///toggling the toggler on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn toggle_with_disable_default_no_option<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    default_value: bool,
    on_toggle: impl Fn(bool) -> Message + Clone + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_toggle_c = on_toggle.clone();
    let on_toggle_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_toggle);
    let is_dirty = value != default_value;
    let label = if is_dirty {
        let on_default = (on_toggle_c)(default_value);
        row![name, reset_button(on_default)]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            toggler(value)
                .label(if value { "On" } else { "Off" })
                .on_toggle_maybe(on_toggle_maybe),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a `toggler` with `name` as label and a disable checkbox which allows
///toggling the toggler on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn toggle_with_disable_default<'a, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: Option<bool>,
    default_value: Option<bool>,
    on_toggle: impl Fn(Option<bool>) -> Message + Clone + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_toggle_c = on_toggle.clone();
    let on_toggle_maybe = (!matches!(&disable_args, Some(args) if args.disable))
        .then_some(move |v| on_toggle(Some(v)));
    let is_dirty = if let (Some(v), Some(df)) = (&value, &default_value) {
        v != df
    } else {
        !matches!((&value, &default_value), (None, None))
    };
    let value = value.unwrap_or_default();
    let label = if is_dirty {
        let on_default = (on_toggle_c)(default_value.as_ref().cloned());
        row![name, reset_button(on_default)]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(
            toggler(value)
                .label(if value { "On" } else { "Off" })
                .on_toggle_maybe(on_toggle_maybe),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name`.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn choose<'a, T, V, L, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    V: std::borrow::Borrow<T> + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
{
    let element = Row::new()
        .spacing(10)
        .align_y(Center)
        .push_maybe((!name.is_empty()).then_some(label_with_description(name, description)))
        .push(pick_list(options, selected, on_selected));
    opt_box(element).into()
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name`. And adds a disable
///checkbox which allows toggling the choose on/off.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn choose_with_disable<'a, T, V, L, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    V: std::borrow::Borrow<T> + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
{
    let element = Row::new()
        .spacing(10)
        .align_y(Center)
        .push_maybe((!name.is_empty()).then_some(label_with_description(name, description)))
        .push_maybe(disable_checkbox(disable_args.as_ref()))
        .push(pick_list(options, selected, on_selected));
    opt_box(element).into()
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name`. And adds a disable
///checkbox which allows toggling the choose on/off.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn choose_with_disable_default<'a, T, V, L, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    options_descriptions: Vec<Element<'a, Message>>,
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(Option<T>) -> Message + 'a,
    default_value: Option<V>,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    V: std::borrow::Borrow<T> + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
{
    let is_dirty = if let (Some(v), Some(df)) = (&selected, &default_value) {
        v.borrow() != df.borrow()
    } else {
        !matches!((&selected, &default_value), (None, None))
    };
    let label = if is_dirty {
        let on_default = (on_selected)(default_value.as_ref().map(|df| df.borrow()).cloned());
        row![name, reset_button(on_default)]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let selected_description: Element<'a, Message> = (|| {
        if !options_descriptions.is_empty() {
            if let Some(ref selected) = selected {
                if let Some(i) = (options.borrow() as &[T])
                    .iter()
                    .position(|v| v == selected.borrow())
                {
                    if let Some((_, d)) = options_descriptions
                        .into_iter()
                        .enumerate()
                        .find(|(idx, _)| i == *idx)
                    {
                        return d;
                    }
                }
            }
        }
        iced::widget::Space::new(iced::Shrink, iced::Shrink).into()
    })();
    let element = row![column![
        label_element_with_description(label, description),
        selected_description
    ]
    .spacing(10)]
    .push_maybe(disable_checkbox(disable_args.as_ref()))
    .push(
        pick_list(options, selected, move |v| on_selected(Some(v)))
            .font(ICONS)
            .text_shaping(text::Shaping::Advanced),
    )
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name`. And adds a disable
///checkbox which allows toggling the choose on/off.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn choose_with_disable_default_bg<'a, T, V, L, Message: Clone + 'a>(
    name: &'a str,
    description: Option<&'a str>,
    options_descriptions: Vec<Element<'a, Message>>,
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(Option<T>) -> Message + 'a,
    default_value: Option<V>,
    disable_args: Option<DisableArgs<'a, Message>>,
    bg_color: Color,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    V: std::borrow::Borrow<T> + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
{
    let is_dirty = if let (Some(v), Some(df)) = (&selected, &default_value) {
        v.borrow() != df.borrow()
    } else {
        !matches!((&selected, &default_value), (None, None))
    };
    let label = if is_dirty {
        let on_default = (on_selected)(default_value.as_ref().map(|df| df.borrow()).cloned());
        row![name, reset_button(on_default)]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let selected_description: Element<'a, Message> = (|| {
        if !options_descriptions.is_empty() {
            if let Some(ref selected) = selected {
                if let Some(i) = (options.borrow() as &[T])
                    .iter()
                    .position(|v| v == selected.borrow())
                {
                    if let Some((_, d)) = options_descriptions
                        .into_iter()
                        .enumerate()
                        .find(|(idx, _)| i == *idx)
                    {
                        return d;
                    }
                }
            }
        }
        iced::widget::Space::new(iced::Shrink, iced::Shrink).into()
    })();

    // Calculate text_color according to bg_color. Based on this stackoverflow answer:
    // https://stackoverflow.com/a/3943023
    let linear_bg = bg_color.into_linear();
    let luminance = 0.2126 * linear_bg[0] + 0.7152 * linear_bg[1] + 0.0722 * linear_bg[2];
    let text_color = if luminance > 0.179 {
        Color::BLACK
    } else {
        Color::WHITE
    };

    let element = row![column![
        label_element_with_description(label, description),
        selected_description
    ]
    .spacing(10)]
    .push_maybe(disable_checkbox(disable_args.as_ref()))
    .push(
        pick_list(options, selected, move |v| on_selected(Some(v)))
            .font(ICONS)
            .style(move |t, s| pick_list::Style {
                background: bg_color.into(),
                text_color,
                ..pick_list::default(t, s)
            })
            .text_shaping(text::Shaping::Advanced),
    )
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates a `combo_box`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name`. And adds a disable
///checkbox which allows toggling the choose on/off.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn combo_with_disable_default<'a, T, Message: Clone + 'a>(
    name: &'a str,
    placeholder: &'a str,
    description: Option<&'a str>,
    options_descriptions: Vec<Element<'a, Message>>,
    options: &'a combo_box::State<T>,
    selected: Option<T>,
    on_selected: impl Fn(Option<T>) -> Message + 'static,
    default_value: Option<T>,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    T: Display + PartialEq + Clone + 'static,
{
    let is_dirty = if let (Some(v), Some(df)) = (&selected, &default_value) {
        v != df
    } else {
        !matches!((&selected, &default_value), (None, None))
    };
    let label = if is_dirty {
        let on_default = (on_selected)(default_value.clone());
        row![name, reset_button(on_default)]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name].height(30).align_y(Center)
    };
    let selected_description: Element<'a, Message> = (|| {
        if !options_descriptions.is_empty() {
            if let Some(ref selected) = selected {
                if let Some(i) = options.options().iter().position(|v| v == selected) {
                    if let Some((_, d)) = options_descriptions
                        .into_iter()
                        .enumerate()
                        .find(|(idx, _)| i == *idx)
                    {
                        return d;
                    }
                }
            }
        }
        iced::widget::Space::new(iced::Shrink, iced::Shrink).into()
    })();
    let element = row![column![
        label_element_with_description(label, description),
        selected_description
    ]
    .spacing(10)]
    .push_maybe(disable_checkbox(disable_args.as_ref()))
    .push(
        combo_box(options, placeholder, selected.as_ref(), move |v| {
            on_selected(Some(v))
        })
        .width(250),
    )
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates an expandable option with children options to be shown when expanded.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn expandable<'a, Message: Clone + 'a, I>(
    name: impl text::IntoFragment<'a>,
    description: Option<&'a str>,
    children: impl Fn() -> I + 'a,
    is_dirty: bool,
    on_default: Message,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    I: IntoIterator<Item = Element<'a, Message>> + 'a,
{
    let right_button = move |hovered: bool, expanded: bool| {
        container(if expanded {
            text("▲").size(10)
        } else {
            text("▼").size(10)
        })
        .padding(padding::all(5).left(10).right(10))
        .style(move |t: &iced::Theme| {
            let palette = t.extended_palette();
            let background = if hovered {
                // Similar to `button::secondary`
                Some(Background::Color(palette.secondary.strong.color))
            } else {
                // Similar to `button::text`
                None
            };
            let text_color = if hovered {
                // Similar to `button::secondary`
                Some(palette.secondary.base.text)
            } else {
                // Similar to `button::text`
                None
            };
            // Use a container style to emulate a button, that changes from
            // `button::text` to `button::secondary` when hovered
            container::Style {
                background,
                text_color,
                border: border::rounded(2),
                shadow: Default::default(),
            }
        })
        .into()
    };

    expandable_custom(
        name,
        description,
        right_button,
        children,
        is_dirty,
        false,
        on_default,
        disable_args,
    )
}

///Creates an expandable option with a custom element to the right of the title name and
///description, with children options to be shown when expanded.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn expandable_custom<'a, Message: Clone + 'a, I>(
    name: impl text::IntoFragment<'a>,
    description: Option<&'a str>,
    right_element: impl Fn(bool, bool) -> Element<'a, Message> + 'a,
    children: impl Fn() -> I + 'a,
    is_dirty: bool,
    force_expand: bool,
    on_default: Message,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message>
where
    I: IntoIterator<Item = Element<'a, Message>> + 'a,
{
    Expandable::with(
        Some(name),
        description,
        None,
        Some(on_default),
        is_dirty,
        force_expand,
        disable_args,
        right_element,
        children,
    )
    .into()
}

///Creates a row of options within a section. This is a simple helper to create
///a row with the predefined spacing. It can have more than one option on it,
///for example if you want to have multiple checkboxes side by side, you should
///put them all on one section_row.
pub fn section_row<'a, Message: 'a>(
    contents: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    Row::with_children(contents).spacing(10).into()
}

///Creates the view for a sub section of options with `title` and the `contents` to be inserted on
///a section_view.
pub fn sub_section_view<'a, Message: 'a>(
    title: Element<'a, Message>,
    contents: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        title,
        horizontal_rule(2.0),
        Column::with_children(contents)
            .padding(padding::top(10).bottom(10))
            .spacing(10),
    ]
    .spacing(10)
    .into()
}

///Creates the view for a section of options with `title` and the `contents`.
pub fn section_view<'a, Message: 'a>(
    title: impl Into<Text<'a>>,
    contents: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let section_title: Text = (title.into() as Text).size(20.0).font(*BOLD_FONT);
    column![
        section_title,
        horizontal_rule(2.0),
        scrollable(
            Column::with_children(contents)
                .padding(padding::top(10).bottom(10).right(20))
                .spacing(10)
                .width(Fill)
        )
    ]
    .spacing(10)
    .into()
}
