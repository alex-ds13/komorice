use super::{
    monitor::{self, Monitor, MonitorView},
    workspace,
};

use crate::{
    BOLD_FONT,
    config::{DEFAULT_MONITOR_CONFIG, DEFAULT_WORKSPACE_CONFIG},
    widget::{icons, monitors_viewer, opt_helpers},
};

use std::collections::{BTreeMap, HashMap};

use iced::{
    Center, Element, Fill, Subscription, Task, padding,
    widget::{Id, button, checkbox, column, container, row, rule, scrollable, space, text},
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
    ChangeNewIndexPreferenceIndex(usize),
    ChangeNewIndexPreferenceId(String),
    AddNewIndexPreference,
    RemoveIndexPreference(usize),
    ChangeIndexPreferenceIndex(usize, usize),
    ChangeIndexPreferenceId(usize, String),
    ChangeDisplayIndexPreferences(Option<HashMap<usize, String>>),
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
    pub new_idx_preference_index: usize,
    pub new_idx_preference_id: String,
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
            show_monitors_list: false,
            new_idx_preference_id: String::new(),
            new_idx_preference_index: 0,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        monitors_config: &mut Vec<MonitorConfig>,
        display_index_preferences: &mut Option<HashMap<usize, String>>,
        display_info: &mut HashMap<usize, DisplayInfo>,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigMonitor(idx) => {
                if self.monitor_to_config == Some(idx) {
                    self.monitor_to_config = None;
                } else if let Some(DisplayInfo {
                    device_id: _device_id,
                    ..
                }) = display_info.get(&idx)
                {
                    // println!(
                    //     "Go to ConfigMonitor screen for monitor {idx} with id: {}",
                    //     _device_id
                    // );
                    self.monitor_to_config = Some(idx);
                } else {
                    // println!("Go to ConfigMonitor screen for monitor {idx} which doesn't exist");
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
            Message::ToggleShowMonitorsList => {
                self.show_monitors_list = !self.show_monitors_list;
            }
            Message::DeleteMonitor(idx) => {
                monitors_config.remove(idx);
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
                        ..DEFAULT_MONITOR_CONFIG.clone()
                    },
                );
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
                        ..DEFAULT_MONITOR_CONFIG.clone()
                    });
                } else {
                    monitors_config.insert(
                        idx + 1,
                        MonitorConfig {
                            workspaces: Vec::from([DEFAULT_WORKSPACE_CONFIG.clone()]),
                            ..DEFAULT_MONITOR_CONFIG.clone()
                        },
                    );
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
            Message::ChangeNewIndexPreferenceIndex(idx) => {
                self.new_idx_preference_index = idx;
            }
            Message::ChangeNewIndexPreferenceId(id) => self.new_idx_preference_id = id,
            Message::AddNewIndexPreference => {
                if let Some(dip) = display_index_preferences {
                    let id = std::mem::take(&mut self.new_idx_preference_id);
                    dip.insert(self.new_idx_preference_index, id);
                    self.new_idx_preference_index = 0;
                } else {
                    let mut dip = HashMap::new();
                    let id = std::mem::take(&mut self.new_idx_preference_id);
                    dip.insert(self.new_idx_preference_index, id);
                    self.new_idx_preference_index = 0;
                    *display_index_preferences = Some(dip);
                }
                *display_info = get_display_information(display_index_preferences);
            }
            Message::RemoveIndexPreference(idx) => {
                if display_index_preferences
                    .as_mut()
                    .and_then(|dip| dip.remove(&idx))
                    .is_some()
                {
                    *display_info = get_display_information(display_index_preferences);
                }
            }
            Message::ChangeIndexPreferenceIndex(idx, new_idx) => {
                if let Some(dip) = display_index_preferences
                    && let Some(preference) = dip.remove(&idx)
                {
                    dip.insert(new_idx, preference);
                    *display_info = get_display_information(display_index_preferences);
                }
            }
            Message::ChangeIndexPreferenceId(idx, new_id) => {
                if let Some(dip) = display_index_preferences {
                    dip.insert(idx, new_id);
                    *display_info = get_display_information(display_index_preferences);
                }
            }
            Message::ChangeDisplayIndexPreferences(dip) => {
                *display_index_preferences = dip;
                *display_info = get_display_information(display_index_preferences);
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        monitors_config: &'a [MonitorConfig],
        display_info: &'a HashMap<usize, DisplayInfo>,
        display_index_preferences: &'a Option<HashMap<usize, String>>,
    ) -> Element<'a, Message> {
        let mut main_title = if let Some(idx) = self.monitor_to_config {
            row![
                button(text("Monitors > ").size(20).font(*BOLD_FONT))
                    .on_press(Message::ConfigMonitor(idx))
                    .padding(0)
                    .style(button::text)
            ]
        } else {
            row![text("Monitors:").size(20).font(*BOLD_FONT)]
        };

        let mut col = column![space::horizontal()]
            .spacing(10)
            .padding(padding::bottom(10).right(20));

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
                    let info = display_index_preferences.as_ref().map_or(
                        display_info
                            .get(&idx)
                            .map_or("[Display Not Found]", |d| &d.device_id),
                        |dip| dip.get(&idx).map_or("[Display Not Found]", |d| d),
                    );
                    col.push(opt_helpers::opt_button_add_move(
                        format!("Monitor [{}] - {}", idx, info),
                        None,
                        monitors_config.len() > 1,
                        idx > 0,
                        idx < monitors_config.len() - 1,
                        Message::ConfigMonitor(idx),
                        Message::DeleteMonitor(idx),
                        Message::AddMonitorUp(idx),
                        Message::AddMonitorDown(idx),
                        Message::MoveUpMonitor(idx),
                        Message::MoveDownMonitor(idx),
                    ))
                });
        };

        let dip = self.monitor_to_config.is_none().then(|| {
            opt_helpers::expandable(
                "Display Index Preferences",
                Some(
                    "Set display index preferences (default: None)\n\n\
                    Define which config index to use for a specific monitor, using its id. \
                    This id can either be the 'serial_number_id' or 'device_id'. You can get \
                    these values by running the command:\n\
                    > 'komorebic monitor-info'\n\
                    Sometimes the 'device_id' might change on restart, so it is better to use \
                    the 'serial_number_id' instead!",
                ),
                || self.display_index_preference_children(display_index_preferences),
                display_index_preferences.is_some(),
                Message::ChangeDisplayIndexPreferences(None),
                Some(opt_helpers::DisableArgs {
                    disable: display_index_preferences.is_none(),
                    label: Some("None"),
                    on_toggle: |v| {
                        Message::ChangeDisplayIndexPreferences((!v).then_some(HashMap::new()))
                    },
                }),
            )
        });

        col = col.push(dip);
        let contents = scrollable(col).id(Id::new("monitors_scrollable"));

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

        column![main_title, rule::horizontal(2.0), show_monitors_display]
            .push(monitors_display)
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

    fn display_index_preference_children<'a>(
        &'a self,
        display_index_preferences: &'a Option<HashMap<usize, String>>,
    ) -> Vec<Element<'a, Message>> {
        let mut children = Vec::new();
        let new_preference = opt_helpers::opt_box(
            column![
                text("Add New Display Index Preference:"),
                index_preference(
                    self.new_idx_preference_index,
                    &self.new_idx_preference_id,
                    Message::ChangeNewIndexPreferenceIndex,
                    Message::ChangeNewIndexPreferenceId,
                    true,
                ),
            ]
            .spacing(10),
        );
        children.push(new_preference.into());
        let mut preferences = display_index_preferences
            .as_ref()
            .map_or(Vec::new(), |dip| {
                dip.iter()
                    .collect::<BTreeMap<&usize, &String>>()
                    .into_iter()
                    .map(|(index, id)| {
                        let index = *index;
                        index_preference(
                            index,
                            id,
                            move |new_index| Message::ChangeIndexPreferenceIndex(index, new_index),
                            move |new_behaviour| {
                                Message::ChangeIndexPreferenceId(index, new_behaviour)
                            },
                            false,
                        )
                    })
                    .collect()
            });
        if preferences.is_empty() {
            preferences.push(text("Preferences:").into());
            // The 30.8 height came from trial and error to make it so the space is the
            // same as the one from one rule. Why isn't it 30, I don't know?! Any other
            // value other 30.8 would result in the UI adjusting when adding first rule.
            preferences.push(space().height(30.8).into());
        } else {
            preferences.insert(0, text("Preferences:").into());
        }
        children.push(rule::horizontal(2.0).into());
        children.extend(preferences);
        children
    }
}

