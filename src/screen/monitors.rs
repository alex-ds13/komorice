use super::{
    monitor::{self, Monitor},
    workspace,
};

use crate::{widget::monitors_viewer, BOLD_FONT};

use std::collections::HashMap;

use iced::{
    padding,
    widget::{checkbox, column, container, horizontal_rule, scrollable, text},
    Center, Element, Fill, Subscription, Task,
};
use komorebi_client::{MonitorConfig, Rect};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigMonitor(usize),
    MonitorConfigChanged(usize, monitor::Message),
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

        Monitors {
            monitors,
            monitor_to_config: None,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        monitors_config: &mut [MonitorConfig],
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
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        monitors_config: &'a [MonitorConfig],
        display_info: &'a HashMap<usize, (String, Rect)>,
    ) -> Element<'a, Message> {
        let title = text("Monitors:").size(20).font(*BOLD_FONT);
        let monitors: Element<Message> = {
            let mut col = column![].spacing(10).padding(padding::bottom(10).right(20));

            let m: Element<Message> = monitors_viewer::Monitors::new(display_info)
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

            if let Some((Some(monitor), Some(m_config))) = self
                .monitor_to_config
                .map(|idx| (self.monitors.get(&idx), monitors_config.get(idx)))
            {
                let monitor_idx = self.monitor_to_config.expect("unreachable");
                col = col.push(
                    monitor
                        .view(m_config)
                        .map(move |message| Message::MonitorConfigChanged(monitor_idx, message)),
                );
            }
            scrollable(col)
                .id(scrollable::Id::new("monitors_scrollable"))
                .into()
        };

        column![title, horizontal_rule(2.0), monitors]
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
