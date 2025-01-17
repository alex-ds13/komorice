use super::{
    monitor::{self, Monitor, MonitorView},
    workspace,
};

use crate::{
    config::{DEFAULT_MONITOR_CONFIG, DEFAULT_WORKSPACE_CONFIG},
    widget::{monitors_viewer, opt_helpers},
    BOLD_FONT,
};

use std::collections::HashMap;

use iced::{
    padding,
    widget::{button, checkbox, column, container, horizontal_rule, row, scrollable, text},
    Center, Element, Fill, Subscription, Task,
};
use komorebi_client::{MonitorConfig, Rect};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigMonitor(usize),
    MonitorConfigChanged(usize, monitor::Message),
    ToggleShowMonitorsList,
    DeleteMonitor(usize),
    AddMonitorUp(usize),
    AddMonitorDown(usize),
    MoveUpMonitor(usize),
    MoveDownMonitor(usize),
    ToggleMonitorButtonHover(usize, bool),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Default)]
pub struct Monitors {
    pub monitors: HashMap<usize, Monitor>,
    pub monitor_to_config: Option<usize>,
    pub show_monitors_list: bool,
    pub monitors_buttons_hovered: Vec<bool>,
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

        let monitors_buttons_hovered = config.monitors.as_ref().map_or(Vec::new(), |monitors| {
            monitors.iter().map(|_| false).collect()
        });

