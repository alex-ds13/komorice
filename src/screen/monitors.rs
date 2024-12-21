use super::{
    monitor::{self, Monitor},
    workspace,
};

use crate::{widget::monitors_viewer, BOLD_FONT};

use std::{collections::HashMap, sync::Arc};

use iced::{
    padding,
    widget::{checkbox, column, container, horizontal_rule, row, scrollable, text, Space},
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    Subscription, Task,
};
use komorebi_client::MonitorConfig;

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
    pub fn new(
        config: &komorebi_client::StaticConfig,
        state: &Option<Arc<komorebi_client::State>>,
    ) -> Self {
        let mut monitors = config.monitors.as_ref().map_or(HashMap::new(), |monitors| {
            monitors
                .iter()
                .enumerate()
                .map(|(index, m)| {
                    (
                        index,
                        monitor::Monitor {
                            index,
                            sub_screen: monitor::SubScreen::Monitor,
                            window_based_work_area_offset_expanded: false,
                            window_based_work_area_offset_hovered: false,
                            work_area_offset_expanded: false,
                            work_area_offset_hovered: false,
                            workspaces_button_hovered: false,
                            workspaces: m
                                .workspaces
                                .iter()
                                .enumerate()
                                .map(|(i, _)| (i, workspace::Workspace::new(i)))
                                .collect(),
                        },
                    )
                })
                .collect()
        });

        // If there are more monitors physically than on the config, then we will create a default
        // config for each one of them so that the user can change it if they want to.
        if let Some(state) = state {
            while monitors.len() < state.monitors.elements().len() {
                monitors.insert(monitors.len(), monitor::DEFAULT_MONITOR.clone());
            }
        }

        Monitors {
            monitors,
            monitor_to_config: None,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        komorebi_state: &Option<Arc<komorebi_client::State>>,
        monitors_config: &mut [MonitorConfig],
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
                if let (Some(m), Some(m_config)) =
                    (self.monitors.get_mut(&idx), monitors_config.get_mut(idx))
                {
                    return (
                        Action::None,
                        m.update(message, m_config)
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
        monitors_config: &'a [MonitorConfig],
    ) -> Element<'a, Message> {
        let title = text("Monitors:").size(20).font(*BOLD_FONT);
        let monitors: Element<Message> =
            if let Some(state) = &komorebi_state {
                let mut col = column![]
                    .spacing(10)
                    .padding(padding::top(10).bottom(10).right(20));

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

                if let Some((Some(device), Some(monitor), Some(m_config))) =
                    self.monitor_to_config.map(|idx| {
                        (
                            state.monitors.elements().get(idx),
                            self.monitors.get(&idx),
                            monitors_config.get(idx),
                        )
                    })
                {
                    let monitor_idx = self.monitor_to_config.expect("unreachable");
                    col =
                        col.push(monitor.view(m_config).map(move |message| {
                            Message::MonitorConfigChanged(monitor_idx, message)
                        }));
                    col = col.push(horizontal_rule(2.0));
                    col = col.push(column![
                        text!("Monitor {}:", monitor_idx).size(16),
                        text!("    -> Id: {}", device.id()),
                        text!("    -> DeviceId: {}", device.device_id()),
                        text!("    -> Device: {}", device.device()),
                        text!("    -> Size: {:#?}", device.size()),
                    ]);
                    col = col.push(horizontal_rule(2.0));
                    col = col.push(text("Workspaces:"));
                    col = device.workspaces().iter().enumerate().fold(
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
                scrollable(col)
                    .id(scrollable::Id::new("monitors_scrollable"))
                    .into()
            } else {
                Space::new(Shrink, Shrink).into()
            };
        column![title, horizontal_rule(2.0), monitors,]
            .spacing(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(monitor) = self
            .monitor_to_config
            .and_then(|idx| self.monitors.get(&idx))
        {
            monitor
                .subscription()
                .map(|(m_idx, _w_idx, m)| Message::MonitorConfigChanged(m_idx, m))
        } else {
            Subscription::none()
        }
    }
}
