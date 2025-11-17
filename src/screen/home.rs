use super::{ConfigState, ConfigType, Configuration};
use crate::{EMOJI_FONT, config, whkd};

use iced::{
    Center, Element, Fill, Shrink, Task,
    widget::{button, center, column, container, image, opaque, row, space, stack, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    EditCurrent(ConfigType),
    Load(ConfigType),
    New(ConfigType),
    ChangeConfiguration(ConfigType, ConfigState),
    ClosedDialog,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    ContinueEdit,
    ChangedConfiguration,
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
            Message::EditCurrent(config_type) => {
                configuration.config_type = config_type;
                return (Action::ContinueEdit, Task::none());
            }
            Message::Load(config_type) => {
                self.dialog_opened = true;
                match config_type {
                    ConfigType::Komorebi => return (Action::None, load_komorebi()),
                    ConfigType::Whkd => return (Action::None, load_whkd()),
                }
            }
            Message::New(config_type) => {
                self.dialog_opened = true;
                match config_type {
                    ConfigType::Komorebi => return (Action::None, new_komorebi()),
                    ConfigType::Whkd => return (Action::None, new_whkd()),
                }
            }
            Message::ChangeConfiguration(config_type, state) => {
                self.dialog_opened = false;
                configuration.config_type = config_type;
                match state {
                    ConfigState::Active => {
                        println!(
                            "Got 'Active' state on a configuration change, it shouldn't happen!"
                        );
                    }
                    ConfigState::Loaded(_) => match configuration.config_type {
                        ConfigType::Komorebi => {
                            configuration.komorebi_state = state;
                            configuration.has_loaded_komorebi = false;
                        }
                        ConfigType::Whkd => {
                            configuration.whkd_state = state;
                            configuration.has_loaded_whkd = false;
                        }
                    },
                    ConfigState::New(_) => match configuration.config_type {
                        ConfigType::Komorebi => {
                            configuration.komorebi_state = state;
                            configuration.saved_new_komorebi = false;
                        }
                        ConfigType::Whkd => {
                            configuration.whkd_state = state;
                            configuration.saved_new_whkd = false;
                        }
                    },
                }
                return (Action::ChangedConfiguration, Task::none());
            }
            Message::ClosedDialog => self.dialog_opened = false,
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
        let komorebi_buttons = self.button_col(
            ConfigType::Komorebi,
            configuration,
            Message::EditCurrent(ConfigType::Komorebi),
            Message::Load(ConfigType::Komorebi),
            Message::New(ConfigType::Komorebi),
        );
        let whkd_buttons = self.button_col(
            ConfigType::Whkd,
            configuration,
            Message::EditCurrent(ConfigType::Whkd),
            Message::Load(ConfigType::Whkd),
            Message::New(ConfigType::Whkd),
        );
        let buttons_row = row![komorebi_buttons, whkd_buttons]
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

fn load_komorebi() -> Task<Message> {
    let (home_dir, _) = config::home_path();
    Task::future(async move {
        rfd::FileDialog::new()
            .add_filter("json", &["json"])
            .set_directory(home_dir.as_path())
            .pick_file()
    })
    .map(|res| match res {
        Some(file) => Message::ChangeConfiguration(ConfigType::Komorebi, ConfigState::Loaded(file)),
        None => Message::ClosedDialog,
    })
}

fn new_komorebi() -> Task<Message> {
    let (home_dir, _) = config::home_path();
    Task::future(async move {
        rfd::FileDialog::new()
            .add_filter("json", &["json"])
            .set_directory(home_dir.as_path())
            .save_file()
    })
    .map(|res| match res {
        Some(file) => Message::ChangeConfiguration(ConfigType::Komorebi, ConfigState::New(file)),
        None => Message::ClosedDialog,
    })
}

fn load_whkd() -> Task<Message> {
    let home_dir = whkd::home_path();
    Task::future(async move {
        rfd::FileDialog::new()
            .set_directory(home_dir.as_path())
            .pick_file()
    })
    .map(|res| match res {
        Some(file) => Message::ChangeConfiguration(ConfigType::Whkd, ConfigState::Loaded(file)),
        None => Message::ClosedDialog,
    })
}

fn new_whkd() -> Task<Message> {
    let home_dir = whkd::home_path();
    Task::future(async move {
        rfd::FileDialog::new()
            .set_directory(home_dir.as_path())
            .save_file()
    })
    .map(|res| match res {
        Some(file) => Message::ChangeConfiguration(ConfigType::Whkd, ConfigState::New(file)),
        None => Message::ClosedDialog,
    })
}
