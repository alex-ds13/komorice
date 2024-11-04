mod komorebi_connect;
mod monitors_widget;

use iced::{
    color,
    widget::{checkbox, column, container, horizontal_rule, row, scrollable, text, text_input, vertical_rule, Column, Space},
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    Subscription, Task, Theme,
};

fn main() -> iced::Result {
    iced::application("Komofig", Komofig::update, Komofig::view)
        .subscription(Komofig::subscription)
        .theme(Komofig::theme)
        .run()
}

#[derive(Debug)]
enum Message {
    KomorebiNotification(komorebi_client::Notification),
    ConfigMonitor(usize),
    ToggleWorkspaceTile(usize, usize, bool),
}

#[derive(Default)]
struct Komofig {
    notifications: Vec<komorebi_client::NotificationEvent>,
    komorebi_state: Option<komorebi_client::State>,
    monitor_to_config: Option<usize>,
}

impl Komofig {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KomorebiNotification(notification) => {
                self.notifications.push(notification.event);
                self.komorebi_state = Some(notification.state);
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
            Message::ToggleWorkspaceTile(monitor_idx, workspace_idx, tile) => {
                let _ = komorebi_client::send_message(&komorebi_client::SocketMessage::WorkspaceTiling(monitor_idx, workspace_idx, tile));
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let monitors: Element<Message> = if let Some(state) = &self.komorebi_state {
            let mut col: Column<Message> = column![text("Monitors:").size(20)];
            let m: Element<Message> =
                monitors_widget::Monitors::new(state.monitors.elements().iter().collect())
                    .on_press(Message::ConfigMonitor)
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
                col = monitor.workspaces().iter().enumerate().fold(col, |col, (idx, workspace)| {
                    col.push(column![
                        row![text("Name: "), text!("{}", workspace.name().as_ref().map_or("", |v| v))],
                        row![text("Tile: "), checkbox("Tile", *workspace.tile()).on_toggle(move |c| Message::ToggleWorkspaceTile(monitor_idx, idx, c))],
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
        let col = column![text("Notifications:").size(20),];
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
        komorebi_connect::connect()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }
}
