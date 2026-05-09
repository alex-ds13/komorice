pub mod aspect_ratio;
pub mod config;
mod connect;
pub mod layout;

pub use config::{DEFAULT_CONFIG, backup_task, config_path, home_path, load_task, save_task};
pub use connect::connect;

use crate::apperror::AppError;
use crate::screen::{ConfigState, ConfigType, Configuration, Screen, View, komorebi::*};
use crate::settings::Settings;

use std::collections::HashMap;
use std::sync::Arc;

use iced::{Subscription, Task, widget::space};

#[derive(Debug, Clone)]
pub enum Message {
    // View/Screen related Messages
    Animation(animation::Message),
    Border(border::Message),
    General(general::Message),
    LiveDebug(live_debug::Message),
    Monitors(monitors::Message),
    Rules(rules::Message),
    Stackbar(stackbar::Message),
    Theme(theme::Message),
    Transparency(transparency::Message),

    // Config related Messages
    Saved,
    Loaded(Arc<komorebi_client::StaticConfig>),
    FailedToLoad(AppError),
    BackupComplete,
    BackupFailed(AppError),
    ConfigFileWatcherTx(smol::channel::Sender<config::Input>),
    ConfigWatcherError(AppError),

    // Misc
    AppError(AppError),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Saved,
    Loaded,
    FailedToLoad(AppError),
    BackupComplete,
    BackupFailed(AppError),
    AppError(AppError),
}

pub struct Komorebi {
    pub screen: Screen,
    monitors: monitors::Monitors,
    border: border::Border,
    general: general::General,
    stackbar: stackbar::Stackbar,
    transparency: transparency::Transparency,
    animation: animation::Animation,
    theme_screen: theme::Theme,
    rules: rules::Rules,
    live_debug: live_debug::LiveDebug,

    config_watcher_tx: Option<smol::channel::Sender<config::Input>>,
    pub config: komorebi_client::StaticConfig,
    loaded_config: Arc<komorebi_client::StaticConfig>,
    pub is_dirty: bool,
    display_info: HashMap<usize, monitors::DisplayInfo>,
}

impl Default for Komorebi {
    fn default() -> Self {
        Self {
            screen: Default::default(),
            monitors: monitors::Monitors::new(&DEFAULT_CONFIG),
            border: Default::default(),
            general: Default::default(),
            stackbar: Default::default(),
            transparency: Default::default(),
            animation: Default::default(),
            theme_screen: Default::default(),
            rules: Default::default(),
            live_debug: Default::default(),
            config: DEFAULT_CONFIG.clone(),
            loaded_config: Arc::new(DEFAULT_CONFIG.clone()),
            is_dirty: Default::default(),
            config_watcher_tx: Default::default(),
            display_info: Default::default(),
        }
    }
}

