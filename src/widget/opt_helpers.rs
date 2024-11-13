#![allow(dead_code)]
use crate::{widget, BOLD_FONT};

use iced::{
    color, padding,
    widget::{
        button, checkbox, column, container, horizontal_rule, pick_list, row, scrollable, text,
        toggler, Column, Row, Text,
    },
    Center, Element, Fill,
};

fn opt_box<'a, Message: 'a>(element: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(element)
        .padding(10)
        .style(|t| {
            let palette = t.extended_palette();
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
        })
        .into()
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
                    text::Style {
                        color,
                    }
                })
                .size(13)
                .width(Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
        }))
        .width(Fill)
        .spacing(5)
        .into()
}

///Creates a row with a label with `name` and a `text_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
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
    .align_y(Center)
    .into();
    element
    // match description {
    //     Some(desc) => widget::create_tooltip(element, desc),
    //     None => element,
    // }
}

///Creates a row with a label with `name` and a `number_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
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
    .align_y(Center)
    .into();
    element
    // match description {
    //     Some(desc) => widget::create_tooltip(element, desc),
    //     None => element,
    // }
}

///Creates a `checkbox` with `name` as label
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
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
    .align_y(Center)
    .into();
    element
    // match description {
    //     Some(desc) => widget::create_tooltip(element, desc),
    //     None => element,
    // }
}

///Creates a `toggler` with `name` as label
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
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
    .align_y(Center)
    .into();
    element
    // match description {
    //     Some(desc) => widget::create_tooltip(element, desc),
    //     None => element,
    // }
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name` in front.
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
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
        // .push_maybe((!name.is_empty()).then_some(widget::label(name)))
        .push_maybe((!name.is_empty()).then_some(label_with_description(name, description)))
        .push(pick_list(options, selected, on_selected))
        .into();
    element
    // match description {
    //     Some(desc) => widget::create_tooltip(element, desc),
    //     None => element,
    // }
}

///Creates an expandable option with children options to be shown when expanded.
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
pub fn expandable<'a, Message: 'a + Clone>(
    name: impl Into<Text<'a>>,
    description: Option<&'a str>,
    children: impl IntoIterator<Item = Element<'a, Message>>,
    expanded: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let main = column![Row::new()
        .push(label_with_description(name, description))
        .push(
            button(if expanded {
                text("▲").size(10)
            } else {
                text("▼").size(10)
            })
            .on_press(on_press.clone())
            .style(|t, s| {
                if matches!(s, button::Status::Hovered) {
                    button::secondary(t, s)
                } else {
                    button::text(t, s)
                }
            })
        )
        .align_y(Center)
        .padding(padding::right(10))
        .height(30.0)]
    .spacing(1);

    let col = if expanded {
        let inner = Column::with_children(children)
            .spacing(10)
            .padding(padding::all(10).left(20));
        main.push(inner)
    } else {
        main
    };
    let element = button(col)
        .padding(0.0)
        .on_press(on_press)
        .style(move |t: &iced::Theme, s: button::Status| {
            if matches!(s, button::Status::Hovered) {
                button::text(t, s).with_background(t.extended_palette().background.weak.color)
            } else {
                button::text(t, s)
            }
        })
        .into();
    element
    // match description {
    //     Some(desc) => widget::create_tooltip(element, desc),
    //     None => element,
    // }
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

///Creates the view for a section of options with `title` and the `contents`.
pub fn section_view<'a, Message: 'a>(
    title: impl Into<Text<'a>>,
    contents: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let section_title = row![title.into().size(20.0).font(*BOLD_FONT)];
    column![
        section_title,
        horizontal_rule(2.0),
        scrollable(
            Column::with_children(contents.into_iter().map(opt_box))
                .padding(padding::top(10).bottom(10).right(20))
                .spacing(10)
        )
    ]
    .spacing(10)
    .into()
}
