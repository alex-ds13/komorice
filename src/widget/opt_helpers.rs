#![allow(dead_code)]
use super::{icons, ICONS};

use crate::{widget, BOLD_FONT, EMOJI_FONT};

use iced::{
    padding,
    widget::{
        button, checkbox, column, container, horizontal_rule, mouse_area, pick_list, row,
        scrollable, text, text_input, toggler, Button, Column, Container, Row, Text,
    },
    Center, Color, Element, Fill,
};
use iced_aw::core::color::HexString;

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

fn reset_button<'a, Message>(message: Message) -> Button<'a, Message> {
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

fn disable_checkbox<'a, Message: 'a + Clone>(
    disable_args: Option<DisableArgs<'a, Message>>,
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

pub fn num_button_style(
    t: &iced::Theme,
    _s: iced_aw::style::Status,
) -> iced_aw::number_input::number_input::Style {
    iced_aw::number_input::number_input::Style {
        button_background: Some(t.extended_palette().background.weak.color.into()),
        icon_color: t.extended_palette().background.weak.text,
    }
}

fn num_input_style(
    disable: bool,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |t: &iced::Theme, s: text_input::Status| {
        text_input::default(
            t,
            if disable {
                text_input::Status::Disabled
            } else {
                s
            },
        )
    }
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
    label_el: impl Into<Element<'a, Message>>,
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
pub fn opt_custom_button<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    on_press: Message,
    on_hover: impl Fn(bool) -> Message,
    element: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let main = row![label_with_description(name, description), element.into()]
        .align_y(Center)
        .padding(padding::right(10));

    let area = |el| {
        mouse_area(el)
            .on_press(on_press)
            .on_enter(on_hover(true))
            .on_exit(on_hover(false))
            .interaction(iced::mouse::Interaction::Pointer)
    };

    area(opt_box(main)).into()
}

///Creates a `button` with `name` as label.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn opt_button<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    hovered: bool,
    on_press: Message,
    on_hover: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    let right_button = button(text("›").font(*EMOJI_FONT).size(25))
        .on_press(on_press.clone())
        .padding(padding::left(10).right(10))
        .style(move |t, s| {
            if hovered {
                button::secondary(t, button::Status::Active)
            } else {
                button::text(t, s)
            }
        });

    opt_custom_button(name, description, on_press, on_hover, right_button)
}

///Creates a `button` with `name` as label with "Delete", "Move Up" and "Move Down" buttons.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn opt_button_add_move<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    hovered: bool,
    show_delete: bool,
    show_up: bool,
    show_down: bool,
    on_press: Message,
    on_delete: Message,
    on_add_up: Message,
    on_add_down: Message,
    on_move_up: Message,
    on_move_down: Message,
    on_hover: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    let right_button = container(
        button(text("›").font(*EMOJI_FONT).size(25))
            .on_press(on_press.clone())
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

    opt_custom_button(name, description, on_press, on_hover, element)
}

///Creates a row with a label with `name` and a `text_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn input<'a, Message: 'a + Clone>(
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
pub fn input_with_disable<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    placeholder: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a + Clone,
    on_submit: Option<Message>,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_input_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_change.clone());
    let element = row![label_with_description(name, description),]
        .push_maybe(disable_checkbox(disable_args))
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
pub fn input_with_disable_default<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    placeholder: &'a str,
    value: &'a str,
    default_value: String,
    on_change: impl Fn(String) -> Message + 'a + Clone,
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
        .push_maybe(disable_checkbox(disable_args))
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
pub fn number_simple<'a, Message: 'a + Clone>(
    value: i32,
    on_change: impl Fn(i32) -> Message + 'a + Clone + 'static,
) -> iced_aw::NumberInput<'a, i32, Message> {
    iced_aw::number_input(value, i32::MIN..=i32::MAX, on_change).style(num_button_style)
}