impl Komorebi {
    pub fn init() -> (Self, Task<Message>) {
        let mut config = DEFAULT_CONFIG.clone();
        let loaded_config = Arc::new(config.clone());
        let display_info = monitors::get_display_information(&config.display_index_preferences);
        config::fill_monitors(&mut config, &display_info);
        let monitors = monitors::Monitors::new(&config);
        let init = Komorebi {
            display_info,
            config,
            loaded_config,
            monitors,
            ..Default::default()
        };
        (init, config::load_task(config::config_path()))
    }

    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::Animation(message) => {
                if self.config.animation.is_none() {
                    self.config.animation = Some(animation::default_animations_config());
                }
                if let Some(animation_config) = self.config.animation.as_mut() {
                    let (action, task) = self.animation.update(message, animation_config);
                    let action_task = match action {
                        animation::Action::None => Task::none(),
                    };
                    self.check_changes();
                    return (
                        Action::None,
                        Task::batch([task.map(Message::Animation), action_task]),
                    );
                }
            }
            Message::Border(message) => {
                let (action, task) = self.border.update(message, &mut self.config);
                let action_task = match action {
                    border::Action::None => Task::none(),
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::Border), action_task]),
                );
            }
            Message::General(message) => {
                let (action, task) = self.general.update(message, &mut self.config);
                let action_task = match action {
                    general::Action::None => Task::none(),
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::General), action_task]),
                );
            }
            Message::LiveDebug(message) => {
                let (action, task) = self.live_debug.update(message);
                let (action, action_task) = match action {
                    live_debug::Action::None => (Action::None, Task::none()),
                    live_debug::Action::Error(apperror) => {
                        (Action::AppError(apperror), Task::none())
                    }
                };
                return (
                    action,
                    Task::batch([task.map(Message::LiveDebug), action_task]),
                );
            }
            Message::Monitors(message) => {
                if let Some(monitors_config) = &mut self.config.monitors {
                    let (action, task) = self.monitors.update(
                        message,
                        monitors_config,
                        &mut self.config.display_index_preferences,
                        &mut self.display_info,
                    );
                    let action_task = match action {
                        monitors::Action::None => Task::none(),
                    };
                    self.check_changes();
                    return (
                        Action::None,
                        Task::batch([task.map(Message::Monitors), action_task]),
                    );
                }
            }
            Message::Rules(message) => {
                let (action, task) = self.rules.update(message, &mut self.config);
                let action_task = match action {
                    rules::Action::None => Task::none(),
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::Rules), action_task]),
                );
            }
            Message::Stackbar(message) => {
                if self.config.stackbar.is_none() {
                    self.config.stackbar = Some(stackbar::default_stackbar_config());
                }
                if let Some(stackbar_config) = self.config.stackbar.as_mut() {
                    let (action, task) = self.stackbar.update(message, stackbar_config);
                    let action_task = match action {
                        stackbar::Action::None => Task::none(),
                    };
                    self.check_changes();
                    return (
                        Action::None,
                        Task::batch([task.map(Message::Stackbar), action_task]),
                    );
                }
            }
            Message::Theme(message) => {
                let (action, task) = self.theme_screen.update(message, &mut self.config);
                let action_task = match action {
                    theme::Action::None => Task::none(),
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::Theme), action_task]),
                );
            }
            Message::Transparency(message) => {
                let (action, task) = self.transparency.update(message, &mut self.config);
                let action_task = match action {
                    transparency::Action::None => Task::none(),
                };
                self.check_changes();
                return (
                    Action::None,
                    Task::batch([task.map(Message::Transparency), action_task]),
                );
            }
            Message::Saved => {
                if let Some(sender) = &self.config_watcher_tx {
                    let _ = sender.try_send(config::Input::IgnoreNextEvent);
                }
                self.loaded_config = Arc::new(self.config.clone());
                self.is_dirty = false;
                return (Action::Saved, Task::none());
            }
            Message::Loaded(config) => {
                if let Some(config) = Arc::into_inner(config) {
                    log::debug!("Config Loaded");
                    log::trace!("Loaded config:\n{config:#?}");
                    let config = config::merge_default(config);
                    self.config = config.clone();
                    self.is_dirty = self.populate_monitors();
                    self.loaded_config = Arc::new(config);
                    //TODO: show message on app to load external changes
                    return (Action::Loaded, Task::none());
                }
            }
            Message::FailedToLoad(apperror) => {
                return (Action::FailedToLoad(apperror), Task::none());
            }
            Message::BackupComplete => return (Action::BackupComplete, Task::none()),
            Message::BackupFailed(app_error) => {
                return (Action::BackupFailed(app_error), Task::none());
            }
            Message::ConfigFileWatcherTx(sender) => {
                self.config_watcher_tx = Some(sender);
            }
            Message::ConfigWatcherError(apperror) => {
                return (Action::AppError(apperror), Task::none());
            }
            Message::AppError(apperror) => {
                return (Action::AppError(apperror), Task::none());
            }
        }
        (Action::None, Task::none())
    }

    pub fn view(&self, settings: &Settings) -> View<'_, Message> {
        match self.screen {
            Screen::Animations => self
                .animation
                .view(self.config.animation.as_ref())
                .map(Message::Animation)
                .into(),
            Screen::Border => self.border.view(&self.config).map(Message::Border).into(),
            Screen::General => self
                .general
                .view(&self.config, settings.show_advanced)
                .map(Message::General),
            Screen::LiveDebug => self.live_debug.view().map(Message::LiveDebug).into(),
            Screen::Monitors => {
                if let Some(monitors_config) = &self.config.monitors {
                    self.monitors
                        .view(
                            monitors_config,
                            &self.display_info,
                            &self.config.display_index_preferences,
                        )
                        .map(Message::Monitors)
                } else {
                    space::horizontal().into()
                }
            }
            Screen::Rules => self
                .rules
                .view(&self.config, settings.show_advanced)
                .map(Message::Rules)
                .into(),
            Screen::Stackbar => self
                .stackbar
                .view(self.config.stackbar.as_ref(), self.config.theme.as_ref())
                .map(Message::Stackbar)
                .into(),
            Screen::Theme => self
                .theme_screen
                .view(&self.config)
                .map(Message::Theme)
                .into(),
            Screen::Transparency => self
                .transparency
                .view(&self.config)
                .map(Message::Transparency)
                .into(),
            _ => space::horizontal().into(),
        }
    }

    pub fn subscription(&self, configuration: &Configuration) -> Subscription<Message> {
        let screen_subscription = match self.screen {
            Screen::Animations
            | Screen::Border
            | Screen::General
            | Screen::Home
            | Screen::LiveDebug
            | Screen::Settings
            | Screen::Stackbar
            | Screen::Theme
            | Screen::Whkd
            | Screen::WhkdBindings
            | Screen::WhkdAppBindings => Subscription::none(),
            Screen::Monitors => self.monitors.subscription().map(Message::Monitors),
            Screen::Rules => self.rules.subscription().map(Message::Rules),
            Screen::Transparency => self.transparency.subscription().map(Message::Transparency),
        };

        let worker = if matches!(configuration.config_type, ConfigType::Komorebi)
            && (!matches!(configuration.komorebi_state, ConfigState::New(_))
                || configuration.saved_new_komorebi)
        {
            // Only start the worker if has the config_type as `Komorebi` and in case the komorebi state is
            // `New` the worker should only run if it has already been saved once at least.
            config::worker(configuration.path())
        } else {
            Subscription::none()
        };

        Subscription::batch([
            connect().map(Message::LiveDebug),
            worker,
            screen_subscription,
        ])
    }

    /// Tries to create a `Monitor` and a `MonitorConfig` for each physical monitor that it detects
    /// in case the loaded config doesn't have it already.
    /// Returns wether or not `fill_monitors` made any changes to the config.
    fn populate_monitors(&mut self) -> bool {
        self.display_info =
            monitors::get_display_information(&self.config.display_index_preferences);
        let made_changes = config::fill_monitors(&mut self.config, &self.display_info);
        self.monitors = monitors::Monitors::new(&self.config);
        made_changes
    }

    fn check_changes(&mut self) {
        self.is_dirty = self.config != *self.loaded_config;
    }

    pub fn load_default(&mut self) {
        let mut config = DEFAULT_CONFIG.clone();
        self.display_info = monitors::get_display_information(&config.display_index_preferences);
        config::fill_monitors(&mut config, &self.display_info);
        self.config = config;
        self.loaded_config = Arc::new(self.config.clone());
        self.monitors = monitors::Monitors::new(&self.config);
        self.is_dirty = false;
    }

    pub fn discard_changes(&mut self) {
        let update_display_info =
            self.config.display_index_preferences != self.loaded_config.display_index_preferences;
        self.config = (*self.loaded_config).clone();
        self.is_dirty = false;
        if update_display_info {
            self.display_info =
                monitors::get_display_information(&self.config.display_index_preferences);
        }
    }

    /// Applies the `to_start_screen` function to the provided screen.
    pub fn screen_to_start(&mut self) {
        match self.screen {
            Screen::Home => {
                unreachable!("should never try to reset home screen!")
            }
            Screen::General => self.general = general::General::default(),
            Screen::Monitors => self.monitors = monitors::Monitors::new(&self.config),
            Screen::Border => self.border = border::Border::default(),
            Screen::Stackbar => self.stackbar = stackbar::Stackbar::default(),
            Screen::Transparency => self.transparency = transparency::Transparency::default(),
            Screen::Animations => self.animation = animation::Animation,
            Screen::Theme => self.theme_screen = theme::Theme::default(),
            Screen::Rules => self.rules = rules::Rules::default(),
            Screen::LiveDebug => self.live_debug.goto_start_screen(),
            Screen::Settings => {
                unreachable!("should never try to reset settings screen!")
            }
            Screen::Whkd | Screen::WhkdBindings | Screen::WhkdAppBindings => {
                unreachable!("should never try to reset whkd screens!")
            }
        }
    }
}
