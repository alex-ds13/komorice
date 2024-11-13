use super::monitor::{self, Monitor};

use crate::{widget::monitors_viewer, BOLD_FONT};

use std::{collections::HashMap, sync::Arc};

use iced::{
    padding,
    widget::{checkbox, column, container, horizontal_rule, row, scrollable, text, Space},
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    Task,
};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigMonitor(usize),
    MonitorConfigChanged(usize, monitor::Message),

    // Komorebi Command Messages
    ToggleWorkspaceTile(usize, usize, bool),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Default)]
pub struct Monitors {
    pub monitors: HashMap<usize, Monitor>,
    pub monitor_to_config: Option<usize>,
}

impl Monitors {
    pub fn new(config: &komorebi_client::StaticConfig) -> Self {
        let monitors = config.monitors.as_ref().map_or(HashMap::new(), |monitors| {
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
        Monitors {
            monitors,
            monitor_to_config: None,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        komorebi_state: &Option<Arc<komorebi_client::State>>,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigMonitor(idx) => {
                if self.monitor_to_config == Some(idx) {
                    self.monitor_to_config = None;
                } else if let Some(state) = komorebi_state {
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
            Message::MonitorConfigChanged(idx, message) => {
                if let Some(m) = self.monitors.get_mut(&idx) {
                    return (
                        Action::None,
                        m.update(message)
                            .map(move |message| Message::MonitorConfigChanged(idx, message)),
                    );
                }
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
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        komorebi_state: &'a Option<Arc<komorebi_client::State>>,
    ) -> Element<'a, Message> {
        let title = text("Monitors:").size(20).font(*BOLD_FONT);
        let monitors: Element<Message> = if let Some(state) = &komorebi_state {
            let mut col = column![].spacing(10).padding(padding::top(10).bottom(10).right(20));

            let m = monitors_viewer::Monitors::new(state.monitors.elements().iter().collect())
                .selected(self.monitor_to_config)
                .on_selected(Message::ConfigMonitor);
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
                col = col.push(
                    m.view()
                        .map(move |message| Message::MonitorConfigChanged(monitor_idx, message)),
                );
                col = col.push(horizontal_rule(2.0));
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
            scrollable(col).into()
        } else {
            Space::new(Shrink, Shrink).into()
        };
        column![title, horizontal_rule(2.0), monitors,]
            .spacing(10)
            .into()
    }
}
