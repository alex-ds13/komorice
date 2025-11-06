use super::ConfigType;

use crate::EMOJI_FONT;

use iced::{
    Center, Element, Fill, Shrink, Task,
    widget::{Space, button, center, column, container, image, row, text},
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
    ChangeConfigType(ConfigType),
}

#[derive(Debug, Default, Clone)]
pub struct Home {
    pub has_loaded_config: bool,
    pub has_loaded_whkdrc: bool,
}

impl Home {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::EditActiveConfig => {
                return (Action::ChangeConfigType(ConfigType::Komorebi), Task::none());
            }
            Message::LoadConfig => { /*TODO*/ }
            Message::NewConfig => { /*TODO*/ }
            Message::EditActiveWhkdrc => {
                return (Action::ChangeConfigType(ConfigType::Whkd), Task::none());
            }
            Message::LoadWhkdrc => { /*TODO*/ }
            Message::NewWhkdrc => { /*TODO*/ }
        }
        (Action::None, Task::none())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let image = center(image("assets/komorice.png").width(256).height(256));
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
            ConfigType::Komorebi,
            Message::EditActiveConfig,
            Message::LoadConfig,
            Message::NewConfig,
        );
        let whkd_buttons = self.button_col(
            ConfigType::Whkd,
            Message::EditActiveWhkdrc,
            Message::LoadWhkdrc,
            Message::NewWhkdrc,
        );
        let buttons_row = row![config_buttons, whkd_buttons]
            .spacing(50)
            .height(Shrink);
        let col = column![title, subtitle, image, buttons_row]
            .spacing(20)
            .align_x(Center);

        container(col)
            .padding(20)
            .center_x(Fill)
            .height(Fill)
            .into()
    }

    fn button_col(
        &self,
        config_type: ConfigType,
        edit: Message,
        load: Message,
        new_file: Message,
    ) -> Element<'_, Message> {
        let fixed_width = Space::new(180, Shrink);
        let edit = match config_type {
            ConfigType::Komorebi => self.has_loaded_config.then_some(edit),
            ConfigType::Whkd => self.has_loaded_whkdrc.then_some(edit),
        };

        column![
            fixed_width,
            text(config_type.to_string().to_uppercase()).size(18),
            container(
                button(
                    text!("Edit active {}", config_type.file_str())
                        .width(Fill)
                        .align_x(Center)
                )
                .on_press_maybe(edit)
                .style(button::secondary)
            ),
            container(
                button(
                    text!("Load {} file", config_type.file_str())
                        .width(Fill)
                        .align_x(Center)
                )
                .on_press(load)
                .style(button::secondary)
            ),
            container(
                button(
                    text!("New {} file", config_type.file_str())
                        .width(Fill)
                        .align_x(Center)
                )
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
