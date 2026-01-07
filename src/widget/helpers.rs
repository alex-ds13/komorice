#![allow(dead_code)]
use crate::widget::text_input::TextInput;
use iced::widget::{
    Button, Row, Rule, Text, button, container, row, rule, text::IntoFragment, tooltip,
};
use iced::{Center, Element, Task, Theme};

pub fn label<'a, Message>(text: impl Into<Text<'a>>) -> Row<'a, Message> {
    Row::new().push(text.into()).align_y(Center).height(30.0)
}

pub fn input<'a, Message: Clone>(
    placeholder: &'a str,
    value: impl IntoFragment<'a>,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
) -> TextInput<'a, Message> {
    TextInput::new(placeholder, value)
        .padding([0, 5])
        .line_height(1.85)
        .on_input(on_input)
        .on_submit_maybe(on_submit)
}

pub fn button_with_icon<'a, Message: 'a>(
    icon: Text<'a>,
    text: impl Into<Text<'a>>,
) -> Button<'a, Message> {
    button(row![icon, text.into()].spacing(5).align_y(Center))
}

pub fn button_separator<'a>() -> Rule<'a> {
    rule::vertical(1.0).style(|t: &Theme| rule::Style {
        color: t.extended_palette().primary.base.color,
        ..rule::default(t)
    })
}

///Wraps the `element` on a tooltip widget with `description`.
pub fn create_tooltip<'a, Message: 'a>(
    element: Element<'a, Message>,
    description: &'a str,
) -> Element<'a, Message> {
    tooltip(
        element,
        container(Text::new(description))
            .padding(5.0)
            .max_width(700.0)
            .style(|t: &Theme| container::Style {
                background: Some(
                    iced::Color {
                        a: 0.95,
                        ..iced::Color::BLACK
                    }
                    .into(),
                ),
                text_color: Some(iced::Color::WHITE),
                ..container::rounded_box(t)
            }),
        tooltip::Position::Bottom,
    )
    .into()
}

/// An unfocus task to remove focus from the currently focused widget.
pub fn unfocus<M: Send + 'static>() -> Task<M> {
    use iced::advanced::widget;

    widget::operate(widget::operation::focusable::unfocus())
}
