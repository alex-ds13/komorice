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
    animation, border, general, home, live_debug, monitors, rules, sidebar, stackbar, theme,
    transparency, ConfigType, Screen,
};
use crate::whkd::Whkdrc;
use crate::widget::{button_with_icon, icons};

use std::collections::HashMap;
use std::sync::Arc;

use iced::{
    padding,
    widget::{
        button, checkbox, column, container, horizontal_rule, horizontal_space, rich_text, row,
        scrollable, span, text, vertical_rule,
    },
    Center, Element, Fill, Font, Right, Subscription, Task, Theme,
};
use lazy_static::lazy_static;
use whkd::DEFAULT_WHKDRC;

lazy_static! {
    static ref KOMOREBI_VERSION: &'static str = "master";
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
                    println!("Error creating icon: {}", error);
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
    Whkd(screen::whkd::Message),

    // Config related Messages
    LoadedConfig(Arc<komorebi_client::StaticConfig>),
    FailedToLoadConfig(AppError),
    ConfigFileWatcherTx(smol::channel::Sender<config::Input>),
    DiscardChanges,
    TrySave,
    ToggleConfigModal,
    Save,
    Saved,
}

struct Komorice {
    main_screen: Screen,
    config_type: ConfigType,
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
    whkd: screen::whkd::Whkd,
    config: komorebi_client::StaticConfig,
    loaded_config: Arc<komorebi_client::StaticConfig>,
    is_dirty: bool,
    config_watcher_tx: Option<smol::channel::Sender<config::Input>>,
    whkdrc: Whkdrc,
    errors: Vec<AppError>,
    show_save_config_modal: bool,
    show_errors_modal: bool,
}

