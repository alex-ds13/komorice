use crate::{
    BOLD_FONT,
    apperror::{AppError, AppErrorKind},
    monitors::DisplayInfo,
    widget::{monitors_viewer, opt_helpers},
};

use std::{collections::HashMap, sync::Arc};

use iced::{
    Center, Element, Fill, Task, padding,
    widget::{button, checkbox, column, container, horizontal_rule, row, scrollable, text},
};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigMonitor(usize),
    ChangeScreen(Screen),

    // Komorebi related Messages
    KomorebiNotification(Arc<komorebi_client::Notification>),

    // Komorebi Command Messages
    ToggleWorkspaceTile(usize, usize, bool),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    Error(AppError),
}

#[derive(Clone, Debug, Default)]
pub enum Screen {
    #[default]
    Main,
    Monitors,
    Notifications,
}

#[derive(Default)]
pub struct LiveDebug {
    pub screen: Screen,
    pub monitor_to_config: Option<usize>,
    pub notifications: Vec<Arc<komorebi_client::NotificationEvent>>,
    pub komorebi_state: Option<Arc<komorebi_client::State>>,
    pub actual_display_info: HashMap<usize, DisplayInfo>,
}

impl LiveDebug {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::ConfigMonitor(idx) => {
                if self.monitor_to_config == Some(idx) {
                    self.monitor_to_config = None;
                } else if let Some(DisplayInfo { device_id, .. }) =
                    self.actual_display_info.get(&idx)
                {
                    println!(
                        "Go to ConfigMonitor screen for monitor {idx} with id: {}",
                        device_id
                    );
                    self.monitor_to_config = Some(idx);
                }
            }
            Message::ChangeScreen(screen) => self.screen = screen,
            Message::ToggleWorkspaceTile(monitor_idx, workspace_idx, tile) => {
                let _ = komorebi_client::send_message(
                    &komorebi_client::SocketMessage::WorkspaceTiling(
                        monitor_idx,
                        workspace_idx,
                        tile,
                    ),
                );
            }
            Message::KomorebiNotification(notification) => {
                if let Some(notification) = Arc::into_inner(notification) {
                    self.notifications.push(Arc::from(notification.event));
                    self.actual_display_info = notification
                        .state
                        .monitors
                        .elements()
                        .iter()
                        .enumerate()
                        .map(|(i, monitor)| {
                            (
                                i,
                                DisplayInfo {
                                    device_id: monitor.device_id.clone(),
                                    serial_number_id: monitor.serial_number_id.clone(),
                                    size: monitor.size,
                                },
                            )
                        })
                        .collect();
                    self.komorebi_state = Some(Arc::from(notification.state));
                } else {
                    return (
                        Action::Error(AppError {
                            title: "Failed to get notification properly.".into(),
                            description: Some(
                                "There were other references to the same notification `Arc`".into(),
                            ),
                            kind: AppErrorKind::Warning,
                        }),
                        Task::none(),
                    );
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Main => self.main_view(),
            Screen::Monitors => self.monitors_view(),
            Screen::Notifications => self.notifications_view(),
        }
    }

    fn main_view(&self) -> Element<'_, Message> {
        let monitors =
            opt_helpers::opt_button("Monitors", None, Message::ChangeScreen(Screen::Monitors));
        let notifications = opt_helpers::opt_button(
            "Notifications",
            None,
            Message::ChangeScreen(Screen::Notifications),
        );
        column![
            text("Live Debug:").size(20).font(*BOLD_FONT),
            horizontal_rule(2.0),
            monitors,
            notifications,
        ]
        .spacing(10)
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn monitors_view(&self) -> Element<'_, Message> {
        let title = row![
            button(text("Live Debug").size(20).font(*BOLD_FONT))
                .padding(0)
                .on_press(Message::ChangeScreen(Screen::Main))
                .style(button::text),
            text(" > Monitors:").size(20).font(*BOLD_FONT)
        ];
        let monitors: Element<Message> = {
            let mut col = column![].spacing(10).padding(padding::bottom(10).right(20));

            let m: Element<Message> = monitors_viewer::Monitors::new(&self.actual_display_info)
                .selected(self.monitor_to_config)
                .on_selected(Message::ConfigMonitor)
                .into();
            // let m = m.explain(iced::color!(0x00aaff));
            let m = container(m)
                .padding(10)
                .width(Fill)
                .align_x(Center)
                .style(container::rounded_box);
            col = col.push(m);

            if let Some(device) = self.monitor_to_config.and_then(|idx| {
                self.komorebi_state
                    .as_ref()
                    .and_then(|s| s.monitors.elements().get(idx))
            }) {
                let monitor_idx = self.monitor_to_config.expect("unreachable");
                col = col.push(horizontal_rule(2.0));
                col = col.push(column![
                    text!("Monitor {}:", monitor_idx).size(16),
                    text!("    -> Id: {}", device.id),
                    text!("    -> DeviceId: {}", device.device_id),
                    text!("    -> Device: {}", device.device),
                    text!("    -> Size: {:#?}", device.size),
                ]);
                col = col.push(horizontal_rule(2.0));
                col = col.push(text("Workspaces:"));
                col = device
                    .workspaces()
                    .iter()
                    .enumerate()
                    .fold(col, |col, (idx, workspace)| {
                        col.push(column![
                            row![
                                text("Name: "),
                                text!("{}", workspace.name.as_ref().map_or("", |v| v))
                            ],
                            row![
                                text("Tile: "),
                                checkbox("Tile", workspace.tile).on_toggle(move |c| {
                                    Message::ToggleWorkspaceTile(monitor_idx, idx, c)
                                })
                            ],
                        ])
                    });
            }
            scrollable(col)
                .id(scrollable::Id::new("monitors_scrollable"))
                .into()
        };

        column![title, horizontal_rule(2.0), monitors]
            .spacing(10)
            .into()
    }

    fn notifications_view(&self) -> Element<'_, Message> {
        let title = row![
            button(text("Live Debug").size(20).font(*BOLD_FONT))
                .padding(0)
                .on_press(Message::ChangeScreen(Screen::Main))
                .style(button::text),
            text(" > Notifications:").size(20).font(*BOLD_FONT)
        ];
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
        column![title, horizontal_rule(2.0), notifications]
            .spacing(10)
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn goto_start_screen(&mut self) {
        self.screen = Screen::Main;
    }
}