fn index_preference<'a>(
    index: usize,
    id: &'a str,
    index_message: impl Fn(usize) -> Message + Copy + 'static,
    id_changed_message: impl Fn(String) -> Message + 'a,
    is_add: bool,
) -> Element<'a, Message> {
    let number = opt_helpers::number_simple(index, index_message).width(50);
    let input = container(crate::widget::input("", id, id_changed_message, None))
        .max_width(200)
        .width(Fill);
    let final_button = if is_add {
        button(icons::plus().style(|t| text::Style {
            color: t.palette().primary.into(),
        }))
        .on_press(Message::AddNewIndexPreference)
        .style(button::text)
    } else {
        button(icons::delete().style(|t| text::Style {
            color: t.palette().danger.into(),
        }))
        .on_press(Message::RemoveIndexPreference(index))
        .style(button::text)
    };
    row![
        text("Use config index "),
        number,
        text("for monitor with id "),
        input,
        final_button,
    ]
    .spacing(5)
    .align_y(Center)
    .into()
}

#[derive(Debug, Default, Clone)]
pub struct DisplayInfo {
    pub device_id: String,
    pub serial_number_id: Option<String>,
    pub size: Rect,
}

pub fn get_display_information(
    display_index_preferences: &Option<HashMap<usize, String>>,
) -> HashMap<usize, DisplayInfo> {
    let devices = std::thread::spawn(|| {
        // Since `win32_display_data` has some `COMLibrary` thing going on it can't be called on
        // the main thread otherwise it panics with:
        // ```
        // OleInitialize failed! Result was: `RPC_E_CHANGED_MODE`. Make sure other crates are not
        // using multithreaded COM library on the same thread or disable drag and drop support.
        // ```
        // To prevent this we spawn a new thread and join it immediately to get the result.
        komorebi_client::win32_display_data::connected_displays_all()
            .flatten()
            .map(|display| {
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

                DisplayInfo {
                    device_id,
                    serial_number_id: display.serial_number_id,
                    size: display.size.into(),
                }
            })
            .collect::<Vec<_>>()
    })
    .join()
    .unwrap_or_default();

    let configs_with_preference = display_index_preferences
        .as_ref()
        .map_or(Vec::new(), |dip| dip.keys().copied().collect());
    let mut configs_used = Vec::new();

    let devices_count = devices.len();
    devices
        .into_iter()
        .flat_map(|display| {
            let preferred_config_idx = display_index_preferences.as_ref().and_then(|dpi| {
                dpi.iter().find_map(|(c_idx, id)| {
                    (display.serial_number_id.as_ref().is_some_and(|sn| sn == id)
                        || &display.device_id == id)
                        .then_some(*c_idx)
                })
            });
            let idx = preferred_config_idx.or({
                // Display without preferred config idx.
                // Get index of first config that is not a preferred config of some other display
                // and that has not been used yet. This might return `None` as well, in that case
                // this display won't have a config tied to it.
                (0..devices_count)
                    .find(|i| !configs_with_preference.contains(i) && !configs_used.contains(i))
            });
            if let Some(idx) = idx {
                configs_used.push(idx);
                Some((idx, display))
            } else {
                None
            }
        })
        .collect()
}
