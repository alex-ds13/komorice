mod apperror;
mod config;
mod komorebi_connect;
mod screen;
mod utils;
mod views;
mod widget;

use crate::apperror::AppError;
use crate::config::{ConfigHelpers, ConfigHelpersAction, ConfigStrs, GlobalConfigChangeType};
use crate::screen::{monitor, sidebar, Screen};
use crate::widget::monitors_viewer;

use std::{collections::HashMap, sync::Arc};

use iced::widget::{center, horizontal_space, pick_list};
use iced::{
    padding,
    widget::{
        checkbox, column, container, horizontal_rule, row, scrollable, text, vertical_rule, Column,
        Space,
    },
    Alignment::Center,
    Element, Font,
    Length::{Fill, Shrink},
    Subscription, Task, Theme,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_FONT: Font = Font::with_name("Segoe UI Emoji");
    static ref BOLD_FONT: Font = {
        let mut f = Font::with_name("Segoe UI");
        f.weight = iced::font::Weight::Bold;
        f
    };
    static ref NONE_STR: Arc<str> = Arc::from("");
}

fn main() -> iced::Result {
    iced::application("Komofig", Komofig::update, Komofig::view)
        .subscription(Komofig::subscription)
        .theme(Komofig::theme)
        .default_font(*DEFAULT_FONT)
        .font(iced_aw::iced_fonts::REQUIRED_FONT_BYTES)
        .run_with(Komofig::initialize)
}

#[derive(Debug, Clone)]
enum Message {
    // General App Messages
    AppError(AppError),
    ThemeChanged(Theme),

    // View/Screen related Messages
    ConfigMonitor(usize),
    Sidebar(sidebar::Message),

    // Global Editing config related Messages
    GlobalConfigChanged(GlobalConfigChangeType),
    MonitorConfigChanged(usize, monitor::Message),
    ConfigHelpers(ConfigHelpersAction),

    // Komorebi related Messages
    KomorebiNotification(Arc<komorebi_client::Notification>),
    LoadedConfig(Arc<komorebi_client::StaticConfig>),
    ConfigFileWatcherTx(async_std::channel::Sender<config::Input>),

    // Komorebi Command Messages
    ToggleWorkspaceTile(usize, usize, bool),
}

