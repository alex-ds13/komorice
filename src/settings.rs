use crate::apperror::{AppError, AppErrorKind};
use crate::widget::opt_helpers;
use crate::BOLD_FONT;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_std::channel::{self, Receiver};
use iced::futures::{SinkExt, StreamExt};
use iced::theme::{Custom, Theme};
use iced::widget::{column, horizontal_rule, text};
use iced::{padding, Element, Fill, Subscription, Task};
use notify_debouncer_mini::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    #[serde(with = "ThemeDef")]
    pub theme: Theme,
    pub show_advanced: bool,
    #[serde(skip)]
    settings_watcher_tx: Option<async_std::channel::Sender<Input>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::TokyoNightStorm,
            show_advanced: false,
            settings_watcher_tx: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AppError(AppError),
    LoadedSettings(Settings),
    FailedToLoadSettings(AppError),
    SavedSettings,
    ThemeChanged(Theme),
    ShowAdvancedChanged(bool),
    SettingsFileWatcherTx(async_std::channel::Sender<Input>),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Error(AppError),
}

impl Settings {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::AppError(apperror) => {
                return (Action::Error(apperror), Task::none());
            }
            Message::LoadedSettings(settings) => {
                let sender = self.settings_watcher_tx.take();
                *self = Settings {
                    theme: settings.theme,
                    show_advanced: settings.show_advanced,
                    settings_watcher_tx: sender,
                };
            }
            Message::FailedToLoadSettings(apperror) => {
                return (Action::Error(apperror), Task::none());
            }
            Message::SavedSettings => {
                if let Some(sender) = &self.settings_watcher_tx {
                    let _ = sender.try_send(Input::IgnoreNextEvent);
                }
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                return (Action::None, save_task(self.clone()));
            }
            Message::ShowAdvancedChanged(show_advanced) => {
                self.show_advanced = show_advanced;
                return (Action::None, save_task(self.clone()));
            }
            Message::SettingsFileWatcherTx(sender) => self.settings_watcher_tx = Some(sender),
        }
        (Action::None, Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        let title = text("Settings:").size(20).font(*BOLD_FONT);
        let theme = opt_helpers::choose(
            "Theme:",
            Some("Theme for the Komorice app\n\nThis theme has nothing to do with komorebi!"),
            Theme::ALL,
            Some(&self.theme),
            Message::ThemeChanged,
        );
        let show_advanced = opt_helpers::toggle(
            "Show advanced options",
            Some("By default Komorice tries to be as simple as possible for new users by showing \
                only the simpler options that should be required to use and configure komorebi. If some option you \
                    want to configure isn't showing, enable this setting."),
                    self.show_advanced,
                    Message::ShowAdvancedChanged,
        );
        let col = column![theme, show_advanced]
            .spacing(10)
            .padding(padding::top(10).bottom(10).right(20));
        column![title, horizontal_rule(2.0), col]
            .spacing(10)
            .width(Fill)
            .height(Fill)
            .into()
    }
}

enum State {
    Starting,
    Ready(Data),
}

struct Data {
    debouncer: Debouncer<ReadDirectoryChangesWatcher>,
    receiver: Receiver<Input>,
    ignore_event: usize,
}

pub enum Input {
    IgnoreNextEvent,
    DebouncerRes(DebounceEventResult),
}

pub fn worker() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(10, move |mut output| async move {
            let mut state = State::Starting;

            loop {
                match state {
                    State::Starting => {
                        let (sender, receiver) = channel::bounded(10);

                        let sender_clone = sender.clone();
                        match output
                            .send(Message::SettingsFileWatcherTx(sender_clone))
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error trying to send the options watcher sender:\n{e:?}");
                            }
                        }

                        let mut debouncer = new_debouncer(Duration::from_millis(250), move |res| {
                            async_std::task::block_on(async {
                                let input = Input::DebouncerRes(res);
                                match sender.send(input).await {
                                    Ok(_) => {}
                                    Err(error) => {
                                        println!(
                                            "Error sending a debounced \
                                                event to the worker channel. \
                                                E: {error:?}"
                                        );
                                    }
                                }
                            })
                        })
                        .unwrap();

                        let path = config_path();
                        if matches!(std::fs::exists(&path), Ok(false) | Err(_)) {
                            if let Err(apperror) = save(Settings::default()).await {
                                match output.send(Message::AppError(apperror)).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("Error trying to send error:\n{e:?}");
                                    }
                                }
                            }
                        }
                        debouncer
                            .watcher()
                            .watch(&path, RecursiveMode::NonRecursive)
                            .unwrap();

                        state = State::Ready(Data {
                            debouncer,
                            receiver,
                            ignore_event: 0,
                        });
                    }
                    State::Ready(data) => {
                        let Data {
                            mut receiver,
                            debouncer,
                            mut ignore_event,
                        } = data;
                        let input = receiver.select_next_some().await;

                        match input {
                            Input::IgnoreNextEvent => {
                                println!("IgnoreNextEvent");
                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event: ignore_event + 1,
                                });
                            }
                            Input::DebouncerRes(res) => {
                                match res {
                                    Ok(events) => {
                                        events.iter().for_each(|event| {
                                            handle_event(event, &mut ignore_event, &mut output);
                                        });
                                    }
                                    Err(error) => {
                                        println!("Error from file watcher: {error:?}")
                                    }
                                }

                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event,
                                });
                            }
                        }
                    }
                }
            }
        })
    })
}