impl Default for Komorice {
    fn default() -> Self {
        Self {
            main_screen: Default::default(),
            config_type: Default::default(),
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
            whkdrc: DEFAULT_WHKDRC.clone(),
            errors: Default::default(),
            show_save_config_modal: Default::default(),
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
                config::load_task(),
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
                let (action, task) = self.home.update(message);
                let action_task = match action {
                    home::Action::None => Task::none(),
                    home::Action::ChangeConfigType(config_type) => {
                        self.config_type = config_type;
                        self.main_screen = self.sidebar.selected_screen(&self.config_type);
                        if matches!(self.config_type, screen::ConfigType::Whkd)
                            && !self.whkd.loaded_commands
                        {
                            self.whkd.load_commands().map(Message::Whkd)
                        } else {
                            Task::none()
                        }
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
                let (action, task) = self.whkd.update(message, &mut self.whkdrc);
                let action_task = match action {
                    screen::whkd::Action::None => Task::none(),
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
                let (action, task) = self.sidebar.update(message, &self.config_type);
                let action_task = match action {
                    sidebar::Action::None => Task::none(),
                    sidebar::Action::SetHomeScreen => {
                        self.main_screen = Screen::Home;
                        Task::none()
                    }
                    sidebar::Action::UpdateMainScreen(screen) => {
                        self.main_screen = screen;
                        self.reset_screen();
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
                    self.home.has_loaded_config = true;
                    self.loaded_config = Arc::new(config);
                    //TODO: show message on app to load external changes
                }
            }
            Message::FailedToLoadConfig(apperror) => self.add_error(apperror),
            Message::ConfigFileWatcherTx(sender) => {
                self.config_watcher_tx = Some(sender);
            }
            Message::TrySave => {
                if self.settings.show_save_warning {
                    self.show_save_config_modal = true;
                } else {
                    return config::save_task(self.config.clone());
                }
            }
            Message::ToggleConfigModal => {
                self.show_save_config_modal = !self.show_save_config_modal;
            }
            Message::Save => {
                self.show_save_config_modal = false;
                return config::save_task(self.config.clone());
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
            Screen::Home => self.home.view().map(Message::Home),
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
            Screen::Whkd => self
                .whkd
                .view(&self.whkdrc, &self.settings.theme)
                .map(Message::Whkd),
            Screen::WhkdBinding => self
                .whkd
                .view(&self.whkdrc, &self.settings.theme)
                .map(Message::Whkd),
        };

        if matches!(self.main_screen, Screen::Home) {
            return main_screen;
        }

        let sidebar = self.sidebar.view(&self.config_type).map(Message::Sidebar);
        let mut save_buttons = row![].spacing(10).padding(padding::left(10)).width(Fill);
        save_buttons = save_buttons.push_maybe((!self.errors.is_empty()).then(|| {
            button("Errors")
                .on_press(Message::OpenErrorsModal)
                .style(button::danger)
        }));
        save_buttons = save_buttons.extend([
            horizontal_space().into(),
            button("Save")
                .on_press_maybe(self.is_dirty.then_some(Message::TrySave))
                .into(),
            button("Discard Changes")
                .on_press_maybe(self.is_dirty.then_some(Message::DiscardChanges))
                .into(),
        ]);
        let right_col = column![
            container(main_screen)
                .height(Fill)
                .padding(padding::all(20).bottom(0)),
            container(horizontal_rule(2.0)).padding(padding::bottom(5)),
            save_buttons,
        ];
        let main_content = row![sidebar, vertical_rule(2.0), right_col].padding(10);
        let modal_content = self.show_save_config_modal.then(|| self.save_warning());
        let main_modal = widget::modal(main_content, modal_content, Message::ToggleConfigModal);
        let errors_modal_content = self.show_errors_modal.then(|| self.errors_modal());
        widget::modal(main_modal, errors_modal_content, Message::CloseErrorsModal)
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
            | Screen::Settings
            | Screen::WhkdBinding => Subscription::none(),
            Screen::Monitors => self.monitors.subscription().map(Message::Monitors),
            Screen::Transparency => self.transparency.subscription().map(Message::Transparency),
            Screen::Rules => self.rules.subscription().map(Message::Rules),
            Screen::Whkd => self.whkd.subscription().map(Message::Whkd),
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

    /// Checks if the current screen allows resetting and if it does, it applies the resetting
    /// function for that screen.
    fn reset_screen(&mut self) {
        if SCREENS_TO_RESET.contains(&self.main_screen) {
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
                Screen::LiveDebug => self.live_debug.reset_screen(),
                Screen::Settings => {
                    unreachable!("should never try to reset settings screen!")
                }
                Screen::Whkd => self.whkd = Default::default(),
                Screen::WhkdBinding => {}
            }
        }
    }

    fn save_warning(&self) -> container::Container<Message> {
        let save = button("Save").on_press_maybe(self.is_dirty.then_some(Message::Save));
        let cancel = button("Cancel")
            .on_press(Message::ToggleConfigModal)
            .style(button::secondary);
        let stop_showing = container(
            Element::from(
                checkbox(
                    "Don't show this message again",
                    !self.settings.show_save_warning,
                )
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
        println!("Received AppError: {apperror:#?}");
        if matches!(apperror.kind, AppErrorKind::Error) {
            self.show_errors_modal = true;
        }
        self.errors.push(apperror);
    }

    fn errors_modal(&self) -> container::Container<Message> {
        let mut errors_column = column![row![
            text("Errors").size(30.0),
            horizontal_space(),
            button(text("❌").font(*EMOJI_FONT))
                .on_press(Message::CloseErrorsModal)
                .style(button::text),
        ]
        .spacing(10)
        .padding([10, 0])
        .align_y(Center),]
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
            column![button_with_icon(icons::delete(), "Clear").on_press(Message::ClearErrors),]
                .width(Fill)
                .align_x(Right),
        );

        container(
            container(errors_column)
                .padding(20)
                .max_width(850.0)
                .center(iced::Shrink)
                .style(widget::modal::red),
        )
        .padding(20)
    }
}
