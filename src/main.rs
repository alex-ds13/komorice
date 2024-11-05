mod apperror;
mod config;
mod komorebi_connect;
mod monitors_viewer;

use std::sync::Arc;

use apperror::AppError;
use iced::{
    widget::{
        checkbox, column, container, horizontal_rule, row, scrollable, text, vertical_rule, Column,
        Space,
    },
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    Subscription, Task, Theme,
};

fn main() -> iced::Result {
    iced::application("Komofig", Komofig::update, Komofig::view)
        .subscription(Komofig::subscription)
        .theme(Komofig::theme)
        .run_with(Komofig::initialize)
}

#[derive(Debug)]
enum Message {
    AppError(AppError),
    ConfigMonitor(usize),
    KomorebiNotification(Arc<komorebi_client::Notification>),
    LoadedConfig(Arc<komorebi_client::StaticConfig>),
    ConfigFileWatcherTx(async_std::channel::Sender<config::Input>),
    ToggleWorkspaceTile(usize, usize, bool),
}

#[derive(Default)]
struct Komofig {
    notifications: Vec<Arc<komorebi_client::NotificationEvent>>,
    komorebi_state: Option<Arc<komorebi_client::State>>,
    monitor_to_config: Option<usize>,
    loaded_config: Option<Arc<komorebi_client::StaticConfig>>,
    config_watcher_tx: Option<async_std::channel::Sender<config::Input>>,
    errors: Vec<AppError>,
}

impl Komofig {
    pub fn initialize() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(config::load(), |res| {
                match res {
                    Ok(config) => Message::LoadedConfig(Arc::new(config)),
                    Err(apperror) => Message::AppError(apperror),
                }
            })
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppError(apperror) => {
                println!("Received AppError: {apperror:#?}");
                self.errors.push(apperror);
            }
            Message::ConfigMonitor(idx) => {
                if let Some(state) = &self.komorebi_state {
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
                println!("Config Loaded: {config:#?}");
                self.loaded_config = Some(config);
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
            text!("Config was {} loaded!", if self.loaded_config.is_some() { "successfully" } else { "not" }),
            horizontal_rule(2.0),
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
        Subscription::batch([
            komorebi_connect::connect(),
            config::worker(),
        ])
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }
}