#[derive(Default)]
struct Komofig {
    sidebar: sidebar::Sidebar,
    main_screen: Screen,
    notifications: Vec<Arc<komorebi_client::NotificationEvent>>,
    komorebi_state: Option<Arc<komorebi_client::State>>,
    monitor_to_config: Option<usize>,
    monitors: HashMap<usize, monitor::Monitor>,
    config: Option<komorebi_client::StaticConfig>,
    config_helpers: ConfigHelpers,
    config_strs: Option<ConfigStrs>,
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
            Message::ConfigMonitor(idx) => {
                if self.monitor_to_config == Some(idx) {
                    self.monitor_to_config = None;
                } else if let Some(state) = &self.komorebi_state {
                    let monitors = state.monitors.elements();
                    if let Some(monitor) = monitors.get(idx) {
                        println!(
                            "Go to ConfigMonitor screen for monitor {idx} with id: {}",
                            monitor.device_id()
                        );
                        self.monitor_to_config = Some(idx);
                    }
                }
            }
            Message::Sidebar(message) => {
                let (action, task) = self.sidebar.update(message);
                let action_task = match action {
                    sidebar::Action::None => Task::none(),
                    sidebar::Action::UpdateMainScreen(screen) => {
                        self.main_screen = screen;
                        Task::none()
                    }
                };
                return Task::batch([task.map(Message::Sidebar), action_task]);
            }
            Message::GlobalConfigChanged(change_type) => match change_type {
                GlobalConfigChangeType::AppSpecificConfigurationPath(path) => {
                    if let Some(config) = &mut self.config {
                        config.app_specific_configuration_path = path;
                    }
                }
                GlobalConfigChangeType::CrossBoundaryBehaviour(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        let behaviour = match value {
                            ref s
                                if **s
                                    == *komorebi::CrossBoundaryBehaviour::Monitor.to_string() =>
                            {
                                Some(komorebi::CrossBoundaryBehaviour::Monitor)
                            }
                            ref s
                                if **s
                                    == *komorebi::CrossBoundaryBehaviour::Workspace.to_string() =>
                            {
                                Some(komorebi::CrossBoundaryBehaviour::Workspace)
                            }
                            _ => None,
                        };
                        config.cross_boundary_behaviour = behaviour;
                        config_strs.cross_boundary_behaviour = value;
                    }
                }
                GlobalConfigChangeType::CrossMonitorMoveBehaviour(value) => {
                    if let Some(config) = &mut self.config {
                        config.cross_monitor_move_behaviour = Some(value);
                    }
                }
                GlobalConfigChangeType::DefaultContainerPadding(value) => {
                    if let Some(config) = &mut self.config {
                        config.default_container_padding = Some(value);
                    }
                }
                GlobalConfigChangeType::DefaultWorkspacePadding(value) => {
                    if let Some(config) = &mut self.config {
                        config.default_workspace_padding = Some(value);
                    }
                }
                GlobalConfigChangeType::DisplayIndexPreferences(value) => {
                    if let Some(config) = &mut self.config {
                        config.display_index_preferences = Some(value);
                    }
                }
                GlobalConfigChangeType::FloatOverride(value) => {
                    if let Some(config) = &mut self.config {
                        config.float_override = Some(value);
                    }
                }
                GlobalConfigChangeType::FocusFollowsMouse(value) => {
                    if let Some(config) = &mut self.config {
                        config.focus_follows_mouse = value;
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffset(value) => {
                    if let Some(config) = &mut self.config {
                        config.global_work_area_offset = Some(value);
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetTop(value) => {
                    if let Some(config) = &mut self.config {
                        if let Some(offset) = &mut config.global_work_area_offset {
                            offset.top = value;
                        } else {
                            config.global_work_area_offset = Some(komorebi::Rect {
                                left: 0,
                                top: value,
                                right: 0,
                                bottom: 0,
                            });
                        }
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetBottom(value) => {
                    if let Some(config) = &mut self.config {
                        if let Some(offset) = &mut config.global_work_area_offset {
                            offset.bottom = value;
                        } else {
                            config.global_work_area_offset = Some(komorebi::Rect {
                                left: 0,
                                top: 0,
                                right: 0,
                                bottom: value,
                            });
                        }
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetRight(value) => {
                    if let Some(config) = &mut self.config {
                        if let Some(offset) = &mut config.global_work_area_offset {
                            offset.right = value;
                        } else {
                            config.global_work_area_offset = Some(komorebi::Rect {
                                left: 0,
                                top: 0,
                                right: value,
                                bottom: 0,
                            });
                        }
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetLeft(value) => {
                    if let Some(config) = &mut self.config {
                        if let Some(offset) = &mut config.global_work_area_offset {
                            offset.left = value;
                        } else {
                            config.global_work_area_offset = Some(komorebi::Rect {
                                left: value,
                                top: 0,
                                right: 0,
                                bottom: 0,
                            });
                        }
                    }
                }
                GlobalConfigChangeType::MouseFollowsFocus(value) => {
                    if let Some(config) = &mut self.config {
                        config.mouse_follows_focus = Some(value);
                    }
                }
                GlobalConfigChangeType::ResizeDelta(value) => {
                    if let Some(config) = &mut self.config {
                        config.resize_delta = Some(value);
                    }
                }
                GlobalConfigChangeType::Transparency(value) => {
                    if let Some(config) = &mut self.config {
                        config.transparency = Some(value);
                    }
                }
                GlobalConfigChangeType::TransparencyAlpha(value) => {
                    if let Some(config) = &mut self.config {
                        config.transparency_alpha = Some(value.try_into().unwrap_or(0));
                    }
                }
                GlobalConfigChangeType::UnmanagedWindowBehaviour(value) => {
                    if let Some(config) = &mut self.config {
                        config.unmanaged_window_operation_behaviour = Some(value);
                    }
                }
                GlobalConfigChangeType::WindowContainerBehaviour(value) => {
                    if let Some(config) = &mut self.config {
                        config.window_container_behaviour = Some(value);
                    }
                }
                GlobalConfigChangeType::WindowHidingBehaviour(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        let behaviour = match value {
                            ref s if **s == *komorebi::HidingBehaviour::Cloak.to_string() => {
                                Some(komorebi::HidingBehaviour::Cloak)
                            }
                            ref s if **s == *komorebi::HidingBehaviour::Hide.to_string() => {
                                Some(komorebi::HidingBehaviour::Hide)
                            }
                            ref s if **s == *komorebi::HidingBehaviour::Minimize.to_string() => {
                                Some(komorebi::HidingBehaviour::Minimize)
                            }
                            _ => None,
                        };
                        config.window_hiding_behaviour = behaviour;
                        config_strs.window_hiding_behaviour = value;
                    }
                }
            },
            Message::MonitorConfigChanged(idx, message) => {
                if let Some(m) = self.monitors.get_mut(&idx) {
                    return m
                        .update(message)
                        .map(move |message| Message::MonitorConfigChanged(idx, message));
                }
            }
            Message::ConfigHelpers(action) => match action {
                ConfigHelpersAction::ToggleGlobalWorkAreaOffsetExpand => {
                    self.config_helpers.global_work_area_offset_expanded =
                        !self.config_helpers.global_work_area_offset_expanded;
                }
                ConfigHelpersAction::ToggleMonitorWindowBasedWorkAreaOffsetExpand(monitor_idx) => {
                    self.config_helpers
                        .monitors_window_based_work_area_offset_expanded
                        .entry(monitor_idx)
                        .and_modify(|v| *v = !*v)
                        .or_insert(true);
                }
                ConfigHelpersAction::ToggleMonitorWorkAreaOffsetExpand(monitor_idx) => {
                    self.config_helpers
                        .monitors_work_area_offset_expanded
                        .entry(monitor_idx)
                        .and_modify(|v| *v = !*v)
                        .or_insert(true);
                }
            },
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
                        self.populate_config_helpers(&config);
                        self.populate_monitors(&config);
                        self.config = Some(config);
                    }
                    //TODO: show message on app to load external changes
                }
            }
            Message::ConfigFileWatcherTx(sender) => {
                self.config_watcher_tx = Some(sender);
            }
            Message::ToggleWorkspaceTile(monitor_idx, workspace_idx, tile) => {
                let _ = komorebi_client::send_message(
                    &komorebi_client::SocketMessage::WorkspaceTiling(
                        monitor_idx,
                        workspace_idx,
                        tile,
                    ),
                );
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let main_screen: Element<Message> = match self.main_screen {
            Screen::Home => {
                let title = text("🍉Komorice🍚").size(75).width(Fill).align_x(Center);
                let subtitle = text("A komorebi GUI ricing configurator!")
                    .size(20)
                    .width(Fill)
                    .align_x(Center);
                let col = column![title, subtitle].spacing(20);
                center(col).padding(20).into()
            }
            Screen::General => scrollable(column![
                text("Config:").size(20),
                text!(
                    "Config was {} loaded!",
                    if self.config.is_some() {
                        "successfully"
                    } else {
                        "not"
                    }
                ),
                horizontal_rule(8.0),
                views::config::view(self),
            ])
            .into(),
            Screen::Monitors => {
                let monitors: Element<Message> = if let Some(state) = &self.komorebi_state {
                    let mut col: Column<Message> =
                        column![text("Monitors:").size(20)].padding(padding::all(5).right(20));

                    let m: Element<Message> =
                        monitors_viewer::Monitors::new(state.monitors.elements().iter().collect())
                            .selected(self.monitor_to_config)
                            .on_selected(Message::ConfigMonitor)
                            .into();
                    // let m = m.explain(color!(0x00aaff));
                    let m = container(m)
                        .padding(10)
                        .width(Fill)
                        .align_x(Center)
                        .style(container::rounded_box);
                    col = col.push(m);
                    if let Some(monitor) = self
                        .monitor_to_config
                        .and_then(|idx| state.monitors.elements().get(idx))
                    {
                        let monitor_idx = self.monitor_to_config.expect("unreachable");
                        let m = &self.monitors[&monitor_idx];
                        col = col.push(m.view().map(move |message| {
                            Message::MonitorConfigChanged(monitor_idx, message)
                        }));
                        col = col.push(column![
                            text!("Monitor {}:", monitor_idx).size(16),
                            text!("    -> Id: {}", monitor.id()),
                            text!("    -> DeviceId: {}", monitor.device_id()),
                            text!("    -> Device: {}", monitor.device()),
                            text!("    -> Size: {:#?}", monitor.size()),
                        ]);
                        col = col.push(horizontal_rule(2.0));
                        col = col.push(text("Workspaces:"));
                        col = monitor.workspaces().iter().enumerate().fold(
                            col,
                            |col, (idx, workspace)| {
                                col.push(column![
                                    row![
                                        text("Name: "),
                                        text!("{}", workspace.name().as_ref().map_or("", |v| v))
                                    ],
                                    row![
                                        text("Tile: "),
                                        checkbox("Tile", *workspace.tile()).on_toggle(move |c| {
                                            Message::ToggleWorkspaceTile(monitor_idx, idx, c)
                                        })
                                    ],
                                ])
                            },
                        );
                    }
                    // let monitors = state.monitors.elements()
                    //     .iter()
                    //     .enumerate()
                    //     .fold(col, |col, (idx, monitor)| {
                    //         col.push(column![
                    //             text!("Monitor {idx}:").size(16),
                    //             text!("    -> Id: {}", monitor.id()),
                    //             text!("    -> DeviceId: {}", monitor.device_id()),
                    //             text!("    -> Device: {}", monitor.device()),
                    //             text!("    -> Size: {:#?}", monitor.size()),
                    //         ])
                    //     });
                    // monitors.into()
                    scrollable(col).into()
                } else {
                    Space::new(Shrink, Shrink).into()
                };
                monitors
            }
            Screen::Monitor(_) => todo!(),
            Screen::Workspaces(_) => todo!(),
            Screen::Workspace(_, _) => todo!(),
            Screen::Border => center(text("Border").size(50)).into(),
            Screen::Stackbar => center(text("Stackbar").size(50)).into(),
            Screen::Transparency => center(text("Transparency").size(50)).into(),
            Screen::Rules => center(text("Rules").size(50)).into(),
            Screen::Debug => {
                let notifications = scrollable(
                    self.notifications
                        .iter()
                        .fold(column![], |col, notification| {
                            col.push(text(format!("-> {:?}", notification)))
                        })
                        .spacing(10)
                        .width(Fill)
                        .padding(padding::all(5).right(20)),
                );
                column![
                    text("Notifications:").font(*BOLD_FONT).size(30),
                    horizontal_rule(2.0),
                    notifications,
                ]
                .spacing(10)
                .padding(20)
                .into()
            }
            Screen::Settings => {
                let title = text("Settings:").size(20).font(*BOLD_FONT);
                let theme = row![
                    "Theme:",
                    horizontal_space(),
                    pick_list(Theme::ALL, self.theme.as_ref(), Message::ThemeChanged),
                ]
                .spacing(10);
                column![
                    title,
                    horizontal_rule(2.0),
                    theme
                ]
                .spacing(10)
                .padding(20)
                .width(Fill)
                .height(Fill)
                .into()
            }
        };

        let sidebar: Element<Message> = self.sidebar.view().map(Message::Sidebar);
        row![sidebar, vertical_rule(2.0), main_screen]
            .spacing(10)
            .padding(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([komorebi_connect::connect(), config::worker()])
    }

    pub fn theme(&self) -> Theme {
        match &self.theme {
            Some(theme) => theme.clone(),
            None => Theme::CatppuccinMocha,
        }
    }

    fn populate_config_strs(&mut self, config: &komorebi::StaticConfig) {
        let config_strs = ConfigStrs {
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

        self.config_strs = Some(config_strs);
    }

    fn populate_config_helpers(&mut self, config: &komorebi::StaticConfig) {
        self.config_helpers = ConfigHelpers {
            global_work_area_offset_expanded: false,
            monitors_window_based_work_area_offset_expanded: config.monitors.as_ref().map_or(
                HashMap::new(),
                |monitors| {
                    monitors
                        .iter()
                        .enumerate()
                        .map(|(i, _)| (i, false))
                        .collect()
                },
            ),
            monitors_work_area_offset_expanded: config.monitors.as_ref().map_or(
                HashMap::new(),
                |monitors| {
                    monitors
                        .iter()
                        .enumerate()
                        .map(|(i, _)| (i, false))
                        .collect()
                },
            ),
        };
    }

    fn populate_monitors(&mut self, config: &komorebi::StaticConfig) {
        self.monitors = config.monitors.as_ref().map_or(HashMap::new(), |monitors| {
            monitors
                .iter()
                .enumerate()
                .map(|(index, m)| {
                    (
                        index,
                        monitor::Monitor {
                            index,
                            config: m.clone(),
                            window_based_work_area_offset_expanded: false,
                            work_area_offset_expanded: false,
                            show_workspaces: false,
                            expanded_workspaces: m
                                .workspaces
                                .iter()
                                .enumerate()
                                .map(|(i, _)| (i, false))
                                .collect(),
                        },
                    )
                })
                .collect()
        });
    }
}