fn handle_event(
    event: &DebouncedEvent,
    ignore_event: &mut usize,
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
) {
    // println!("FileWatcher event: {event:?}");
    if let DebouncedEventKind::Any = event.kind {
        if *ignore_event == 0 {
            println!("FileWatcher: loading options");
            async_std::task::block_on(async {
                match load().await {
                    Ok(loaded_options) => {
                        let _ = output.send(Message::LoadedSettings(loaded_options)).await;
                    }
                    Err(e) => {
                        let _ = output.send(Message::AppError(e)).await;
                    }
                }
            });
        } else {
            println!("FileWatcher: ignoring event");
            *ignore_event = ignore_event.saturating_sub(1);
        }
    }
}

pub fn load_task() -> Task<Message> {
    Task::perform(load(), |res| match res {
        Ok(settings) => Message::LoadedSettings(settings),
        Err(apperror) => Message::FailedToLoadSettings(apperror),
    })
}

pub async fn load() -> Result<Settings, AppError> {
    use async_std::prelude::*;

    let mut contents = String::new();

    let file_open_res = async_std::fs::File::open(config_path()).await;

    let mut file = match file_open_res {
        Ok(file) => file,
        Err(error) => {
            println!("Failed to find 'komorice.json' file.\nError: {}", error);
            return Err(AppError {
                title: "Failed to find 'komorice.json' file.".into(),
                description: None,
                kind: AppErrorKind::Info,
            });
        }
    };

    file.read_to_string(&mut contents)
        .await
        .map_err(|e| AppError {
            title: "Error opening 'komorice.json' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    serde_json::from_str(&contents).map_err(|e| AppError {
        title: "Error reading 'komorice.json' file.".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })
}

pub fn save_task(settings: Settings) -> Task<Message> {
    Task::future(save(settings)).map(|res| match res {
        Ok(_) => Message::SavedSettings,
        Err(apperror) => Message::AppError(apperror),
    })
}

pub async fn save(settings: Settings) -> Result<(), AppError> {
    use async_std::prelude::*;

    let json = serde_json::to_string_pretty(&settings).map_err(|e| AppError {
        title: "Error writing to 'komorice.json' file".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })?;

    let path = config_path();

    if let Some(dir) = path.parent() {
        async_std::fs::create_dir_all(dir)
            .await
            .map_err(|e| AppError {
                title: "Error creating folder for 'komorice.json' file".into(),
                description: Some(e.to_string()),
                kind: AppErrorKind::Error,
            })?;
    }

    let mut file = async_std::fs::File::create(path)
        .await
        .map_err(|e| AppError {
            title: "Error creating 'komorice.json' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    file.write_all(json.as_bytes())
        .await
        .map_err(|e| AppError {
            title: "Error saving 'komorice.json' file".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    // This is a simple way to save at most once every couple seconds
    // async_std::task::sleep(std::time::Duration::from_secs(2)).await;

    Ok(())
}

pub fn config_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("there is no local data directory")
        .join("komorice")
        .join("settings.json")
}

/// A built-in theme.
#[derive(Deserialize, Serialize)]
#[serde(remote = "Theme")]
pub enum ThemeDef {
    /// The built-in light variant.
    Light,
    /// The built-in dark variant.
    Dark,
    /// The built-in Dracula variant.
    Dracula,
    /// The built-in Nord variant.
    Nord,
    /// The built-in Solarized Light variant.
    SolarizedLight,
    /// The built-in Solarized Dark variant.
    SolarizedDark,
    /// The built-in Gruvbox Light variant.
    GruvboxLight,
    /// The built-in Gruvbox Dark variant.
    GruvboxDark,
    /// The built-in Catppuccin Latte variant.
    CatppuccinLatte,
    /// The built-in Catppuccin Frappé variant.
    CatppuccinFrappe,
    /// The built-in Catppuccin Macchiato variant.
    CatppuccinMacchiato,
    /// The built-in Catppuccin Mocha variant.
    CatppuccinMocha,
    /// The built-in Tokyo Night variant.
    TokyoNight,
    /// The built-in Tokyo Night Storm variant.
    TokyoNightStorm,
    /// The built-in Tokyo Night Light variant.
    TokyoNightLight,
    /// The built-in Kanagawa Wave variant.
    KanagawaWave,
    /// The built-in Kanagawa Dragon variant.
    KanagawaDragon,
    /// The built-in Kanagawa Lotus variant.
    KanagawaLotus,
    /// The built-in Moonfly variant.
    Moonfly,
    /// The built-in Nightfly variant.
    Nightfly,
    /// The built-in Oxocarbon variant.
    Oxocarbon,
    /// The built-in Ferra variant:
    Ferra,
    /// A [`Theme`] that uses a [`Custom`] palette.
    #[serde(skip)]
    Custom(Arc<Custom>),
}
