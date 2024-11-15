#![allow(dead_code)]
use crate::{widget, BOLD_FONT, EMOJI_FONT};

use iced::{
    padding,
    widget::{
        button, checkbox, column, container, horizontal_rule, mouse_area, pick_list, row,
        scrollable, text, toggler, Column, Container, Row, Text,
    },
    Center, Element, Fill,
};

pub struct DisableArgs<'a, Message, F>
where
    F: Fn(bool) -> Message + 'a,
{
    pub disable: bool,
    pub label: Option<&'a str>,
    pub on_toggle: F,
}

fn opt_box_style(theme: &iced::Theme) -> container::Style {
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

fn opt_box_style_top(theme: &iced::Theme) -> container::Style {
    container::Style {
        border: iced::Border {
            radius: iced::border::top(5),
            ..opt_box_style(theme).border
        },
        ..opt_box_style(theme)
    }
}

fn opt_box_style_bottom(theme: &iced::Theme) -> container::Style {
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

fn opt_box<'a, Message: 'a>(element: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(element).padding(10).style(opt_box_style)
}

///Creates a column with a label and a description
///
///If `Some(description)` is given, it adds the description below the name.
pub fn label_with_description<'a, Message: 'a>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
) -> Element<'a, Message> {
    column![widget::label(name)]
        .push_maybe(description.map(|d| {
            text(d)
                .style(|t: &iced::Theme| {
                    let palette = t.extended_palette();
                    let color = if palette.is_dark {
                        Some(palette.secondary.strong.color)
                    } else {
                        Some(palette.background.base.text.scale_alpha(0.75))
                    };
                    text::Style { color }
                })
                .size(13)
                .width(Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
        }))
        .width(Fill)
        .spacing(5)
        .into()
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

    area(opt_box(main)).into()
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
    on_change: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
    disable_args: Option<DisableArgs<'a, Message, impl Fn(bool) -> Message + 'a>>,
) -> Element<'a, Message> {
    let element = row![label_with_description(name, description),]
        .push_maybe(disable_args.map(|args| {
            checkbox(args.label.unwrap_or_default(), args.disable).on_toggle(args.on_toggle)
        }))
        .push(widget::input(placeholder, value, on_change, on_submit))
        .spacing(10)
        .align_y(Center);
    opt_box(element).into()
}

///Creates a row with a label with `name` and a `number_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it adds the description below the label.
pub fn number<'a, Message: 'a + Clone>(
    name: &'a str,
    description: Option<&'a str>,
    value: i32,
    on_change: impl Fn(i32) -> Message + 'a + Copy + 'static,
) -> Element<'a, Message> {
    let element = row![
        label_with_description(name, description),
        iced_aw::number_input(value, i32::MIN..=i32::MAX, on_change).style(|t: &iced::Theme, _| {
            iced_aw::number_input::number_input::Style {
                button_background: Some(t.extended_palette().background.weak.color.into()),
                icon_color: t.extended_palette().background.weak.text,
            }
        }),
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
    disable_args: Option<DisableArgs<'a, Message, impl Fn(bool) -> Message + 'a>>,
) -> Element<'a, Message> {
    let should_disable = disable_args.as_ref().map_or(false, |args| args.disable);
    let on_change = move |v| {
        if should_disable {
            on_change(value)
        } else {
            on_change(v)
        }
    };
    let bounds = if should_disable {
        0..=0
    } else {
        i32::MIN..=i32::MAX
    };
    let element = row![label_with_description(name, description),]
        .push_maybe(disable_args.map(|args| {
            row![
                text(args.label.unwrap_or_default()),
                checkbox("", args.disable)
                    .spacing(0)
                    .on_toggle(args.on_toggle)
            ]
            .spacing(10)
        }))
        .push(
            iced_aw::number_input(value, bounds, on_change).style(|t: &iced::Theme, _| {
                iced_aw::number_input::number_input::Style {
                    button_background: Some(t.extended_palette().background.weak.color.into()),
                    icon_color: t.extended_palette().background.weak.text,
                }
            }),
        )
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
pub fn bool_with_disable<'a, Message: 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message, impl Fn(bool) -> Message + 'a>>,
) -> Element<'a, Message> {
    let on_toggle_maybe = disable_args
        .as_ref()
        .and_then(|args| (!args.disable).then_some(on_toggle));
    let element = row![label_with_description(name, description)]
        .push_maybe(disable_args.map(|args| {
            checkbox(args.label.unwrap_or_default(), args.disable).on_toggle(args.on_toggle)
        }))
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
pub fn toggle_with_disable<'a, Message: 'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    disable_args: Option<DisableArgs<'a, Message, impl Fn(bool) -> Message + 'a>>,
) -> Element<'a, Message> {
    let on_toggle_maybe = disable_args
        .as_ref()
        .and_then(|args| (!args.disable).then_some(on_toggle));
    let element = row![label_with_description(name, description)]
        .push_maybe(disable_args.map(|args| {
            checkbox(args.label.unwrap_or_default(), args.disable).on_toggle(args.on_toggle)
        }))
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
    disable_args: Option<DisableArgs<'a, Message, impl Fn(bool) -> Message + 'a>>,
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
        .push_maybe(disable_args.map(|args| {
            checkbox(args.label.unwrap_or_default(), args.disable).on_toggle(args.on_toggle)
        }))
        .push(pick_list(options, selected, on_selected));
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
    // .padding(padding::Padding {
    //     top: 10.0,
    //     right: 10.0,
    //     bottom: 5.0,
    //     left: 10.0,
    // })
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
        )
    ]
    .spacing(10)
    .into()
}
