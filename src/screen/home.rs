use super::{Screen, Sidebar};

use crate::{config, EMOJI_FONT, ITALIC_FONT};

use iced::{
    padding,
    widget::{button, column, container, image, row, stack, text, Space},
    Center, Element, Fill, Shrink, Task,
};

#[derive(Debug, Clone)]
pub enum Message {
    EditActiveConfig,
    LoadConfig,
    NewConfig,
    EditActiveWhkdrc,
    LoadWhkdrc,
    NewWhkdrc,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    ChangeMainScreen(Screen, Sidebar),
}

#[derive(Debug, Default, Clone)]
pub struct Home {
    pub has_loaded_config: bool,
    pub has_loaded_whkdrc: bool,
}

impl Home {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::EditActiveConfig => (
                Action::ChangeMainScreen(Screen::General, Sidebar::Config(Default::default())),
                Task::none(),
            ),
            Message::LoadConfig => todo!(),
            Message::NewConfig => todo!(),
            Message::EditActiveWhkdrc => (
                Action::ChangeMainScreen(Screen::Whkd, Sidebar::Whkd(Default::default())),
                Task::none(),
            ),
            Message::LoadWhkdrc => todo!(),
            Message::NewWhkdrc => todo!(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let image = container(image("assets/komorice.png").width(256).height(256)).center_x(Fill);
        let title = container(
            row![
                text("🍉").font(*EMOJI_FONT).size(70),
                text("Komorice").size(75),
                text("🍚").font(*EMOJI_FONT).size(70)
            ]
            .align_y(Center),
        )
        .center_x(Fill);
        let subtitle = text("A komorebi GUI ricing configurator!")
            .size(20)
            .width(Fill)
            .align_x(Center);
        let config_buttons = self.button_col(
            "config",
            Message::EditActiveConfig,
            Message::LoadConfig,
            Message::NewConfig,
        );
        let whkd_buttons = self.button_col(
            "whkdrc",
            Message::EditActiveWhkdrc,
            Message::LoadWhkdrc,
            Message::NewWhkdrc,
        );
        let buttons_row = row![config_buttons, whkd_buttons].spacing(50);
        let col = column![title, subtitle, image, buttons_row]
            .spacing(20)
            .align_x(Center);
        stack([
            container(col)
                .padding(20)
                .center_x(Fill)
                .height(Fill)
                .into(),
            container(
                text!(
                    "Config was {} loaded from \"{}\"!",
                    if self.has_loaded_config {
                        "successfully"
                    } else {
                        "not"
                    },
                    config::config_path().display()
                )
                .font(*ITALIC_FONT)
                .size(18),
            )
            .center_x(Fill)
            .align_bottom(Fill)
            .padding(padding::bottom(10))
            .into(),
        ])
        .into()
    }

    fn button_col(
        &self,
        name: &str,
        edit: Message,
        load: Message,
        new_file: Message,
    ) -> Element<Message> {
        let fixed_width = Space::new(180, Shrink);
        column![
            fixed_width,
            text(name.to_uppercase()).size(18),
            container(
                button(text!("Edit active {name}").width(Fill).align_x(Center))
                    .on_press_maybe(self.has_loaded_config.then_some(edit))
                    .style(button::secondary)
            ),
            container(
                button(text!("Load {name} file").width(Fill).align_x(Center))
                    .on_press(load)
                    .style(button::secondary)
            ),
            container(
                button(text!("New {name} file").width(Fill).align_x(Center))
                    .on_press(new_file)
                    .style(button::secondary)
            ),
        ]
        .align_x(Center)
        .width(Shrink)
        .spacing(10)
        .into()
    }
}
