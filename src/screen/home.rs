use super::{Screen, Sidebar};

use crate::EMOJI_FONT;

use iced::{
    widget::{button, center, column, container, image, row, text, Space},
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

#[derive(Debug, Clone)]
pub enum ConfigType {
    Komorebi,
    Whkd,
}

impl ConfigType {
    pub fn file_str(&self) -> &'static str {
        match self {
            ConfigType::Komorebi => "config",
            ConfigType::Whkd => "whkdrc",
        }
    }
}

impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigType::Komorebi => write!(f, "Komorebi"),
            ConfigType::Whkd => write!(f, "Whkd"),
        }
    }
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
    ) -> Element<Message> {
        let fixed_width = Space::new(180, Shrink);
        let edit = match config_type {
            ConfigType::Komorebi => self.has_loaded_config.then_some(edit),
            ConfigType::Whkd => self.has_loaded_config.then_some(edit),
            //TODO: change this to:
            //ConfigType::Whkd => self.has_loaded_whkdrc.then_some(edit),
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