///Creates a row with a label with `name` and a `number_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: i32,
    on_change: impl Fn(i32) -> Message + 'a + Clone + 'static,
) -> Element<'a, Message> {
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
pub fn number_with_disable<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: i32,
    on_change: impl Fn(i32) -> Message + 'a + Copy + 'static,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let should_disable = disable_args.as_ref().is_some_and(|args| args.disable);
    let on_change = move |v| {
        if should_disable {
            on_change(value)
        } else {
            on_change(v)
        }
    };
    let bounds = if should_disable {
        value..=value
    } else {
        i32::MIN..=i32::MAX
    };
    let element = row![label_with_description(name, description),]
        .push_maybe(disable_checkbox(disable_args))
        .push(
            iced_aw::number_input(value, bounds, on_change)
                .style(num_button_style)
                .input_style(num_input_style(should_disable)),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name`, a `number_input` and a disable checkbox which allows
///toggling the number input on/off. It also adds a button in front of the label in case the value
///is diferent from `default_value` to send a message with the default value.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number_with_disable_default<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: i32,
    default_value: i32,
    on_change: impl Fn(i32) -> Message + 'a + Copy + 'static,
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
    let on_change = move |v| {
        if should_disable {
            on_change(value)
        } else {
            on_change(v)
        }
    };
    let bounds = if should_disable {
        value..=value
    } else {
        i32::MIN..=i32::MAX
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args))
        .push(
            iced_aw::number_input(value, bounds, on_change)
                .style(num_button_style)
                .input_style(num_input_style(should_disable)),
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
pub fn number_with_disable_default_option<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: Option<i32>,
    default_value: Option<i32>,
    on_change: impl Fn(Option<i32>) -> Message + 'a + Clone + 'static,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
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
    let bounds = if should_disable {
        value_internal..=value_internal
    } else {
        i32::MIN..=i32::MAX
    };
    let element = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args))
        .push(
            iced_aw::number_input(value_internal, bounds, on_change)
                .style(num_button_style)
                .input_style(num_input_style(should_disable)),
        )
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name` and a `colo_picker`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn color_picker_simple<'a, Message: 'a + Clone, F>(
    show_picker: bool,
    color: Color,
    underlay: Element<'a, Message>,
    on_cancel: Message,
    on_submit: F,
) -> iced_aw::ColorPicker<'a, Message>
where
    F: 'static + Fn(Color) -> Message,
{
    iced_aw::color_picker(show_picker, color, underlay, on_cancel, on_submit)
}

#[allow(clippy::too_many_arguments)]
pub fn color<'a, Message: 'a + Clone + 'static, F>(
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
    F: 'static + Fn(Option<Color>) -> Message,
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
        .push_maybe(disable_checkbox(disable_args))
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
pub fn bool_with_disable<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_toggle_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_toggle);
    let element = row![label_with_description(name, description)]
        .push_maybe(disable_checkbox(disable_args))
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
pub fn toggle_with_disable<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_toggle_maybe =
        (!matches!(&disable_args, Some(args) if args.disable)).then_some(on_toggle);
    let element = row![label_with_description(name, description)]
        .push_maybe(disable_checkbox(disable_args))
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
pub fn toggle_with_disable_default_no_option<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    default_value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a + Clone,
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
        .push_maybe(disable_checkbox(disable_args))
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
pub fn toggle_with_disable_default<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: Option<bool>,
    default_value: Option<bool>,
    on_toggle: impl Fn(Option<bool>) -> Message + 'a + Clone,
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
        .push_maybe(disable_checkbox(disable_args))
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
pub fn choose<'a, T, V, L, Message: 'a + Clone>(
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
pub fn choose_with_disable<'a, T, V, L, Message: 'a + Clone>(
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
        .push_maybe(disable_checkbox(disable_args))
        .push(pick_list(options, selected, on_selected));
    opt_box(element).into()
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name`. And adds a disable
///checkbox which allows toggling the choose on/off.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn choose_with_disable_default<'a, T, V, L, Message: 'a + Clone>(
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
    .push_maybe(disable_checkbox(disable_args))
    .push(
        pick_list(options, selected, move |v| on_selected(Some(v)))
            .font(ICONS)
            .text_shaping(text::Shaping::Advanced),
    )
    .spacing(10)
    .align_y(Center);
    opt_box(element).into()
}

///Creates an expandable option with children options to be shown when expanded.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn expandable<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    children: impl IntoIterator<Item = Element<'a, Message>>,
    expanded: bool,
    hovered: bool,
    on_press: Message,
    on_hover: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    let right_button = button(if expanded {
        text("▲").size(10)
    } else {
        text("▼").size(10)
    })
    .on_press(on_press.clone())
    .style(move |t, s| {
        if hovered {
            button::secondary(t, button::Status::Active)
        } else {
            button::text(t, s)
        }
    });

    let main = row![label_with_description(name, description), right_button]
        .align_y(Center)
        .padding(padding::right(10));

    let area = |el| {
        mouse_area(el)
            .on_press(on_press)
            .on_enter(on_hover(true))
            .on_exit(on_hover(false))
            .interaction(iced::mouse::Interaction::Pointer)
    };

    let element = if expanded {
        let top = opt_box(main).style(opt_box_style_top);
        let wrapped_top = area(top);
        let inner = Column::with_children(children)
            .spacing(10)
            .padding(padding::all(10).left(20));
        let wrapped_inner = opt_box(inner).style(opt_box_style_bottom);
        column![wrapped_top, horizontal_rule(2.0), wrapped_inner].into()
    } else {
        area(opt_box(main)).into()
    };
    element
}

///Creates an expandable option with children options to be shown when expanded.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn expandable_with_disable_default<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    children: impl IntoIterator<Item = Element<'a, Message>>,
    expanded: bool,
    hovered: bool,
    on_press: Message,
    on_hover: impl Fn(bool) -> Message,
    is_dirty: bool,
    on_default: Message,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let on_press_clone = on_press.clone();
    let right_button = |hovered: bool| {
        button(if expanded {
            text("▲").size(10)
        } else {
            text("▼").size(10)
        })
        .on_press(on_press_clone)
        .style(move |t, s| {
            if hovered {
                button::secondary(t, button::Status::Active)
            } else {
                button::text(t, s)
            }
        })
        .into()
    };

    expandable_with_disable_default_custom(
        name,
        description,
        right_button,
        children,
        expanded,
        hovered,
        Some(on_press),
        Some(on_hover),
        is_dirty,
        on_default,
        disable_args,
    )
}

///Creates an expandable option with children options to be shown when expanded.
///
///If `Some(description)` is given, it adds the description below the label.
#[allow(clippy::too_many_arguments)]
pub fn expandable_with_disable_default_custom<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    right_element: impl FnOnce(bool) -> Element<'a, Message>,
    children: impl IntoIterator<Item = Element<'a, Message>>,
    expanded: bool,
    hovered: bool,
    on_press: Option<Message>,
    on_hover: Option<impl Fn(bool) -> Message>,
    is_dirty: bool,
    on_default: Message,
    disable_args: Option<DisableArgs<'a, Message>>,
) -> Element<'a, Message> {
    let label = if is_dirty {
        row![name.into(), reset_button(on_default)]
            .spacing(5)
            .height(30)
            .align_y(Center)
    } else {
        row![name.into()].height(30).align_y(Center)
    };
    let main = row![label_element_with_description(label, description)]
        .push_maybe(disable_checkbox(disable_args))
        .push(right_element(hovered))
        .align_y(Center)
        .padding(padding::right(10))
        .spacing(10);

    let area = |el: Container<'a, Message>| -> Element<'a, Message> {
        if let (Some(on_press), Some(on_hover)) = (on_press, on_hover) {
            mouse_area(el)
                .on_press(on_press)
                .on_enter(on_hover(true))
                .on_exit(on_hover(false))
                .interaction(iced::mouse::Interaction::Pointer)
                .into()
        } else {
            el.into()
        }
    };
    // let disable_area = |el| {
    //     mouse_area(container(el).width(Fill).height(Fill))
    //         .on_press(on_default)
    //         .interaction(iced::mouse::Interaction::NotAllowed)
    // };

    let element = if expanded {
        let top = opt_box(main).style(opt_box_style_top);
        let wrapped_top = area(top);
        let inner = Column::with_children(children)
            .spacing(10)
            .padding(padding::all(10).left(20));
        let wrapped_inner = opt_box(inner).style(opt_box_style_bottom);
        // let wrapped_inner: Element<Message> = if should_disable {
        //     iced::widget::stack([wrapped_inner.into(), disable_area("").into()]).into()
        // } else {
        //     wrapped_inner.into()
        // };
        column![wrapped_top, horizontal_rule(2.0), wrapped_inner].into()
    } else {
        area(opt_box(main))
    };
    element
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
