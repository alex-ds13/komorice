mod apperror;
mod config;
mod komo_interop;
mod screen;
mod settings;
mod utils;
mod widget;

use crate::apperror::AppError;
use crate::config::DEFAULT_CONFIG;
use crate::screen::{
    animation, border, general, live_debug, monitors, rules, sidebar, stackbar, theme,
    transparency, Screen,
};

use std::collections::HashMap;
use std::sync::Arc;

use iced::widget::{button, center, horizontal_space, stack, vertical_space};
use iced::{
    padding,
    widget::{column, container, horizontal_rule, row, text, vertical_rule},
    Center, Element, Fill, Font, Subscription, Task, Theme,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_FONT: Font = Font::with_name("Segoe UI");
    static ref EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");
    static ref ITALIC_FONT: Font = {
        let mut f = Font::with_name("Segoe UI");
        f.style = iced::font::Style::Italic;
        f
    };
    static ref BOLD_FONT: Font = {
        let mut f = Font::with_name("Segoe UI");
        f.weight = iced::font::Weight::Bold;
        f
    };
    static ref NONE_STR: Arc<str> = Arc::from("[None]");
    static ref SCREENS_TO_RESET: [Screen; 3] =
        [Screen::Rules, Screen::Transparency, Screen::LiveDebug];
}

fn main() -> iced::Result {
    iced::application("Komorice", Komorice::update, Komorice::view)
        .subscription(Komorice::subscription)
        .theme(Komorice::theme)
        .default_font(*DEFAULT_FONT)
        .font(iced_aw::iced_fonts::REQUIRED_FONT_BYTES)
        .font(include_bytes!("../assets/icons.ttf"))
        .run_with(Komorice::initialize)
}

#[derive(Debug, Clone)]
enum Message {
    // General App Messages
    AppError(AppError),

    // View/Screen related Messages
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

    // Config related Messages
    LoadedConfig(Arc<komorebi_client::StaticConfig>),
    FailedToLoadConfig(AppError),
    ConfigFileWatcherTx(async_std::channel::Sender<config::Input>),
    DiscardChanges,
    Save,
    Saved,
}

struct Komorice {
    sidebar: sidebar::Sidebar,
    main_screen: Screen,
    display_info: HashMap<usize, monitors::DisplayInfo>,
    monitors: monitors::Monitors,
    border: border::Border,
    general: general::General,
    stackbar: stackbar::Stackbar,
    transparency: transparency::Transparency,
    animation: animation::Animation,
    theme_screen: theme::Theme,
    rules: rules::Rules,
    live_debug: live_debug::LiveDebug,
    config: komorebi_client::StaticConfig,
    has_loaded_config: bool,
    loaded_config: Arc<komorebi_client::StaticConfig>,
    is_dirty: bool,
    config_watcher_tx: Option<async_std::channel::Sender<config::Input>>,
    errors: Vec<AppError>,
    settings: settings::Settings,
}

impl Default for Komorice {
    fn default() -> Self {
        Self {
            sidebar: Default::default(),
            main_screen: Default::default(),
            display_info: Default::default(),
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
            has_loaded_config: Default::default(),
            loaded_config: Arc::new(DEFAULT_CONFIG.clone()),
            is_dirty: Default::default(),
            config_watcher_tx: Default::default(),
            errors: Default::default(),
            settings: Default::default(),
        }
    }
}

