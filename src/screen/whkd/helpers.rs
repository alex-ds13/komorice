use crate::widget::modal;

use iced::Element;
use iced::widget::{button, column, container, row, text};

pub fn keybind_modal<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    show: bool,
    modifiers: &'a String,
    keys: &'a [String],
    on_close: impl Fn(bool) -> Message + Clone,
) -> Element<'a, Message> {
    modal(
        content.into(),
        show.then_some(modal_content(
            modifiers,
            keys,
            on_close.clone(),
        )),
        on_close(false),
    )
}

pub fn modal_content<'a, Message: Clone + 'a>(
    modifiers: &'a String,
    keys: &'a [String],
    on_close: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    container(
        column![
            "Press some key to bind:",
            {
                let mut key_pressed = row![text!("{}", modifiers),];

                key_pressed = key_pressed
                    .push((!modifiers.is_empty() && !keys.is_empty()).then_some(text(" + ")));
                key_pressed = key_pressed.push(text!(
                    "{}",
                    keys.iter().fold(String::new(), |mut str, k| {
                        if !str.is_empty() {
                            str.push_str(" + ");
                        }
                        str.push_str(k);
                        str
                    })
                ));
                key_pressed
            },
            row![
                button("Save").on_press(on_close(true)),
                button("Cancel")
                    .style(button::secondary)
                    .on_press(on_close(false))
            ]
            .spacing(10),
        ]
        .align_x(iced::Center)
        .spacing(10),
    )
    .padding(50)
    .style(container::bordered_box)
    .into()
}
