mod apperror;
mod config;
mod komo_interop;
mod screen;
mod utils;
mod widget;

use crate::apperror::AppError;
use crate::screen::{general, monitors, rules, sidebar, stackbar, transparency, Screen};

use std::sync::Arc;

use iced::widget::{button, center, horizontal_space, pick_list, stack, vertical_space, Space};
use iced::{
    padding,
    widget::{column, container, horizontal_rule, row, scrollable, text, vertical_rule},
    Center, Element, Fill, Font, Shrink, Subscription, Task, Theme,
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
    static ref SCREENS_TO_RESET: [Screen; 2] = [Screen::Rules, Screen::Transparency];
}

fn main() -> iced::Result {
    iced::application("Komofig", Komofig::update, Komofig::view)
        .subscription(Komofig::subscription)
        .theme(Komofig::theme)
        .default_font(*DEFAULT_FONT)
        .font(iced_aw::iced_fonts::REQUIRED_FONT_BYTES)
        .font(include_bytes!("../assets/icons.ttf"))
        .run_with(Komofig::initialize)
}

#[derive(Debug, Clone)]
enum Message {
    // General App Messages
    AppError(AppError),
    ThemeChanged(Theme),

    // View/Screen related Messages
    General(general::Message),
    Monitors(monitors::Message),
    Rules(rules::Message),
    Sidebar(sidebar::Message),
    Stackbar(stackbar::Message),
    Transparency(transparency::Message),

    // Komorebi related Messages
    KomorebiNotification(Arc<komorebi_client::Notification>),
    LoadedConfig(Arc<komorebi_client::StaticConfig>),
    ConfigFileWatcherTx(async_std::channel::Sender<config::Input>),
}

#[derive(Default)]
struct Komofig {
    sidebar: sidebar::Sidebar,
    main_screen: Screen,
    notifications: Vec<Arc<komorebi_client::NotificationEvent>>,
    komorebi_state: Option<Arc<komorebi_client::State>>,
    monitors: monitors::Monitors,
    general: general::General,
    stackbar: stackbar::Stackbar,
    transparency: transparency::Transparency,
    rules: rules::Rules,
    config: Option<komorebi_client::StaticConfig>,
    // loaded_config: Option<Arc<komorebi_client::StaticConfig>>,
    config_watcher_tx: Option<async_std::channel::Sender<config::Input>>,
    errors: Vec<AppError>,
    theme: Option<Theme>,
}

