#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod apperror;
mod komorebi;
mod screen;
mod settings;
mod utils;
mod whkd;
mod widget;

use crate::apperror::{AppError, AppErrorKind};
use crate::screen::{ConfigState, ConfigType, Configuration, Screen, View, home, sidebar};
use crate::widget::{button_with_icon, icons, opt_helpers::to_description_text, tooltip};

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
    OpenErrorsModal,
    CloseErrorsModal,
    ClearErrors,

    // View/Screen related Messages
    Home(home::Message),
    Sidebar(sidebar::Message),
    Settings(settings::Message),
    Komorebi(komorebi::Message),
    Whkd(whkd::Message),

    // Bottom bar messages
    DiscardChanges,
    TrySave,
    ToggleSaveModal,
    Save,
    ToggleSaveAsDialog,
    SaveAsDialogClosed,
    SaveAs(PathBuf),
    Backup,
    OpenConfigFile,
    OpenConfigFolder,
}

#[derive(Default)]
struct Komorice {
    main_screen: Screen,
    configuration: Configuration,
    sidebar: sidebar::Sidebar,
    home: home::Home,
    settings: settings::Settings,
    komorebi: komorebi::Komorebi,
    whkd: whkd::Whkd,
    errors: Vec<AppError>,
    show_save_modal: bool,
    show_save_as_dialog: bool,
    show_errors_modal: bool,
}

impl Komorice {
    pub fn initialize() -> (Self, Task<Message>) {
        let (komorebi, komorebi_task) = komorebi::Komorebi::init();
        let (whkd, whkd_task) = whkd::Whkd::init();
        let init = Komorice {
            komorebi,
            whkd,
            ..Default::default()
        };
        (
            init,
            Task::batch([
                settings::load_task().map(Message::Settings),
                komorebi_task.map(Message::Komorebi),
                whkd_task.map(Message::Whkd),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
                                ConfigState::Loaded(path) => {
                                    komorebi::load_task(path.clone()).map(Message::Komorebi)
                                }
                                ConfigState::New(_) => {
                                    self.komorebi.load_default();
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
            Message::Komorebi(message) => {
                let (action, task) = self.komorebi.update(message);
                let action_task = match action {
                    komorebi::Action::None => Task::none(),
                    komorebi::Action::Saved => {
                        self.configuration.saved_new_komorebi = true;
                        Task::none()
                    }
                    komorebi::Action::Loaded => {
                        self.configuration.has_loaded_komorebi = true;
                        if self.home.loading.is_some() {
                            self.home.loading = None;
                            self.main_screen = self
                                .sidebar
                                .selected_screen(&self.configuration.config_type);
                        }
                        Task::none()
                    }
                    komorebi::Action::FailedToLoad(app_error) => {
                        self.add_error(app_error);
                        if self.home.loading.is_some() {
                            self.home.loading = None;
                        }
                        Task::none()
                    }
                    komorebi::Action::BackupComplete => {
                        //TODO: give feedback to user
                        tooltip::close(*SAVE_TIP_ID)
                    }
                    komorebi::Action::BackupFailed(app_error) => {
                        self.add_error(app_error);
                        //TODO: give feedback to user
                        tooltip::close(*SAVE_TIP_ID)
                    }
                    komorebi::Action::AppError(app_error) => {
                        self.add_error(app_error);
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Komorebi), action_task]);
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
                        match self.configuration.config_type {
                            ConfigType::Komorebi => self.komorebi.screen = screen.clone(),
                            ConfigType::Whkd => self.whkd.screen = screen.clone(),
                        }
                        self.main_screen = screen;
                        self.screen_to_start();
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Sidebar), action_task]);
            }
            Message::TrySave => {
                if self.settings.show_save_warning {
                    self.show_save_modal = true;
                } else {
                    match self.configuration.config_type {
                        ConfigType::Komorebi => {
                            self.configuration.saved_new_komorebi = true;
                            return komorebi::save_task(
                                self.komorebi.config.clone(),
                                self.configuration.path(),
                            )
                            .map(Message::Komorebi);
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
                        return komorebi::save_task(
                            self.komorebi.config.clone(),
                            self.configuration.path(),
                        )
                        .map(Message::Komorebi);
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
                        return komorebi::save_task(
                            self.komorebi.config.clone(),
                            self.configuration.path(),
                        )
                        .map(Message::Komorebi);
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
                            return komorebi::backup_task(
                                self.komorebi.config.clone(),
                                bck_file_name,
                            )
                            .map(Message::Komorebi);
                        }
                        ConfigType::Whkd => {
                            return whkd::backup_task(self.whkd.whkdrc.clone(), bck_file_name)
                                .map(Message::Whkd);
                        }
                    }
                }
            }
            Message::DiscardChanges => match self.configuration.config_type {
                ConfigType::Komorebi => self.komorebi.discard_changes(),
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
            Screen::Animations
            | Screen::Border
            | Screen::General
            | Screen::LiveDebug
            | Screen::Monitors
            | Screen::Rules
            | Screen::Stackbar
            | Screen::Theme
            | Screen::Transparency => self.komorebi.view(&self.settings).map(Message::Komorebi),
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
        let komorebi_sub = self
            .komorebi
            .subscription(&self.configuration)
            .map(Message::Komorebi);
        let whkd_sub = self
            .whkd
            .subscription(&self.configuration)
            .map(Message::Whkd);

        Subscription::batch([
            settings::worker().map(Message::Settings),
            komorebi_sub,
            whkd_sub,
        ])
    }

    pub fn theme(&self) -> Theme {
        self.settings.theme.clone()
    }

    fn is_unsaved(&self) -> bool {
        match self.configuration.config_type {
            ConfigType::Komorebi => match self.configuration.komorebi_state {
                ConfigState::Active | ConfigState::Loaded(_) => self.komorebi.is_dirty,
                ConfigState::New(_) => {
                    self.komorebi.is_dirty || !self.configuration.saved_new_komorebi
                }
            },
            ConfigType::Whkd => match self.configuration.whkd_state {
                ConfigState::Active | ConfigState::Loaded(_) => self.whkd.is_dirty,
                ConfigState::New(_) => self.whkd.is_dirty || !self.configuration.saved_new_whkd,
            },
        }
    }

    fn is_dirty(&self) -> bool {
        match self.configuration.config_type {
            ConfigType::Komorebi => self.komorebi.is_dirty,
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
                Screen::General
                | Screen::Monitors
                | Screen::Border
                | Screen::Stackbar
                | Screen::Transparency
                | Screen::Animations
                | Screen::Theme
                | Screen::Rules
                | Screen::LiveDebug => self.komorebi.screen_to_start(),
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
