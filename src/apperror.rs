use crate::Message;
use crate::widget::{self, icons};

use iced::{
    Border, Center, Color, Element, Fill,
    widget::{Text, column, container, row, text},
};

#[derive(Debug, Clone)]
pub struct AppError {
    pub title: String,
    pub description: Option<String>,
    pub kind: AppErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppErrorKind {
    Info,
    Warning,
    Error,
}

impl AppError {
    pub fn view(&self) -> Element<'_, Message> {
        column![
            row![self.kind.view(), text(&self.title).size(18)]
                .spacing(10)
                .align_y(Center)
        ]
        .push(self.description.as_ref().map(|d| {
            container(text(d))
                .width(Fill)
                .style(|_| container::Style {
                    background: Some(
                        Color {
                            a: 0.5,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    border: Border {
                        color: self.kind.color(),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..container::Style::default()
                })
                .padding(10.0)
        }))
        .spacing(10)
        .into()
    }
}

impl AppErrorKind {
    fn color(&self) -> Color {
        match self {
            AppErrorKind::Info => *widget::BLUE,
            AppErrorKind::Warning => *widget::YELLOW,
            AppErrorKind::Error => *widget::RED,
        }
    }

    fn icon(&self) -> Text<'_> {
        match self {
            AppErrorKind::Info => icons::info(),
            AppErrorKind::Warning => icons::warning(),
            AppErrorKind::Error => icons::error(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container(self.icon().size(20).color(Color {
            a: 0.9,
            ..Color::BLACK
        }))
        .style(move |t| container::Style {
            background: Some(self.color().into()),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 13.0.into(),
            },
            ..container::transparent(t)
        })
        .clip(false)
        .center(26)
        .padding(0.0)
        .into()
    }
}
