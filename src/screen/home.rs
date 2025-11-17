use super::{ConfigState, ConfigType, Configuration};
use crate::{EMOJI_FONT, config, whkd};

use std::path::PathBuf;

use iced::{
    Center, Element, Fill, Shrink, Task,
    widget::{button, center, column, container, image, opaque, row, space, stack, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    EditCurrentConfig,
    LoadConfig,
    LoadConfigResult(Option<PathBuf>),
    NewConfig,
    NewConfigResult(Option<PathBuf>),
    EditCurrentWhkdrc,
    LoadWhkdrc,
    LoadWhkdrcResult(Option<PathBuf>),
    NewWhkdrc,
    NewWhkdrcResult(Option<PathBuf>),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    ContinueEditConfigType,
    LoadConfigType,
    NewConfigType,
}

#[derive(Debug, Default, Clone)]
pub struct Home {
    dialog_opened: bool,
}

impl Home {
    pub fn update(
        &mut self,
        message: Message,
        configuration: &mut Configuration,
    ) -> (Action, Task<Message>) {
        match message {
            Message::EditCurrentConfig => {
                configuration.config_type = ConfigType::Komorebi;
                return (Action::ContinueEditConfigType, Task::none());
            }
            Message::LoadConfig => {
                let (home_dir, _) = config::home_path();
                self.dialog_opened = true;
                return (
                    Action::None,
                    Task::perform(
                        async move {
                            rfd::FileDialog::new()
                                .add_filter("json", &["json"])
                                .set_directory(home_dir.as_path())
                                .pick_file()
                        },
                        Message::LoadConfigResult,
                    ),
                );
            }
            Message::LoadConfigResult(file) => {
                self.dialog_opened = false;
                if let Some(file) = file {
                    println!("Loading config from '{}'", file.display());
                    configuration.komorebi_state = ConfigState::Loaded(file);
                    configuration.config_type = ConfigType::Komorebi;
                    configuration.has_loaded_komorebi = false;
                    return (Action::LoadConfigType, Task::none());
                }
            }
            Message::NewConfig => {
                let (home_dir, _) = config::home_path();
                self.dialog_opened = true;
                return (
                    Action::None,
                    Task::perform(
                        async move {
                            rfd::FileDialog::new()
                                .add_filter("json", &["json"])
                                .set_directory(home_dir.as_path())
                                .save_file()
                        },
                        Message::NewConfigResult,
                    ),
                );
            }
            Message::NewConfigResult(file) => {
                self.dialog_opened = false;
                if let Some(file) = file {
                    println!("Saving new config to '{}'", file.display());
                    configuration.komorebi_state = ConfigState::New(file);
                    configuration.config_type = ConfigType::Komorebi;
                    configuration.saved_new_komorebi = false;
                    return (Action::NewConfigType, Task::none());
                }
            }
            Message::EditCurrentWhkdrc => {
                configuration.config_type = ConfigType::Whkd;
                return (Action::ContinueEditConfigType, Task::none());
            }
            Message::LoadWhkdrc => {
                let home_dir = whkd::home_path();
                self.dialog_opened = true;
                return (
                    Action::None,
                    Task::perform(
                        async move {
                            rfd::FileDialog::new()
                                .set_directory(home_dir.as_path())
                                .pick_file()
                        },
                        Message::LoadWhkdrcResult,
                    ),
                );
            }
            Message::LoadWhkdrcResult(file) => {
                self.dialog_opened = false;
                if let Some(file) = file {
                    println!("Loading whkdrc from '{}'", file.display());
                    configuration.whkd_state = ConfigState::Loaded(file);
                    configuration.config_type = ConfigType::Whkd;
                    configuration.has_loaded_whkd = false;
                    return (Action::LoadConfigType, Task::none());
                }
            }
            Message::NewWhkdrc => {
                let home_dir = whkd::home_path();
                println!("Using Start dir as: {}", home_dir.display());
                self.dialog_opened = true;
                return (
                    Action::None,
                    Task::perform(
                        async move {
                            rfd::FileDialog::new()
                                .set_directory(home_dir.as_path())
                                .save_file()
                        },
                        Message::NewWhkdrcResult,
                    ),
                );
            }
            Message::NewWhkdrcResult(file) => {
                self.dialog_opened = false;
                if let Some(file) = file {
                    println!("Saving new whkdrc to '{}'", file.display());
                    configuration.whkd_state = ConfigState::New(file);
                    configuration.config_type = ConfigType::Whkd;
                    configuration.saved_new_whkd = false;
                    return (Action::NewConfigType, Task::none());
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view(&self, configuration: &Configuration) -> Element<'_, Message> {
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
            configuration,
            Message::EditCurrentConfig,
            Message::LoadConfig,
            Message::NewConfig,
        );
        let whkd_buttons = self.button_col(
            ConfigType::Whkd,
            configuration,
            Message::EditCurrentWhkdrc,
            Message::LoadWhkdrc,
            Message::NewWhkdrc,
        );
        let buttons_row = row![config_buttons, whkd_buttons]
            .spacing(50)
            .height(Shrink);
        let col = column![title, subtitle, image, buttons_row]
            .spacing(20)
            .align_x(Center);

        stack![
            container(col).padding(20).center_x(Fill).height(Fill),
            self.dialog_opened.then(|| opaque(center("").style(|t| {
                container::Style {
                    background: Some(iced::color!(0x000000, 0.5).into()),
                    ..container::dark(t)
                }
            }))),
        ]
        .into()
    }

    fn button_col(
        &self,
        config_type: ConfigType,
        configuration: &Configuration,
        edit: Message,
        load: Message,
        new_file: Message,
    ) -> Element<'_, Message> {
        let fixed_width = space().width(180);
        let edit = configuration.has_loaded_active(config_type).then_some(edit);

        column![
            fixed_width,
            text(configuration.title().to_uppercase()).size(18),
            container(
                button(
                    text!(
                        "Edit {} {}",
                        configuration.state_str(config_type),
                        config_type.as_str()
                    )
                    .width(Fill)
                    .align_x(Center)
                )
                .on_press_maybe(edit)
                .style(button::secondary)
            ),
            container(
                button(
                    text!("Load {} file", config_type.as_str())
                        .width(Fill)
                        .align_x(Center)
                )
                .on_press(load)
                .style(button::secondary)
            ),
            container(
                button(
                    text!("New {} file", config_type.as_str())
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