impl Komofig {
    pub fn initialize() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(config::load(), |res| match res {
                Ok(config) => Message::LoadedConfig(Arc::new(config)),
                Err(apperror) => Message::AppError(apperror),
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppError(apperror) => {
                println!("Received AppError: {apperror:#?}");
                self.errors.push(apperror);
            }
            Message::ThemeChanged(theme) => {
                self.theme = Some(theme);
            }
            Message::General(message) => {
                if let Some(config) = &mut self.config {
                    let (action, task) = self.general.update(message, config);
                    let action_task = match action {
                        general::Action::None => Task::none(),
                    };
                    return Task::batch([task.map(Message::General), action_task]);
                }
            }
            Message::Monitors(message) => {
                let (action, task) = self.monitors.update(message, &self.komorebi_state);
                let action_task = match action {
                    monitors::Action::None => Task::none(),
                };
                return Task::batch([task.map(Message::Monitors), action_task]);
            }
            Message::Stackbar(message) => {
                if let Some(config) = self.config.as_mut() {
                    if config.stackbar.is_none() {
                        config.stackbar = Some(stackbar::default_stackbar_config());
                    }
                    if let Some(stackbar_config) = config.stackbar.as_mut() {
                        let (action, task) = self.stackbar.update(stackbar_config, message);
                        let action_task = match action {
                            stackbar::Action::None => Task::none(),
                        };
                        return Task::batch([task.map(Message::Stackbar), action_task]);
                    }
                }
            }
            Message::Transparency(message) => {
                if let Some(config) = &mut self.config {
                    let (action, task) = self.transparency.update(message, config);
                    let action_task = match action {
                        transparency::Action::None => Task::none(),
                    };
                    return Task::batch([task.map(Message::Transparency), action_task]);
                }
            }
            Message::Rules(message) => {
                if let Some(c) = &mut self.config {
                    let (action, task) = self.rules.update(c, message);
                    let action_task = match action {
                        rules::Action::None => Task::none(),
                    };
                    return Task::batch([task.map(Message::Rules), action_task]);
                }
            }
            Message::Sidebar(message) => {
                let (action, task) = self.sidebar.update(message);
                let action_task = match action {
                    sidebar::Action::None => Task::none(),
                    sidebar::Action::UpdateMainScreen(screen) => {
                        if SCREENS_TO_RESET.contains(&screen) {
                            if matches!(screen, Screen::Rules) {
                                self.rules = rules::Rules::default();
                            } else if matches!(screen, Screen::Transparency) {
                                self.transparency = transparency::Transparency::default();
                            }
                        }
                        self.main_screen = screen;
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Sidebar), action_task]);
            }
            Message::KomorebiNotification(notification) => {
                if let Some(notification) = Arc::into_inner(notification) {
                    self.notifications.push(Arc::from(notification.event));
                    self.komorebi_state = Some(Arc::from(notification.state));
                } else {
                    self.errors.push(AppError {
                        title: "Failed to get notification properly.".into(),
                        description: Some(
                            "There were other references to the same notification `Arc`".into(),
                        ),
                        kind: apperror::AppErrorKind::Warning,
                    });
                }
            }
            Message::LoadedConfig(config) => {
                if let Some(config) = Arc::into_inner(config) {
                    println!("Config Loaded: {config:#?}");
                    // self.loaded_config = Some(Arc::new(config));
                    if self.config.is_none() {
                        self.populate_config_strs(&config);
                        self.populate_monitors(&config);
                        self.config = Some(config);
                    }
                    //TODO: show message on app to load external changes
                }
            }
            Message::ConfigFileWatcherTx(sender) => {
                self.config_watcher_tx = Some(sender);
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
                                if self.config.is_some() {
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
            Screen::Monitors => self
                .monitors
                .view(&self.komorebi_state)
                .map(Message::Monitors),
            Screen::Monitor(_) => todo!(),
            Screen::Workspaces(_) => todo!(),
            Screen::Workspace(_, _) => todo!(),
            Screen::Border => center(text("Border").size(50)).into(),
            Screen::Stackbar => {
                if let Some(config) = self.config.as_ref() {
                    self.stackbar
                        .view(config.stackbar.as_ref())
                        .map(Message::Stackbar)
                } else {
                    Space::new(Shrink, Shrink).into()
                }
            }
            Screen::Transparency => self
                .transparency
                .view(&self.config)
                .map(Message::Transparency),
            Screen::Rules => {
                if let Some(config) = &self.config {
                    self.rules.view(config).map(Message::Rules)
                } else {
                    Space::new(Shrink, Shrink).into()
                }
            }
            Screen::Debug => {
                let notifications = scrollable(
                    self.notifications
                        .iter()
                        .fold(column![], |col, notification| {
                            col.push(text(format!("-> {:?}", notification)))
                        })
                        .spacing(10)
                        .width(Fill)
                        .padding(padding::top(10).bottom(10).right(20)),
                );
                column![
                    text("Notifications:").size(20).font(*BOLD_FONT),
                    horizontal_rule(2.0),
                    notifications,
                ]
                .spacing(10)
                .width(Fill)
                .height(Fill)
                .into()
            }
            Screen::Settings => {
                let title = text("Settings:").size(20).font(*BOLD_FONT);
                let theme = row![
                    "Theme:",
                    horizontal_space(),
                    pick_list(Theme::ALL, self.theme.as_ref(), Message::ThemeChanged),
                ]
                .align_y(Center)
                .spacing(10);
                let col = column![theme].padding(padding::top(10).bottom(10).right(20));
                column![title, horizontal_rule(2.0), col]
                    .spacing(10)
                    .width(Fill)
                    .height(Fill)
                    .into()
            }
        };

        let sidebar: Element<Message> = self.sidebar.view().map(Message::Sidebar);
        let save_buttons = row![
            horizontal_space(),
            button("Save"),
            button("Discard Changes"),
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
            .spacing(10)
            .padding(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let screen_subscription = match self.main_screen {
            Screen::Monitor(_) => todo!(),
            Screen::Workspaces(_) => todo!(),
            Screen::Workspace(_, _) => todo!(),
            Screen::Home
            | Screen::General
            | Screen::Border
            | Screen::Stackbar
            | Screen::Debug
            | Screen::Settings => Subscription::none(),
            Screen::Monitors => self.monitors.subscription().map(Message::Monitors),
            Screen::Transparency => self.transparency.subscription().map(Message::Transparency),
            Screen::Rules => self.rules.subscription().map(Message::Rules),
        };

        Subscription::batch([
            komo_interop::connect(),
            config::worker(),
            screen_subscription,
        ])
    }

    pub fn theme(&self) -> Theme {
        match &self.theme {
            Some(theme) => theme.clone(),
            None => Theme::TokyoNightStorm,
        }
    }

    fn populate_config_strs(&mut self, config: &komorebi::StaticConfig) {
        let general = general::General {
            global_work_area_offset_expanded: false,
            global_work_area_offset_hovered: false,
            cross_boundary_behaviour: config
                .cross_boundary_behaviour
                .unwrap_or(komorebi::CrossBoundaryBehaviour::Monitor)
                .to_string()
                .into(),
            window_hiding_behaviour: config
                .window_hiding_behaviour
                .unwrap_or(komorebi::HidingBehaviour::Cloak)
                .to_string()
                .into(),
        };
        self.general = general;
    }

    fn populate_monitors(&mut self, config: &komorebi::StaticConfig) {
        self.monitors = monitors::Monitors::new(config);
    }
}
