#![allow(dead_code)]
use crate::{widget, Message, BOLD_FONT};

use iced::{
    padding,
    widget::{
        button, checkbox, column, horizontal_space, pick_list, row, text, toggler, Column, Row,
        Text,
    },
    Center, Element,
};

///Creates a row with a label with `name` and a `text_input`
///using the remainder parameters for it.
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
pub fn input<'a>(
    name: &'a str,
    description: Option<&'a str>,
    placeholder: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
) -> Element<'a, Message> {
    let element = row![
        widget::label(name),
        widget::input(placeholder, value, on_change, on_submit),
    ]
    .spacing(10)
    .align_y(Center)
    .into();
    match description {
        Some(desc) => widget::create_tooltip(element, desc),
        None => element,
    }
}

///Creates a `checkbox` with `name` as label
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
pub fn bool<'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let element = checkbox(name, value).on_toggle(on_toggle).into();
    match description {
        Some(desc) => widget::create_tooltip(element, desc),
        None => element,
    }
}

///Creates a `toggler` with `name` as label
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
pub fn toggle<'a>(
    name: &'a str,
    description: Option<&'a str>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let element = toggler(value).label(name).on_toggle(on_toggle).into();
    match description {
        Some(desc) => widget::create_tooltip(element, desc),
        None => element,
    }
}

///Creates a `pick_list`, if `name` is not empty it wraps the
///`pick_list` on a row with a label with `name` in front.
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
pub fn choose<'a, T, V, L>(
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
        .push_maybe((!name.is_empty()).then_some(widget::label(name)))
        .push(pick_list(options, selected, on_selected))
        .into();
    match description {
        Some(desc) => widget::create_tooltip(element, desc),
        None => element,
    }
}

pub struct InputParameters<'a> {
    pub name: &'a str,
    pub placeholder: &'a str,
    pub value: &'a str,
    pub on_change: Box<dyn Fn(String) -> Message + 'a>,
    pub on_submit: Option<Message>,
}

///Creates a column with a row and a label with `name` and a `text_input` per input
///
///If `Some(description)` is given, it will wrap the resulting
///widget on a tooltip with the given `description`.
pub fn input_col<'a>(
    name: &'a str,
    description: Option<&'a str>,
    inputs: impl IntoIterator<Item = InputParameters<'a>>,
    expanded: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let main = column![Row::new()
        .push(name)
        .push(horizontal_space())
        .push(
            button(if expanded { text("^") } else { text("v") })
                .on_press(on_press.clone())
                .style(iced::widget::button::text)
        )
        .align_y(Center)
        .height(30.0)]
    .spacing(1);

    let col = if expanded {
        let inner = inputs.into_iter().fold(
            column![]
                .spacing(10)
                .padding(iced::Padding::new(10.0).left(20.0)),
            |col, input| {
                col.push(
                    row![
                        widget::label(input.name),
                        widget::input(
                            input.placeholder,
                            input.value,
                            input.on_change,
                            input.on_submit.clone()
                        ),
                    ]
                    .spacing(5)
                    .align_y(Center),
                )
            },
        );
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
    match description {
        Some(desc) => widget::create_tooltip(element, desc),
        None => element,
    }
}

///Creates a row of options within a section. This is a simple helper to create
///a row with the predefined spacing. It can have more than one option on it,
///for example if you want to have multiple checkboxes side by side, you should
///put them all on one section_row.
pub fn section_row<'a>(
    contents: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    Row::with_children(contents).spacing(10).into()
}

///Creates the view for a section of options with `title` and the `contents`.
pub fn section_view<'a>(
    title: impl Into<Text<'a>>,
    contents: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let section_title = row![title.into().size(20.0).font(*BOLD_FONT)];
    column![
        section_title,
        Column::with_children(contents).spacing(10),
    ]
    .spacing(10)
    .padding(padding::top(10).bottom(20))
    .into()
}