        Monitors {
            monitors,
            monitor_to_config: None,
            show_monitors_list: false,
            monitors_buttons_hovered,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        monitors_config: &mut Vec<MonitorConfig>,
        display_info: &HashMap<usize, (String, Rect)>,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigMonitor(idx) => {
                if self.monitor_to_config == Some(idx) {
                    self.monitor_to_config = None;
                } else if let Some((device_id, _)) = display_info.get(&idx) {
                    println!(
                        "Go to ConfigMonitor screen for monitor {idx} with id: {}",
                        device_id
                    );
                    self.monitor_to_config = Some(idx);
                    if let Some(hovered) = self.monitors_buttons_hovered.get_mut(idx) {
                        *hovered = false;
                    }
                } else {
                    println!("Go to ConfigMonitor screen for monitor {idx} which doesn't exist");
                    self.monitor_to_config = Some(idx);
                    if let Some(hovered) = self.monitors_buttons_hovered.get_mut(idx) {
                        *hovered = false;
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
            Message::ToggleShowMonitorsList => {
                self.show_monitors_list = !self.show_monitors_list;
            }
            Message::DeleteMonitor(idx) => {
                monitors_config.remove(idx);
                self.monitors_buttons_hovered.remove(idx);
                if idx < self.monitors.len() - 1 {
                    for i in (self.monitors.len() - 1)..(idx + 1) {
                        if let Some(mut m) = self.monitors.remove(&i) {
                            m.index = i - 1;
                            self.monitors.insert(i - 1, m);
                        }
                    }
                } else {
                    self.monitors.remove(&idx);
                }
            }
            Message::AddMonitorUp(idx) => {
                monitors_config.insert(
                    idx,
                    MonitorConfig {
                        workspaces: Vec::from([DEFAULT_WORKSPACE_CONFIG.clone()]),
                        ..*DEFAULT_MONITOR_CONFIG
                    },
                );
                if let Some(hovered) = self.monitors_buttons_hovered.get_mut(idx) {
                    *hovered = false;
                }
                self.monitors_buttons_hovered.insert(idx, true);
                let mut previous_m = self.monitors.insert(
                    idx,
                    monitor::Monitor {
                        index: idx,
                        workspaces: HashMap::from([(0, workspace::Workspace::new(0))]),
                        ..Default::default()
                    },
                );
                for i in (idx + 1)..(self.monitors.len() + 1) {
                    if let Some(mut w) = previous_m {
                        w.index = i;
                        previous_m = self.monitors.insert(i, w)
                    }
                }
            }
            Message::AddMonitorDown(idx) => {
                if idx + 1 >= monitors_config.len() {
                    monitors_config.push(MonitorConfig {
                        workspaces: Vec::from([DEFAULT_WORKSPACE_CONFIG.clone()]),
                        ..*DEFAULT_MONITOR_CONFIG
                    });
                    self.monitors_buttons_hovered.push(false);
                } else {
                    monitors_config.insert(
                        idx + 1,
                        MonitorConfig {
                            workspaces: Vec::from([DEFAULT_WORKSPACE_CONFIG.clone()]),
                            ..*DEFAULT_MONITOR_CONFIG
                        },
                    );
                    self.monitors_buttons_hovered.insert(idx + 1, false);
                }
                let mut previous_m = self.monitors.insert(
                    idx + 1,
                    monitor::Monitor {
                        index: idx,
                        workspaces: HashMap::from([(0, workspace::Workspace::new(0))]),
                        ..Default::default()
                    },
                );
                for i in (idx + 2)..(self.monitors.len() + 1) {
                    if let Some(mut w) = previous_m {
                        w.index = i;
                        previous_m = self.monitors.insert(i, w)
                    }
                }
            }
            Message::MoveUpMonitor(idx) => {
                let new_idx = if idx == 0 {
                    self.monitors.len() - 1
                } else {
                    idx - 1
                };
                if let (Some(mut current), Some(mut target)) =
                    (self.monitors.remove(&idx), self.monitors.remove(&new_idx))
                {
                    current.index = new_idx;
                    target.index = idx;
                    self.monitors.insert(new_idx, current);
                    self.monitors.insert(idx, target);
                    monitors_config.swap(idx, new_idx);
                }
            }
            Message::MoveDownMonitor(idx) => {
                let new_idx = (idx + 1) % self.monitors.len();
                if let (Some(mut current), Some(mut target)) =
                    (self.monitors.remove(&idx), self.monitors.remove(&new_idx))
                {
                    current.index = new_idx;
                    target.index = idx;
                    self.monitors.insert(new_idx, current);
                    self.monitors.insert(idx, target);
                    monitors_config.swap(idx, new_idx);
                }
            }
            Message::ToggleMonitorButtonHover(idx, hover) => {
                if let Some(hovered) = self.monitors_buttons_hovered.get_mut(idx) {
                    *hovered = hover;
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        monitors_config: &'a [MonitorConfig],
        display_info: &'a HashMap<usize, (String, Rect)>,
    ) -> Element<'a, Message> {
        let mut main_title = if let Some(idx) = self.monitor_to_config {
            row![button(text("Monitors > ").size(20).font(*BOLD_FONT))
                .on_press(Message::ConfigMonitor(idx))
                .padding(0)
                .style(button::text)]
        } else {
            row![text("Monitors:").size(20).font(*BOLD_FONT)]
        };

        let mut col = column![].spacing(10).padding(padding::bottom(10).right(20));

        if let Some((Some(monitor), Some(m_config))) = self
            .monitor_to_config
            .map(|idx| (self.monitors.get(&idx), monitors_config.get(idx)))
        {
            let monitor_idx = self.monitor_to_config.expect("unreachable");
            let MonitorView { title, contents } = monitor
                .view(m_config)
                .map(move |message| Message::MonitorConfigChanged(monitor_idx, message));
            main_title = main_title.push(title);
            col = col.extend(contents);
        } else if self.show_monitors_list {
            col = monitors_config
                .iter()
                .enumerate()
                .fold(col, |col, (idx, _monitor)| {
                    col.push(opt_helpers::opt_button_add_move(
                        text!(
                            "Monitor [{}] - {}",
                            idx,
                            display_info
                                .get(&idx)
                                .map_or("[Display Not Found]", |d| d.0.as_str())
                        ),
                        None,
                        self.monitors_buttons_hovered.get(idx).map_or(false, |v| *v),
                        idx > 0,
                        idx < monitors_config.len() - 1,
                        Message::ConfigMonitor(idx),
                        Message::DeleteMonitor(idx),
                        Message::AddMonitorUp(idx),
                        Message::AddMonitorDown(idx),
                        Message::MoveUpMonitor(idx),
                        Message::MoveDownMonitor(idx),
                        |v| Message::ToggleMonitorButtonHover(idx, v),
                    ))
                });
        };

        let contents = scrollable(col).id(scrollable::Id::new("monitors_scrollable"));

        let show_monitors_display = container(
            checkbox("Show Monitors", !self.show_monitors_list)
                .on_toggle(|_| Message::ToggleShowMonitorsList),
        )
        .padding(padding::top(10));

        let monitors_display = (!self.show_monitors_list).then(|| {
            let monitors = monitors_viewer::Monitors::new(display_info)
                .selected(self.monitor_to_config)
                .on_selected(Message::ConfigMonitor);
            // let m = Element::from(m).explain(iced::color!(0x00aaff));

            container(monitors)
                .padding(10)
                .width(Fill)
                .align_x(Center)
                .style(container::rounded_box)
        });

        column![main_title, horizontal_rule(2.0), show_monitors_display]
            .push_maybe(monitors_display)
            .push(contents)
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

pub fn get_display_information() -> HashMap<usize, (String, Rect)> {
    win32_display_data::connected_displays_physical()
        .flatten()
        .enumerate()
        .map(|(i, display)| {
            let path = display.device_path.clone();

            let (_device, device_id) = if path.is_empty() {
                (String::from("UNKNOWN"), String::from("UNKNOWN"))
            } else {
                let mut split: Vec<_> = path.split('#').collect();
                split.remove(0);
                split.remove(split.len() - 1);
                let device = split[0].to_string();
                let device_id = split.join("-");
                (device, device_id)
            };

            (i, (device_id, display.size.into()))
        })
        .collect()
}

pub fn get_displays() -> Vec<komorebi_client::Monitor> {
    win32_display_data::connected_displays_physical()
        .flatten()
        .map(|display| {
            let path = display.device_path.clone();

            let (device, device_id) = if path.is_empty() {
                (String::from("UNKNOWN"), String::from("UNKNOWN"))
            } else {
                let mut split: Vec<_> = path.split('#').collect();
                split.remove(0);
                split.remove(split.len() - 1);
                let device = split[0].to_string();
                let device_id = split.join("-");
                (device, device_id)
            };

            let name = display.device_name.trim_start_matches(r"\\.\").to_string();
            let name = name.split('\\').collect::<Vec<_>>()[0].to_string();

            komorebi_client::Monitor::new(
                display.hmonitor,
                display.size.into(),
                display.work_area_size.into(),
                name,
                device,
                device_id,
            )
        })
        .collect()
}