impl Komorice {
    pub fn initialize() -> (Self, Task<Message>) {
        let mut config = DEFAULT_CONFIG.clone();
        let loaded_config = Arc::new(config.clone());
        let display_info = monitors::get_display_information(&config.display_index_preferences);
        config::fill_monitors(&mut config, &display_info);
        let mut init = Komorice {
            display_info,
            config,
            loaded_config,
            ..Default::default()
        };
        init.populate_monitors();
        (
            init,
            Task::batch([
                settings::load_task().map(Message::Settings),
                Task::perform(config::load(), |res| match res {
                    Ok(config) => Message::LoadedConfig(Arc::new(config)),
                    Err(apperror) => Message::FailedToLoadConfig(apperror),
                }),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppError(apperror) => {
                println!("Received AppError: {apperror:#?}");
                self.errors.push(apperror);
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
                        println!("Received AppError: {apperror:#?}");
                        self.errors.push(apperror);
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
                        println!("Received AppError: {apperror:#?}");
                        self.errors.push(apperror);
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Settings), action_task]);
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
                let (action, task) = self.sidebar.update(message);
                let action_task = match action {
                    sidebar::Action::None => Task::none(),
                    sidebar::Action::UpdateMainScreen(screen) => {
                        if SCREENS_TO_RESET.contains(&screen) {
                            match screen {
                                Screen::Home => {
                                    unreachable!("should never try to reset home screen!")
                                }
                                Screen::General => self.general = general::General::default(),
                                Screen::Monitors => {
                                    self.monitors = monitors::Monitors::new(&self.config)
                                }
                                Screen::Border => self.border = border::Border::default(),
                                Screen::Stackbar => self.stackbar = stackbar::Stackbar::default(),
                                Screen::Transparency => {
                                    self.transparency = transparency::Transparency::default()
                                }
                                Screen::Animations => {
                                    self.animation = animation::Animation::default()
                                }
                                Screen::Theme => self.theme_screen = theme::Theme::default(),
                                Screen::Rules => self.rules = rules::Rules::default(),
                                Screen::LiveDebug => self.live_debug.reset_screen(),
                                Screen::Settings => {
                                    unreachable!("should never try to reset settings screen!")
                                }
                            }
                        }
                        self.main_screen = screen;
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Sidebar), action_task]);
            }
            Message::LoadedConfig(config) => {
                if let Some(config) = Arc::into_inner(config) {
                    println!("Config Loaded: {config:#?}");
                    let config = config::merge_default(config);
                    self.config = config.clone();
                    self.is_dirty = self.populate_monitors();
                    self.has_loaded_config = true;
                    self.loaded_config = Arc::new(config);
                    //TODO: show message on app to load external changes
                }
            }
            Message::FailedToLoadConfig(apperror) => {
                println!("Received AppError: {apperror:#?}");
                self.errors.push(apperror);
            }
            Message::ConfigFileWatcherTx(sender) => {
                self.config_watcher_tx = Some(sender);
            }
            Message::Save => {
                return Task::future(config::save(self.config.clone())).map(|res| match res {
                    Ok(_) => Message::Saved,
                    Err(apperror) => Message::AppError(apperror),
                });
            }
            Message::Saved => {
                if let Some(sender) = &self.config_watcher_tx {
                    let _ = sender.try_send(config::Input::IgnoreNextEvent);
                }
                self.loaded_config = Arc::new(self.config.clone());
                self.is_dirty = false;
            }
            Message::DiscardChanges => {
                let update_display_info = self.config.display_index_preferences
                    != self.loaded_config.display_index_preferences;
                self.config = (*self.loaded_config).clone();
                self.is_dirty = false;
                if update_display_info {
                    self.display_info =
                        monitors::get_display_information(&self.config.display_index_preferences);
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let main_screen: Element<Message> = match self.main_screen {
            Screen::Home => {
                let title = container(
                    row![
                        text("🍉").font(*EMOJI_FONT).size(70),
                        text("Komorice").size(75),
                        text("🍚").font(*EMOJI_FONT).size(70)
                    ]
                    .align_y(Center),
                )
                .width(Fill)
                .align_x(Center);
                let subtitle = text("A komorebi GUI ricing configurator!")
                    .size(20)
                    .width(Fill)
                    .align_x(Center);
                let col = column![title, subtitle].spacing(20);
                stack([
                    center(col).padding(20).into(),
                    column![
                        vertical_space(),
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
                            .size(12)
                        )
                        .width(Fill)
                        .align_x(Center)
                        .align_y(iced::Bottom)
                    ]
                    .into(),
                ])
                .into()
            }
            Screen::General => self.general.view(&self.config).map(Message::General),
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
                    iced::widget::horizontal_space().into()
                }
            }
            Screen::Border => self.border.view(&self.config).map(Message::Border),
            Screen::Stackbar => self
                .stackbar
                .view(self.config.stackbar.as_ref())
                .map(Message::Stackbar),
            Screen::Transparency => self
                .transparency
                .view(&self.config)
                .map(Message::Transparency),
            Screen::Animations => self
                .animation
                .view(self.config.animation.as_ref())
                .map(Message::Animation),
            Screen::Theme => self.theme_screen.view(&self.config).map(Message::Theme),
            Screen::Rules => self
                .rules
                .view(&self.config, self.settings.show_advanced)
                .map(Message::Rules),
            Screen::LiveDebug => self.live_debug.view().map(Message::LiveDebug),
            Screen::Settings => self.settings.view().map(Message::Settings),
        };

        let sidebar: Element<Message> = self.sidebar.view().map(Message::Sidebar);
        let save_buttons = row![
            horizontal_space(),
            button("Save").on_press_maybe(self.is_dirty.then_some(Message::Save)),
            button("Discard Changes")
                .on_press_maybe(self.is_dirty.then_some(Message::DiscardChanges)),
        ]
        .spacing(10)
        .width(Fill);
        let right_col = column![
            container(main_screen)
                .height(Fill)
                .padding(padding::all(20).bottom(0)),
            container(horizontal_rule(2.0)).padding(padding::bottom(5)),
            save_buttons,
        ];
        row![sidebar, vertical_rule(2.0), right_col]
            .padding(10)
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
        };

        Subscription::batch([
            komo_interop::connect().map(Message::LiveDebug),
            config::worker(),
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
}
