#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod apperror;
mod config;
mod komo_interop;
mod screen;
mod settings;
mod utils;
mod whkd;
mod widget;

use crate::apperror::{AppError, AppErrorKind};
use crate::config::DEFAULT_CONFIG;
use crate::screen::{
    ConfigState, ConfigType, Configuration, Screen, View, animation, border, general, home,
    live_debug, monitors, rules, sidebar, stackbar, theme, transparency,
};
use crate::widget::{button_with_icon, icons, opt_helpers::to_description_text, tooltip};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use iced::{
    Center, Element, Fill, Font, Right, Shrink, Subscription, Task, Theme, padding,
    widget::{
        button, center, checkbox, column, container, opaque, rich_text, row, rule, scrollable,
        space, span, stack, text,
    },
};
use lazy_static::lazy_static;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

lazy_static! {
    static ref KOMOREBI_VERSION: &'static str = "v0.1.39";
    static ref DEFAULT_FONT: Font = Font::with_family("Segoe UI");
    static ref EMOJI_FONT: Font = Font::with_family("Segoe UI Emoji");
    static ref ITALIC_FONT: Font = {
        let mut f = Font::with_family("Segoe UI");
        f.style = iced::font::Style::Italic;
        f
    };
    static ref BOLD_FONT: Font = {
        let mut f = Font::with_family("Segoe UI");
        f.weight = iced::font::Weight::Bold;
        f
    };
    static ref NONE_STR: Arc<str> = Arc::from("[None]");
    static ref SCREENS_BACK_TO_START: [Screen; 3] =
        [Screen::Rules, Screen::Transparency, Screen::LiveDebug];
    static ref PATH_TIP_ID: &'static str = "configuration_path_tooltip_id";
    static ref SAVE_TIP_ID: &'static str = "configuration_save_tooltip_id";
    static ref TIME_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> =
        time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]-[second]",)
            .unwrap_or_default();
    static ref LOCAL_DIR: PathBuf = dirs::data_local_dir()
        .expect("there is no local data directory")
        .join("komorice");
}

