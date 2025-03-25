use iced::widget::{center, container, mouse_area, opaque, stack};
use iced::{Border, Color, Element, Theme};

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: Option<impl Into<Element<'a, Message>>>,
    on_close: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![base.into()]
        .push_maybe(content.map(|content| {
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.5,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_close)
        }))
        .into()
}

pub fn default(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let background = Some(
        Color {
            a: 0.95,
            ..palette.background.base.color
        }
        .into(),
    );

    container::Style {
        text_color: None,
        background,
        border: Border {
            color: palette.background.strong.color,
            width: 1.5,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

pub fn red(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let background = Some(
        Color {
            a: 0.95,
            ..palette.danger.base.color
        }
        .into(),
    );

    container::Style {
        text_color: Some(palette.danger.base.text),
        background,
        border: Border {
            color: palette.danger.base.color,
            width: 2.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}
