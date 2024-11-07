mod apperror;
mod config;
mod komorebi_connect;
mod monitors_viewer;
mod views;
mod widget;

use std::{collections::HashMap, sync::Arc};

use apperror::AppError;
use config::{
    ConfigHelpers, ConfigHelpersAction, ConfigStrs, GlobalConfigChangeType, GlobalConfigStrs,
    MonitorConfigChangeType, MonitorConfigStrs, WorkspaceConfigStrs,
};
use iced::{
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
        .run_with(Komofig::initialize)
}

#[derive(Debug, Clone)]
enum Message {
    // General App Messages
    AppError(AppError),

    // View related Messages
    ConfigMonitor(usize),

    // Global Editing config related Messages
    GlobalConfigChanged(GlobalConfigChangeType),
    MonitorConfigChanged(usize, MonitorConfigChangeType),
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
    notifications: Vec<Arc<komorebi_client::NotificationEvent>>,
    komorebi_state: Option<Arc<komorebi_client::State>>,
    monitor_to_config: Option<usize>,
    config: Option<komorebi_client::StaticConfig>,
    config_helpers: ConfigHelpers,
    config_strs: Option<ConfigStrs>,
    // loaded_config: Option<Arc<komorebi_client::StaticConfig>>,
    config_watcher_tx: Option<async_std::channel::Sender<config::Input>>,
    errors: Vec<AppError>,
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
                        config_strs.global_config_strs.cross_boundary_behaviour = value;
                    }
                }
                GlobalConfigChangeType::CrossMonitorMoveBehaviour(value) => {
                    if let Some(config) = &mut self.config {
                        config.cross_monitor_move_behaviour = Some(value);
                    }
                }
                GlobalConfigChangeType::DefaultContainerPadding(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        config.default_container_padding = Some(value.parse().unwrap_or_default());
                        config_strs.global_config_strs.default_container_padding = value.into();
                    }
                }
                GlobalConfigChangeType::DefaultWorkspacePadding(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        config.default_workspace_padding = Some(value.parse().unwrap_or_default());
                        config_strs.global_config_strs.default_workspace_padding = value.into();
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
                GlobalConfigChangeType::FocusFollowsMouse1(value) => {
                    if let (Some(config), Some(_config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        config.focus_follows_mouse = value;
                    }
                }
                GlobalConfigChangeType::FocusFollowsMouse(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        let implementation = match value {
                            ref s
                                if **s
                                    == *komorebi::FocusFollowsMouseImplementation::Windows
                                        .to_string() =>
                            {
                                Some(komorebi::FocusFollowsMouseImplementation::Windows)
                            }
                            ref s
                                if **s
                                    == *komorebi::FocusFollowsMouseImplementation::Komorebi
                                        .to_string() =>
                            {
                                Some(komorebi::FocusFollowsMouseImplementation::Komorebi)
                            }
                            _ => None,
                        };
                        config.focus_follows_mouse = implementation;
                        config_strs.global_config_strs.focus_follows_mouse = value;
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffset(value) => {
                    if let Some(config) = &mut self.config {
                        config.global_work_area_offset = Some(value);
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetTop(value) => {
                    if let Ok(top) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config.global_work_area_offset {
                                offset.top = top;
                            } else {
                                config.global_work_area_offset = Some(komorebi::Rect {
                                    left: 0,
                                    top,
                                    right: 0,
                                    bottom: 0,
                                });
                            }
                            config_strs.global_config_strs.global_work_area_offset_top =
                                value.into();
                        }
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetBottom(value) => {
                    if let Ok(bottom) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config.global_work_area_offset {
                                offset.bottom = bottom;
                            } else {
                                config.global_work_area_offset = Some(komorebi::Rect {
                                    left: 0,
                                    top: 0,
                                    right: 0,
                                    bottom,
                                });
                            }
                            config_strs
                                .global_config_strs
                                .global_work_area_offset_bottom = value.into();
                        }
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetRight(value) => {
                    if let Ok(right) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config.global_work_area_offset {
                                offset.right = right;
                            } else {
                                config.global_work_area_offset = Some(komorebi::Rect {
                                    left: 0,
                                    top: 0,
                                    right,
                                    bottom: 0,
                                });
                            }
                            config_strs.global_config_strs.global_work_area_offset_right =
                                value.into();
                        }
                    }
                }
                GlobalConfigChangeType::GlobalWorkAreaOffsetLeft(value) => {
                    if let Ok(left) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config.global_work_area_offset {
                                offset.left = left;
                            } else {
                                config.global_work_area_offset = Some(komorebi::Rect {
                                    left,
                                    top: 0,
                                    right: 0,
                                    bottom: 0,
                                });
                            }
                            config_strs.global_config_strs.global_work_area_offset_left =
                                value.into();
                        }
                    }
                }
                GlobalConfigChangeType::MouseFollowsFocus(value) => {
                    if let Some(config) = &mut self.config {
                        config.mouse_follows_focus = Some(value);
                    }
                }
                GlobalConfigChangeType::ResizeDelta(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        config.resize_delta = Some(value.parse().unwrap_or_default());
                        config_strs.global_config_strs.resize_delta = value.into();
                    }
                }
                GlobalConfigChangeType::Transparency(value) => {
                    if let Some(config) = &mut self.config {
                        config.transparency = Some(value);
                    }
                }
                GlobalConfigChangeType::TransparencyAlpha(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        config.transparency_alpha = Some(value.parse().unwrap_or_default());
                        config_strs.global_config_strs.transparency_alpha = value.into();
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
                        config_strs.global_config_strs.window_hiding_behaviour = value;
                    }
                }
            },
            Message::MonitorConfigChanged(idx, change_type) => match change_type {
                MonitorConfigChangeType::WindowBasedWorkAreaOffset(_) => todo!(),
                MonitorConfigChangeType::WindowBasedWorkAreaOffsetTop(value) => {
                    if let Ok(top) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config
                                .monitors
                                .as_mut()
                                .and_then(|monitors| monitors[idx].window_based_work_area_offset)
                            {
                                offset.top = top;
                            } else {
                                let offset = &mut config.monitors.as_mut().and_then(|monitors| {
                                    monitors[idx].window_based_work_area_offset
                                });
                                *offset = Some(komorebi::Rect {
                                    left: 0,
                                    top,
                                    right: 0,
                                    bottom: 0,
                                });
                            }
                            if let Some(monitor_config_str) =
                                config_strs.monitors_config_strs.get_mut(&idx)
                            {
                                monitor_config_str.window_based_work_area_offset_top = value.into();
                            } else {
                                config_strs.monitors_config_strs.insert(
                                    idx,
                                    MonitorConfigStrs {
                                        window_based_work_area_offset_top: value.into(),
                                        window_based_work_area_offset_bottom: "".into(),
                                        window_based_work_area_offset_right: "".into(),
                                        window_based_work_area_offset_left: "".into(),
                                        window_based_work_area_offset_limit: "".into(),
                                        work_area_offset_top: "".into(),
                                        work_area_offset_bottom: "".into(),
                                        work_area_offset_right: "".into(),
                                        work_area_offset_left: "".into(),
                                    },
                                );
                            }
                        }
                    }
                }
                MonitorConfigChangeType::WindowBasedWorkAreaOffsetBottom(value) => {
                    if let Ok(bottom) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config
                                .monitors
                                .as_mut()
                                .and_then(|monitors| monitors[idx].window_based_work_area_offset)
                            {
                                offset.bottom = bottom;
                            } else {
                                let offset = &mut config.monitors.as_mut().and_then(|monitors| {
                                    monitors[idx].window_based_work_area_offset
                                });
                                *offset = Some(komorebi::Rect {
                                    left: 0,
                                    top: 0,
                                    right: 0,
                                    bottom,
                                });
                            }
                            if let Some(monitor_config_str) =
                                config_strs.monitors_config_strs.get_mut(&idx)
                            {
                                monitor_config_str.window_based_work_area_offset_bottom =
                                    value.into();
                            } else {
                                config_strs.monitors_config_strs.insert(
                                    idx,
                                    MonitorConfigStrs {
                                        window_based_work_area_offset_top: "".into(),
                                        window_based_work_area_offset_bottom: value.into(),
                                        window_based_work_area_offset_right: "".into(),
                                        window_based_work_area_offset_left: "".into(),
                                        window_based_work_area_offset_limit: "".into(),
                                        work_area_offset_top: "".into(),
                                        work_area_offset_bottom: "".into(),
                                        work_area_offset_right: "".into(),
                                        work_area_offset_left: "".into(),
                                    },
                                );
                            }
                        }
                    }
                }
                MonitorConfigChangeType::WindowBasedWorkAreaOffsetRight(value) => {
                    if let Ok(right) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config
                                .monitors
                                .as_mut()
                                .and_then(|monitors| monitors[idx].window_based_work_area_offset)
                            {
                                offset.right = right;
                            } else {
                                let offset = &mut config.monitors.as_mut().and_then(|monitors| {
                                    monitors[idx].window_based_work_area_offset
                                });
                                *offset = Some(komorebi::Rect {
                                    left: 0,
                                    top: 0,
                                    right,
                                    bottom: 0,
                                });
                            }
                            if let Some(monitor_config_str) =
                                config_strs.monitors_config_strs.get_mut(&idx)
                            {
                                monitor_config_str.window_based_work_area_offset_right =
                                    value.into();
                            } else {
                                config_strs.monitors_config_strs.insert(
                                    idx,
                                    MonitorConfigStrs {
                                        window_based_work_area_offset_top: "".into(),
                                        window_based_work_area_offset_bottom: "".into(),
                                        window_based_work_area_offset_right: value.into(),
                                        window_based_work_area_offset_left: "".into(),
                                        window_based_work_area_offset_limit: "".into(),
                                        work_area_offset_top: "".into(),
                                        work_area_offset_bottom: "".into(),
                                        work_area_offset_right: "".into(),
                                        work_area_offset_left: "".into(),
                                    },
                                );
                            }
                        }
                    }
                }
                MonitorConfigChangeType::WindowBasedWorkAreaOffsetLeft(value) => {
                    if let Ok(left) = value.parse() {
                        if let (Some(config), Some(config_strs)) =
                            (&mut self.config, &mut self.config_strs)
                        {
                            if let Some(offset) = &mut config
                                .monitors
                                .as_mut()
                                .and_then(|monitors| monitors[idx].window_based_work_area_offset)
                            {
                                offset.left = left;
                            } else {
                                let offset = &mut config.monitors.as_mut().and_then(|monitors| {
                                    monitors[idx].window_based_work_area_offset
                                });
                                *offset = Some(komorebi::Rect {
                                    left,
                                    top: 0,
                                    right: 0,
                                    bottom: 0,
                                });
                            }
                            if let Some(monitor_config_str) =
                                config_strs.monitors_config_strs.get_mut(&idx)
                            {
                                monitor_config_str.window_based_work_area_offset_left =
                                    value.into();
                            } else {
                                config_strs.monitors_config_strs.insert(
                                    idx,
                                    MonitorConfigStrs {
                                        window_based_work_area_offset_top: "".into(),
                                        window_based_work_area_offset_bottom: "".into(),
                                        window_based_work_area_offset_right: "".into(),
                                        window_based_work_area_offset_left: value.into(),
                                        window_based_work_area_offset_limit: "".into(),
                                        work_area_offset_top: "".into(),
                                        work_area_offset_bottom: "".into(),
                                        work_area_offset_right: "".into(),
                                        work_area_offset_left: "".into(),
                                    },
                                );
                            }
                        }
                    }
                }
                MonitorConfigChangeType::WindowBasedWorkAreaOffsetLimit(value) => {
                    if let (Some(config), Some(config_strs)) =
                        (&mut self.config, &mut self.config_strs)
                    {
                        let limit = value.parse().unwrap_or(1);
                        if let Some(monitors) = &mut config.monitors {
                            if let Some(monitor) = monitors.get_mut(idx) {
                                monitor.window_based_work_area_offset_limit = Some(limit);
                            } else {
                                monitors.reserve(idx + 1 - monitors.len());
                                for _ in monitors.len()..idx {
                                    monitors.push(komorebi::MonitorConfig {
                                        workspaces: Vec::new(),
                                        work_area_offset: None,
                                        window_based_work_area_offset: None,
                                        window_based_work_area_offset_limit: None,
                                    });
                                }
                                monitors.push(komorebi::MonitorConfig {
                                    workspaces: Vec::new(),
                                    work_area_offset: None,
                                    window_based_work_area_offset: None,
                                    window_based_work_area_offset_limit: Some(limit),
                                });
                            }
                        } else {
                            let mut monitors = vec![
                                komorebi::MonitorConfig {
                                    workspaces: Vec::new(),
                                    work_area_offset: None,
                                    window_based_work_area_offset: None,
                                    window_based_work_area_offset_limit: None,
                                };
                                idx + 1
                            ];
                            monitors[idx].window_based_work_area_offset_limit = Some(limit);
                            config.monitors = Some(monitors);
                        }
                        if let Some(monitor_config_str) =
                            config_strs.monitors_config_strs.get_mut(&idx)
                        {
                            monitor_config_str.window_based_work_area_offset_limit = value.into();
                        } else {
                            config_strs.monitors_config_strs.insert(
                                idx,
                                MonitorConfigStrs {
                                    window_based_work_area_offset_top: "".into(),
                                    window_based_work_area_offset_bottom: "".into(),
                                    window_based_work_area_offset_right: "".into(),
                                    window_based_work_area_offset_left: "".into(),
                                    window_based_work_area_offset_limit: value.into(),
                                    work_area_offset_top: "".into(),
                                    work_area_offset_bottom: "".into(),
                                    work_area_offset_right: "".into(),
                                    work_area_offset_left: "".into(),
                                },
                            );
                        }
                    }
                }
                MonitorConfigChangeType::WorkAreaOffset(_) => todo!(),
                MonitorConfigChangeType::WorkAreaOffsetTop(_) => todo!(),
                MonitorConfigChangeType::WorkAreaOffsetBottom(_) => todo!(),
                MonitorConfigChangeType::WorkAreaOffsetRight(_) => todo!(),
                MonitorConfigChangeType::WorkAreaOffsetLeft(_) => todo!(),
            },
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
        let monitors: Element<Message> = if let Some(state) = &self.komorebi_state {
            let mut col: Column<Message> = column![text("Monitors:").size(20)];
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
                col = col.push(views::config::view_monitor(self, monitor_idx));
                col = col.push(column![
                    text!("Monitor {}:", monitor_idx).size(16),
                    text!("    -> Id: {}", monitor.id()),
                    text!("    -> DeviceId: {}", monitor.device_id()),
                    text!("    -> Device: {}", monitor.device()),
                    text!("    -> Size: {:#?}", monitor.size()),
                ]);
                col = col.push(horizontal_rule(2.0));
                col = col.push(text("Workspaces:"));
                col = monitor
                    .workspaces()
                    .iter()
                    .enumerate()
                    .fold(col, |col, (idx, workspace)| {
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
                    });
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
        let col = column![
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
            horizontal_rule(8.0),
            text("Notifications:").size(20),
        ];
        let notifications = self.notifications.iter().fold(col, |col, notification| {
            col.push(text(format!("{:?}", notification)))
        });
        let scrollable = scrollable(notifications).width(Fill);
        row![monitors, vertical_rule(2.0), scrollable,]
            .spacing(10)
            .padding(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([komorebi_connect::connect(), config::worker()])
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    fn populate_config_strs(&mut self, config: &komorebi::StaticConfig) {
        let global_config_strs = GlobalConfigStrs {
            cross_boundary_behaviour: config
                .cross_boundary_behaviour
                .unwrap_or(komorebi::CrossBoundaryBehaviour::Monitor)
                .to_string()
                .into(),
            default_container_padding: config.default_container_padding.map_or(
                komorebi::DEFAULT_CONTAINER_PADDING
                    .load(std::sync::atomic::Ordering::SeqCst)
                    .to_string()
                    .into(),
                |v| v.to_string().into(),
            ),
            default_workspace_padding: config.default_workspace_padding.map_or(
                komorebi::DEFAULT_WORKSPACE_PADDING
                    .load(std::sync::atomic::Ordering::SeqCst)
                    .to_string()
                    .into(),
                |v| v.to_string().into(),
            ),
            focus_follows_mouse: config
                .focus_follows_mouse
                .map_or(Arc::clone(&NONE_STR), |i| i.to_string().into()),
            resize_delta: config.resize_delta.unwrap_or(50).to_string().into(),
            transparency_alpha: config.transparency_alpha.unwrap_or(200).to_string().into(),
            global_work_area_offset_top: config
                .global_work_area_offset
                .map_or("0".into(), |r| r.top.to_string().into()),
            global_work_area_offset_bottom: config
                .global_work_area_offset
                .map_or("0".into(), |r| r.bottom.to_string().into()),
            global_work_area_offset_right: config
                .global_work_area_offset
                .map_or("0".into(), |r| r.right.to_string().into()),
            global_work_area_offset_left: config
                .global_work_area_offset
                .map_or("0".into(), |r| r.left.to_string().into()),
            window_hiding_behaviour: config
                .window_hiding_behaviour
                .unwrap_or(komorebi::HidingBehaviour::Cloak)
                .to_string()
                .into(),
        };

        let monitors_config_strs = config.monitors.as_ref().map_or(HashMap::new(), |monitors| {
            monitors
                .iter()
                .enumerate()
                .map(|(idx, m)| {
                    (
                        idx,
                        MonitorConfigStrs {
                            window_based_work_area_offset_top: m
                                .window_based_work_area_offset
                                .map_or("0".into(), |r| r.top.to_string().into()),
                            window_based_work_area_offset_bottom: m
                                .window_based_work_area_offset
                                .map_or("0".into(), |r| r.bottom.to_string().into()),
                            window_based_work_area_offset_right: m
                                .window_based_work_area_offset
                                .map_or("0".into(), |r| r.right.to_string().into()),
                            window_based_work_area_offset_left: m
                                .window_based_work_area_offset
                                .map_or("0".into(), |r| r.left.to_string().into()),
                            window_based_work_area_offset_limit: m
                                .window_based_work_area_offset_limit
                                .unwrap_or(1)
                                .to_string()
                                .into(),
                            work_area_offset_top: m
                                .work_area_offset
                                .map_or("0".into(), |r| r.top.to_string().into()),
                            work_area_offset_bottom: m
                                .work_area_offset
                                .map_or("0".into(), |r| r.bottom.to_string().into()),
                            work_area_offset_right: m
                                .work_area_offset
                                .map_or("0".into(), |r| r.right.to_string().into()),
                            work_area_offset_left: m
                                .work_area_offset
                                .map_or("0".into(), |r| r.left.to_string().into()),
                        },
                    )
                })
                .collect()
        });

        let workspaces_config_strs: HashMap<(usize, usize), WorkspaceConfigStrs> =
            config.monitors.as_ref().map_or(HashMap::new(), |monitors| {
                monitors
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, m)| {
                        let hm: HashMap<_, _> = m
                            .workspaces
                            .iter()
                            .enumerate()
                            .map(|(w_idx, w)| {
                                (
                                    (idx, w_idx),
                                    WorkspaceConfigStrs {
                                        container_padding: w.container_padding.map_or(
                                            komorebi::DEFAULT_CONTAINER_PADDING
                                                .load(std::sync::atomic::Ordering::SeqCst)
                                                .to_string()
                                                .into(),
                                            |v| v.to_string().into(),
                                        ),
                                        workspace_padding: w.workspace_padding.map_or(
                                            komorebi::DEFAULT_WORKSPACE_PADDING
                                                .load(std::sync::atomic::Ordering::SeqCst)
                                                .to_string()
                                                .into(),
                                            |v| v.to_string().into(),
                                        ),
                                    },
                                )
                            })
                            .collect();
                        hm
                    })
                    .collect()
            });

        self.config_strs = Some(ConfigStrs {
            global_config_strs,
            monitors_config_strs,
            workspaces_config_strs,
        });
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
}