fn main() -> iced::Result {
    let logs_folder = LOCAL_DIR.join("logs");
    let filter = EnvFilter::from_default_env();
    let mut _log_guard = None;
    smol::block_on(async {
        let _ = smol::fs::create_dir_all(&logs_folder).await;
    });
    let file_appender = if let Ok(file_appender) = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(8)
        .filename_suffix("komorice.log")
        .build(logs_folder)
    {
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        _log_guard = Some(_guard);
        Some(fmt::layer().with_writer(non_blocking).with_ansi(false))
    } else {
        None
    };
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .with(file_appender)
        .init();

    iced::application(Komorice::initialize, Komorice::update, Komorice::view)
        .title("Komorice")
        .subscription(Komorice::subscription)
        .theme(Komorice::theme)
        .default_font(*DEFAULT_FONT)
        .font(icons::FONT)
        .window(iced::window::Settings {
            icon: match iced::window::icon::from_rgba(
                include_bytes!("../assets/komorice.rgba").to_vec(),
                256,
                256,
            ) {
                Ok(icon) => Some(icon),
                Err(error) => {
                    log::error!("Error creating icon: {}", error);
                    None
                }
            },
            ..iced::window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    // Error Messages
    AppError(AppError),
    OpenErrorsModal,
    CloseErrorsModal,
    ClearErrors,

    // View/Screen related Messages
    Home(home::Message),
    Animation(animation::Message),
    Border(border::Message),
    General(general::Message),
    LiveDebug(live_debug::Message),
    Monitors(monitors::Message),
    Rules(rules::Message),
    Sidebar(sidebar::Message),
    Stackbar(stackbar::Message),
    Theme(theme::Message),
    Transparency(transparency::Message),
    Settings(settings::Message),
    Whkd(whkd::Message),

    // Config related Messages
    LoadedConfig(Arc<komorebi_client::StaticConfig>),
    FailedToLoadConfig(AppError),
    ConfigFileWatcherTx(smol::channel::Sender<config::Input>),
    ConfigWatcherError(AppError),

    // Bottom bar messages
    DiscardChanges,
    TrySave,
    ToggleSaveModal,
    Save,
    Saved,
    ToggleSaveAsDialog,
    SaveAsDialogClosed,
    SaveAs(PathBuf),
    Backup,
    BackupComplete,
    BackupFailed(AppError),
    OpenConfigFile,
    OpenConfigFolder,
}

struct Komorice {
    main_screen: Screen,
    configuration: Configuration,
    display_info: HashMap<usize, monitors::DisplayInfo>,
    sidebar: sidebar::Sidebar,
    home: home::Home,
    monitors: monitors::Monitors,
    border: border::Border,
    general: general::General,
    stackbar: stackbar::Stackbar,
    transparency: transparency::Transparency,
    animation: animation::Animation,
    theme_screen: theme::Theme,
    rules: rules::Rules,
    live_debug: live_debug::LiveDebug,
    settings: settings::Settings,
    whkd: whkd::Whkd,
    config: komorebi_client::StaticConfig,
    loaded_config: Arc<komorebi_client::StaticConfig>,
    is_dirty: bool,
    config_watcher_tx: Option<smol::channel::Sender<config::Input>>,
    errors: Vec<AppError>,
    show_save_modal: bool,
    show_save_as_dialog: bool,
    show_errors_modal: bool,
}

impl Default for Komorice {
    fn default() -> Self {
        Self {
            main_screen: Default::default(),
            configuration: Default::default(),
            sidebar: Default::default(),
            display_info: Default::default(),
            home: Default::default(),
            monitors: monitors::Monitors::new(&DEFAULT_CONFIG),
            border: Default::default(),
            general: Default::default(),
            stackbar: Default::default(),
            transparency: Default::default(),
            animation: Default::default(),
            theme_screen: Default::default(),
            rules: Default::default(),
            live_debug: Default::default(),
            settings: Default::default(),
            whkd: Default::default(),
            config: DEFAULT_CONFIG.clone(),
            loaded_config: Arc::new(DEFAULT_CONFIG.clone()),
            is_dirty: Default::default(),
            config_watcher_tx: Default::default(),
            errors: Default::default(),
            show_save_modal: Default::default(),
            show_save_as_dialog: Default::default(),
            show_errors_modal: Default::default(),
        }
    }
}

impl Komorice {
    pub fn initialize() -> (Self, Task<Message>) {
        let mut config = DEFAULT_CONFIG.clone();
        let loaded_config = Arc::new(config.clone());
        let display_info = monitors::get_display_information(&config.display_index_preferences);
        config::fill_monitors(&mut config, &display_info);
        let monitors = monitors::Monitors::new(&config);
        let (whkd, whkd_task) = whkd::Whkd::init();
        let init = Komorice {
            display_info,
            config,
            loaded_config,
            monitors,
            whkd,
            ..Default::default()
        };
        (
            init,
            Task::batch([
                settings::load_task().map(Message::Settings),
                config::load_task(config::config_path()),
                whkd::load_task(whkd::config_path()).map(Message::Whkd),
                whkd::load_commands().map(Message::Whkd),
                whkd_task.map(Message::Whkd),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppError(apperror) => self.add_error(apperror),
            Message::OpenErrorsModal => self.show_errors_modal = true,
            Message::CloseErrorsModal => self.show_errors_modal = false,
            Message::ClearErrors => {
                self.errors.clear();
                self.show_errors_modal = false;
            }
            Message::Home(message) => {
                let (action, task) = self.home.update(message, &mut self.configuration);
                let action_task = match action {
                    home::Action::None => Task::none(),
                    home::Action::ContinueEdit => {
                        self.main_screen = self
                            .sidebar
                            .selected_screen(&self.configuration.config_type);
                        Task::none()
                    }
                    home::Action::ChangedConfiguration => {
                        if !matches!(
                            self.configuration.state(self.configuration.config_type),
                            ConfigState::Loaded(_)
                        ) {
                            // When loading we don't want to update the screen until we
                            // successfully load the file.
                            self.main_screen = self
                                .sidebar
                                .selected_screen(&self.configuration.config_type);
                        }
                        match self.configuration.config_type {
                            ConfigType::Komorebi => match &self.configuration.komorebi_state {
                                ConfigState::Active => Task::none(),
                                ConfigState::Loaded(path) => config::load_task(path.clone()),
                                ConfigState::New(_) => {
                                    let mut config = DEFAULT_CONFIG.clone();
                                    self.display_info = monitors::get_display_information(
                                        &config.display_index_preferences,
                                    );
                                    config::fill_monitors(&mut config, &self.display_info);
                                    self.config = config;
                                    self.loaded_config = Arc::new(self.config.clone());
                                    self.monitors = monitors::Monitors::new(&self.config);
                                    self.is_dirty = false;
                                    Task::none()
                                }
                            },
                            ConfigType::Whkd => match &self.configuration.whkd_state {
                                ConfigState::Active => Task::none(),
                                ConfigState::Loaded(path) => {
                                    whkd::load_task(path.clone()).map(Message::Whkd)
                                }
                                ConfigState::New(_) => {
                                    self.whkd.load_default();
                                    Task::none()
                                }
                            },
                        }
                    }
                    home::Action::OpenErrorsModal => {
                        self.show_errors_modal = true;
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Home), action_task]);
            }
            Message::General(message) => {
                let (action, task) = self.general.update(message, &mut self.config);
                let action_task = match action {
                    general::Action::None => Task::none(),
                };
                self.check_changes();
                return Task::batch([task.map(Message::General), action_task]);
            }
            Message::Border(message) => {
                let (action, task) = self.border.update(message, &mut self.config);
                let action_task = match action {
                    border::Action::None => Task::none(),
                };
                self.check_changes();
                return Task::batch([task.map(Message::Border), action_task]);
            }
            Message::LiveDebug(message) => {
                let (action, task) = self.live_debug.update(message);
                let action_task = match action {
                    live_debug::Action::None => Task::none(),
                    live_debug::Action::Error(apperror) => {
                        self.add_error(apperror);
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::LiveDebug), action_task]);
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
                    return Task::batch([task.map(Message::Monitors), action_task]);
                }
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
                    return Task::batch([task.map(Message::Stackbar), action_task]);
                }
            }
            Message::Transparency(message) => {
                let (action, task) = self.transparency.update(message, &mut self.config);
                let action_task = match action {
                    transparency::Action::None => Task::none(),
                };
                self.check_changes();
                return Task::batch([task.map(Message::Transparency), action_task]);
            }
            Message::Settings(message) => {
                let (action, task) = self.settings.update(message);
                let action_task = match action {
                    settings::Action::None => Task::none(),
                    settings::Action::Error(apperror) => {
                        self.add_error(apperror);
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Settings), action_task]);
            }
            Message::Whkd(message) => {
                let (action, task) = self.whkd.update(message);
                let action_task = match action {
                    whkd::Action::None => Task::none(),
                    whkd::Action::SavedWhkdrc => {
                        self.configuration.saved_new_whkd = true;
                        Task::none()
                    }
                    whkd::Action::LoadedWhkdrc => {
                        self.configuration.has_loaded_whkd = true;
                        if self.home.loading.is_some() {
                            self.home.loading = None;
                            self.main_screen = self
                                .sidebar
                                .selected_screen(&self.configuration.config_type);
                        }
                        Task::none()
                    }
                    whkd::Action::FailedToLoadWhkdrc(app_error) => {
                        self.add_error(app_error);
                        if self.home.loading.is_some() {
                            self.home.loading = None;
                        }
                        Task::none()
                    }
                    whkd::Action::BackupComplete => {
                        //TODO: give feedback to user
                        tooltip::close(*SAVE_TIP_ID)
                    }
                    whkd::Action::BackupFailed(app_error) => {
                        self.add_error(app_error);
                        //TODO: give feedback to user
                        tooltip::close(*SAVE_TIP_ID)
                    }
                    whkd::Action::AppError(app_error) => {
                        self.add_error(app_error);
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Whkd), action_task]);
            }
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
                    return Task::batch([task.map(Message::Animation), action_task]);
                }
            }
            Message::Theme(message) => {
                let (action, task) = self.theme_screen.update(message, &mut self.config);
                let action_task = match action {
                    theme::Action::None => Task::none(),
                };
                self.check_changes();
                return Task::batch([task.map(Message::Theme), action_task]);
            }
            Message::Rules(message) => {
                let (action, task) = self.rules.update(message, &mut self.config);
                let action_task = match action {
                    rules::Action::None => Task::none(),
                };
                self.check_changes();
                return Task::batch([task.map(Message::Rules), action_task]);
            }
            Message::Sidebar(message) => {
                let (action, task) = self
                    .sidebar
                    .update(message, &self.configuration.config_type);
                let action_task = match action {
                    sidebar::Action::None => Task::none(),
                    sidebar::Action::SetHomeScreen => {
                        self.main_screen = Screen::Home;
                        Task::none()
                    }
                    sidebar::Action::UpdateMainScreen(screen) => {
                        if matches!(self.configuration.config_type, ConfigType::Whkd) {
                            self.whkd.screen = screen.clone();
                        }
                        self.main_screen = screen;
                        self.screen_to_start();
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Sidebar), action_task]);
            }
            Message::LoadedConfig(config) => {
                if let Some(config) = Arc::into_inner(config) {
                    log::debug!("Config Loaded");
                    log::trace!("Loaded config:\n{config:#?}");
                    let config = config::merge_default(config);
                    self.config = config.clone();
                    self.is_dirty = self.populate_monitors();
                    self.configuration.has_loaded_komorebi = true;
                    if self.home.loading.is_some() {
                        self.home.loading = None;
                        self.main_screen = self
                            .sidebar
                            .selected_screen(&self.configuration.config_type);
                    }
                    self.loaded_config = Arc::new(config);
                    //TODO: show message on app to load external changes
                }
            }
            Message::FailedToLoadConfig(apperror) => {
                self.add_error(apperror);
                if self.home.loading.is_some() {
                    self.home.loading = None;
                }
            }
            Message::ConfigFileWatcherTx(sender) => {
                self.config_watcher_tx = Some(sender);
            }
            Message::ConfigWatcherError(apperror) => self.add_error(apperror),
            Message::TrySave => {
                if self.settings.show_save_warning {
                    self.show_save_modal = true;
                } else {
                    match self.configuration.config_type {
                        ConfigType::Komorebi => {
                            self.configuration.saved_new_komorebi = true;
                            return config::save_task(
                                self.config.clone(),
                                self.configuration.path(),
                            );
                        }
                        ConfigType::Whkd => {
                            self.configuration.saved_new_whkd = true;
                            return whkd::save_task(
                                self.whkd.whkdrc.clone(),
                                self.configuration.path(),
                            )
                            .map(Message::Whkd);
                        }
                    }
                }
            }
            Message::ToggleSaveModal => {
                self.show_save_modal = !self.show_save_modal;
            }
            Message::Save => {
                self.show_save_modal = false;
                match self.configuration.config_type {
                    ConfigType::Komorebi => {
                        return config::save_task(self.config.clone(), self.configuration.path());
                    }
                    ConfigType::Whkd => {
                        return whkd::save_task(
                            self.whkd.whkdrc.clone(),
                            self.configuration.path(),
                        )
                        .map(Message::Whkd);
                    }
                }
            }
            Message::Saved => {
                if let Some(sender) = &self.config_watcher_tx {
                    let _ = sender.try_send(config::Input::IgnoreNextEvent);
                }
                self.loaded_config = Arc::new(self.config.clone());
                self.is_dirty = false;
                self.configuration.saved_new_komorebi = true;
            }
            Message::ToggleSaveAsDialog => {
                self.show_save_as_dialog = true;
                let dir = self.configuration.parent_path();
                let config_type = self.configuration.config_type;
                let dialog_task = Task::future(async move {
                    let mut dialog = rfd::FileDialog::new();
                    if matches!(config_type, ConfigType::Komorebi) {
                        dialog = dialog.add_filter("json", &["json"]);
                    }
                    dialog.set_directory(dir.as_path()).save_file()
                })
                .map(|res| match res {
                    Some(file) => Message::SaveAs(file),
                    None => Message::SaveAsDialogClosed,
                });

                return Task::batch([tooltip::close(*SAVE_TIP_ID), dialog_task]);
            }
            Message::SaveAsDialogClosed => {
                self.show_save_as_dialog = false;
                return tooltip::close(*SAVE_TIP_ID);
            }
            Message::SaveAs(file) => {
                self.show_save_as_dialog = false;
                match self.configuration.config_type {
                    ConfigType::Komorebi => {
                        self.configuration.komorebi_state = ConfigState::New(file);
                        return config::save_task(self.config.clone(), self.configuration.path());
                    }
                    ConfigType::Whkd => {
                        self.configuration.whkd_state = ConfigState::New(file);
                        return whkd::save_task(
                            self.whkd.whkdrc.clone(),
                            self.configuration.path(),
                        )
                        .map(Message::Whkd);
                    }
                }
            }
            Message::Backup => {
                if let Ok(now) = time::OffsetDateTime::now_local()
                    && let Ok(now_str) = now.format(&TIME_FORMAT)
                {
                    let bck_file_name = self
                        .configuration
                        .path()
                        .with_extension(format!("{}.bkp", now_str));
                    match self.configuration.config_type {
                        ConfigType::Komorebi => {
                            return config::backup_task(self.config.clone(), bck_file_name);
                        }
                        ConfigType::Whkd => {
                            return whkd::backup_task(self.whkd.whkdrc.clone(), bck_file_name)
                                .map(Message::Whkd);
                        }
                    }
                }
            }
            Message::BackupComplete => {
                //TODO: give feedback to user
                return tooltip::close(*SAVE_TIP_ID);
            }
            Message::BackupFailed(apperror) => {
                self.add_error(apperror);
                //TODO: give feedback to user
                return tooltip::close(*SAVE_TIP_ID);
            }
            Message::DiscardChanges => match self.configuration.config_type {
                ConfigType::Komorebi => {
                    let update_display_info = self.config.display_index_preferences
                        != self.loaded_config.display_index_preferences;
                    self.config = (*self.loaded_config).clone();
                    self.is_dirty = false;
                    if update_display_info {
                        self.display_info = monitors::get_display_information(
                            &self.config.display_index_preferences,
                        );
                    }
                }
                ConfigType::Whkd => self.whkd.discard_changes(),
            },
            Message::OpenConfigFile => {
                let file = self.configuration.path().clone();
                return Task::batch([
                    Task::future(async {
                        smol::unblock(move || open::that_in_background(file).join()).await
                    })
                    .discard(),
                    widget::tooltip::close(*PATH_TIP_ID),
                ]);
            }
            Message::OpenConfigFolder => {
                if let Some(parent) = self.configuration.path().parent().map(|p| p.to_path_buf()) {
                    return Task::batch([
                        Task::future(async {
                            smol::unblock(move || open::that_in_background(parent).join()).await
                        })
                        .discard(),
                        widget::tooltip::close(*PATH_TIP_ID),
                    ]);
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let main_screen: View<Message> = match self.main_screen {
            Screen::Home => self
                .home
                .view(&self.configuration, !self.errors.is_empty())
                .map(Message::Home)
                .into(),
            Screen::General => self
                .general
                .view(&self.config, self.settings.show_advanced)
                .map(Message::General),
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
            Screen::Border => self.border.view(&self.config).map(Message::Border).into(),
            Screen::Stackbar => self
                .stackbar
                .view(self.config.stackbar.as_ref(), self.config.theme.as_ref())
                .map(Message::Stackbar)
                .into(),
            Screen::Transparency => self
                .transparency
                .view(&self.config)
                .map(Message::Transparency)
                .into(),
            Screen::Animations => self
                .animation
                .view(self.config.animation.as_ref())
                .map(Message::Animation)
                .into(),
            Screen::Theme => self
                .theme_screen
                .view(&self.config)
                .map(Message::Theme)
                .into(),
            Screen::Rules => self
                .rules
                .view(&self.config, self.settings.show_advanced)
                .map(Message::Rules)
                .into(),
            Screen::LiveDebug => self.live_debug.view().map(Message::LiveDebug).into(),
            Screen::Settings => self.settings.view().map(Message::Settings).into(),
            Screen::Whkd | Screen::WhkdBindings | Screen::WhkdAppBindings => {
                self.whkd.view(&self.settings.theme).map(Message::Whkd)
            }
        };

        let skip_side_bottom_bars = matches!(self.main_screen, Screen::Home);

        let main_content = if !skip_side_bottom_bars {
            let sidebar = self
                .sidebar
                .view(&self.configuration.config_type)
                .map(Message::Sidebar);
            let save_buttons = self.save_buttons();
            let right_col = column![
                container(main_screen.element)
                    .height(Fill)
                    .padding(padding::all(20).bottom(0)),
                container(rule::horizontal(2.0)).padding(padding::bottom(5)),
                save_buttons,
            ];

            let main_content = row![sidebar, rule::vertical(2.0), right_col].padding(10);
            if let Some(screen_modal) = main_screen.modal {
                widget::modal(
                    main_content,
                    screen_modal.element,
                    screen_modal.close_message,
                )
            } else {
                main_content.into()
            }
        } else {
            if let Some(screen_modal) = main_screen.modal {
                widget::modal(
                    main_screen.element,
                    screen_modal.element,
                    screen_modal.close_message,
                )
            } else {
                main_screen.element
            }
        };

        let save_modal_content = self.show_save_modal.then(|| self.save_warning());
        let with_save_modal =
            widget::modal(main_content, save_modal_content, Message::ToggleSaveModal);
        let errors_modal_content = self.show_errors_modal.then(|| self.errors_modal());
        let with_errors_modal = widget::modal(
            with_save_modal,
            errors_modal_content,
            Message::CloseErrorsModal,
        );
        stack![
            with_errors_modal,
            self.show_save_as_dialog
                .then(|| opaque(center("").style(|t| {
                    container::Style {
                        background: Some(iced::color!(0x000000, 0.5).into()),
                        ..container::dark(t)
                    }
                }))),
        ]
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let screen_subscription = match self.main_screen {
            Screen::Home
            | Screen::General
            | Screen::Border
            | Screen::Stackbar
            | Screen::Animations
            | Screen::Theme
            | Screen::LiveDebug
            | Screen::Settings => Subscription::none(),
            Screen::Monitors => self.monitors.subscription().map(Message::Monitors),
            Screen::Transparency => self.transparency.subscription().map(Message::Transparency),
            Screen::Rules => self.rules.subscription().map(Message::Rules),
            Screen::Whkd | Screen::WhkdBindings | Screen::WhkdAppBindings => self
                .whkd
                .subscription(&self.configuration)
                .map(Message::Whkd),
        };

        let worker = if matches!(self.configuration.config_type, ConfigType::Komorebi)
            && (!matches!(self.configuration.komorebi_state, ConfigState::New(_))
                || self.configuration.saved_new_komorebi)
        {
            // Only start the worker if has the config_type as `Komorebi` and in case the komorebi state is
            // `New` the worker should only run if it has already been saved once at least.
            config::worker(self.configuration.path())
        } else {
            Subscription::none()
        };

        Subscription::batch([
            komo_interop::connect().map(Message::LiveDebug),
            worker,
            settings::worker().map(Message::Settings),
            screen_subscription,
        ])
    }

    pub fn theme(&self) -> Theme {
        self.settings.theme.clone()
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

    fn is_unsaved(&self) -> bool {
        match self.configuration.config_type {
            ConfigType::Komorebi => match self.configuration.komorebi_state {
                ConfigState::Active | ConfigState::Loaded(_) => self.is_dirty,
                ConfigState::New(_) => self.is_dirty || !self.configuration.saved_new_komorebi,
            },
            ConfigType::Whkd => match self.configuration.whkd_state {
                ConfigState::Active | ConfigState::Loaded(_) => self.whkd.is_dirty,
                ConfigState::New(_) => self.whkd.is_dirty || !self.configuration.saved_new_whkd,
            },
        }
    }

    fn is_dirty(&self) -> bool {
        match self.configuration.config_type {
            ConfigType::Komorebi => self.is_dirty,
            ConfigType::Whkd => self.whkd.is_dirty,
        }
    }

    /// Checks if the current screen allows going back to its first starting screen and if it does,
    /// it applies the `to_start_screen` function to that screen.
    fn screen_to_start(&mut self) {
        if SCREENS_BACK_TO_START.contains(&self.main_screen) {
            match self.main_screen {
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
                    self.whkd = Default::default()
                }
            }
        }
    }

    fn save_warning(&self) -> container::Container<'_, Message> {
        let save = button("Save").on_press_maybe(self.is_unsaved().then_some(Message::Save));
        let cancel = button("Cancel")
            .on_press(Message::ToggleSaveModal)
            .style(button::secondary);
        let stop_showing = container(
            Element::from(
                checkbox(!self.settings.show_save_warning)
                    .label("Don't show this message again")
                    .on_toggle(|v| settings::Message::ChangedShowSaveWarning(!v)),
            )
            .map(Message::Settings),
        )
        .align_left(Fill);
        let buttons = container(row![save, cancel].spacing(10)).align_right(Fill);
        let title = text("Save Config").size(20).font(*BOLD_FONT);
        let description = rich_text![
            "When saving the config file, it will overwrite the existing config. ",
            "This means you'll lose any comments you had and all default configs will be removed.",
            "\n\n",
            span("It is recommended that you backup your config before using komorice!")
                .font(*BOLD_FONT),
        ]
        .on_link_click(iced::never);
        let content = column![title, description, row![stop_showing, buttons]].spacing(20);
        container(
            container(content)
                .padding(20)
                .max_width(850.0)
                .center(iced::Shrink)
                .style(widget::modal::default),
        )
        .padding(20)
    }

    fn add_error(&mut self, apperror: AppError) {
        match apperror.kind {
            AppErrorKind::Info => log::info!("Info: {apperror:#?}"),
            AppErrorKind::Warning => log::warn!("Warning: {apperror:#?}"),
            AppErrorKind::Error => {
                log::error!("Error: {apperror:#?}");
                self.show_errors_modal = true;
            }
        }
        self.errors.push(apperror);
    }

    fn errors_modal(&self) -> container::Container<'_, Message> {
        let mut errors_column = column![
            row![
                text("Errors").size(30.0),
                space::horizontal(),
                button(text("❌").font(*EMOJI_FONT))
                    .on_press(Message::CloseErrorsModal)
                    .style(button::text),
            ]
            .spacing(10)
            .padding([10, 0])
            .align_y(Center),
        ]
        .spacing(10);

        let initial_col = column![].spacing(10).padding(padding::all(5.0).right(20.0));
        let errors = container(scrollable(
            self.errors
                .iter()
                .fold(initial_col, |c, e| c.push(e.view())),
        ))
        .max_height(350.0);

        errors_column = errors_column.push(errors);

        errors_column = errors_column.push(
            column![
                button_with_icon(icons::delete(), "Clear")
                    .on_press(Message::ClearErrors)
                    .style(button::danger)
            ]
            .width(Fill)
            .align_x(Right),
        );

        container(errors_column)
            .padding(20)
            .max_width(850.0)
            .center(iced::Fill)
            .height(iced::Shrink)
            .style(widget::modal::red)
    }

    fn save_buttons(&self) -> row::Row<'_, Message> {
        let mut save_buttons = row![]
            .spacing(10)
            .padding(padding::left(10))
            .width(Fill)
            .align_y(Center);
        save_buttons = save_buttons.push((!self.errors.is_empty()).then(|| {
            button_with_icon(icons::error(), "Errors")
                .on_press(Message::OpenErrorsModal)
                .style(button::danger)
        }));
        save_buttons = save_buttons.extend([
            space::horizontal().into(),
            tooltip(
                to_description_text(text!("{}", self.configuration.path().display())),
                container(
                    column![
                        button("Open File")
                            .width(Fill)
                            .on_press(Message::OpenConfigFile)
                            .style(|t, s| {
                                match s {
                                    button::Status::Active => button::text(t, s),
                                    button::Status::Hovered
                                    | button::Status::Pressed
                                    | button::Status::Disabled => button::background(t, s),
                                }
                            }),
                        button("Open Folder")
                            .on_press(Message::OpenConfigFolder)
                            .style(|t, s| {
                                match s {
                                    button::Status::Active => button::text(t, s),
                                    button::Status::Hovered
                                    | button::Status::Pressed
                                    | button::Status::Disabled => button::background(t, s),
                                }
                            }),
                    ]
                    .width(Shrink),
                )
                .style(container::bordered_box)
                .padding(10),
            )
            .id(*PATH_TIP_ID)
            .open(tooltip::Open::RightPointer)
            .into(),
            space::horizontal().into(),
            row![
                button("Save")
                    .on_press_maybe(self.is_unsaved().then_some(Message::TrySave))
                    .style(|t, s| button::Style {
                        border: iced::Border {
                            radius: iced::border::left(2),
                            ..button::primary(t, s).border
                        },
                        ..button::primary(t, s)
                    }),
                tooltip(
                    icons::down_chevron(),
                    container(
                        column![
                            button("Backup")
                                .width(Fill)
                                .on_press(Message::Backup)
                                .style(|t, s| {
                                    match s {
                                        button::Status::Active => button::text(t, s),
                                        button::Status::Hovered
                                        | button::Status::Pressed
                                        | button::Status::Disabled => button::background(t, s),
                                    }
                                }),
                            button("Save As")
                                .on_press(Message::ToggleSaveAsDialog)
                                .style(|t, s| {
                                    match s {
                                        button::Status::Active => button::text(t, s),
                                        button::Status::Hovered
                                        | button::Status::Pressed
                                        | button::Status::Disabled => button::background(t, s),
                                    }
                                }),
                        ]
                        .width(Shrink),
                    )
                    .style(container::bordered_box)
                    .padding(10),
                )
                .id(*SAVE_TIP_ID)
                .open(tooltip::Open::LeftPointer)
                .position(tooltip::Position::TopRight)
                .content_style(|t, s| tooltip::Style {
                    border: iced::Border {
                        radius: iced::border::right(2),
                        ..tooltip::primary(t, s).border
                    },
                    ..tooltip::primary(t, s)
                })
            ]
            .into(),
            button("Discard Changes")
                .on_press_maybe(self.is_dirty().then_some(Message::DiscardChanges))
                .style(button::secondary)
                .into(),
        ]);
        save_buttons
    }
}
