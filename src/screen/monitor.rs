use super::workspace::{self, WorkspaceScreen};

use crate::widget::opt_helpers;

use std::collections::HashMap;

use iced::{widget::{button, row, text}, Element, Task};
use komorebi::MonitorConfig;

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleWindowBasedWorkAreaOffsetExpand,
    ToggleWindowBasedWorkAreaOffsetHover(bool),
    ToggleWorkAreaOffsetExpand,
    ToggleWorkAreaOffsetHover(bool),
    Workspace(usize, workspace::Message),
    SetSubScreenMonitor,
    SetSubScreenWorkspaces,
    SetSubScreenWorkspace(usize),
    ToggleWorkspacesHover(bool),
    ToggleWorkspaceHover(usize, bool),
    Nothing,
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    WindowBasedWorkAreaOffset(komorebi::Rect),
    WindowBasedWorkAreaOffsetTop(i32),
    WindowBasedWorkAreaOffsetBottom(i32),
    WindowBasedWorkAreaOffsetRight(i32),
    WindowBasedWorkAreaOffsetLeft(i32),
    WindowBasedWorkAreaOffsetLimit(i32),
    WorkAreaOffset(komorebi::Rect),
    WorkAreaOffsetTop(i32),
    WorkAreaOffsetBottom(i32),
    WorkAreaOffsetRight(i32),
    WorkAreaOffsetLeft(i32),
}

#[derive(Clone, Debug, Default)]
pub enum SubScreen {
    #[default]
    Monitor,
    Workspaces,
    Workspace(usize),
}

pub struct Monitor {
    pub index: usize,
    pub sub_screen: SubScreen,
    pub config: MonitorConfig,
    pub window_based_work_area_offset_expanded: bool,
    pub window_based_work_area_offset_hovered: bool,
    pub work_area_offset_expanded: bool,
    pub work_area_offset_hovered: bool,
    pub show_workspaces: bool,
    pub show_workspaces_hovered: bool,
    pub hovered_workspaces: HashMap<usize, bool>,
}

impl Monitor {

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
    Message::ConfigChange(change) => match change {
                ConfigChange::WindowBasedWorkAreaOffset(_) => todo!(),
                ConfigChange::WindowBasedWorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut self.config.window_based_work_area_offset {
                        offset.top = value;
                    } else {
                        self.config.window_based_work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: value,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetBottom(value) => {
                    if let Some(offset) = &mut self.config.window_based_work_area_offset {
                        offset.bottom = value;
                    } else {
                        self.config.window_based_work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: value,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetRight(value) => {
                    if let Some(offset) = &mut self.config.window_based_work_area_offset {
                        offset.right = value;
                    } else {
                        self.config.window_based_work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: 0,
                            right: value,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetLeft(value) => {
                    if let Some(offset) = &mut self.config.window_based_work_area_offset {
                        offset.left = value;
                    } else {
                        self.config.window_based_work_area_offset = Some(komorebi::Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetLimit(limit) => {
                    self.config.window_based_work_area_offset_limit = Some(limit.try_into().unwrap_or_default());
                }
                ConfigChange::WorkAreaOffset(rect) => {
                    self.config.work_area_offset = Some(rect);
                }
                ConfigChange::WorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut self.config.work_area_offset {
                        offset.top = value;
                    } else {
                        self.config.work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: value,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetBottom(value) => {
                    if let Some(offset) = &mut self.config.work_area_offset {
                        offset.bottom = value;
                    } else {
                        self.config.work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: value,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetRight(value) => {
                    if let Some(offset) = &mut self.config.work_area_offset {
                        offset.right = value;
                    } else {
                        self.config.work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: 0,
                            right: value,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetLeft(value) => {
                    if let Some(offset) = &mut self.config.work_area_offset {
                        offset.left = value;
                    } else {
                        self.config.work_area_offset = Some(komorebi::Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
            },
            Message::ToggleWindowBasedWorkAreaOffsetExpand => {
                self.window_based_work_area_offset_expanded = !self.window_based_work_area_offset_expanded;
            },
            Message::ToggleWindowBasedWorkAreaOffsetHover(hover) => {
                self.window_based_work_area_offset_hovered = hover;
            },
            Message::ToggleWorkAreaOffsetExpand => {
                self.work_area_offset_expanded = !self.work_area_offset_expanded;
            },
            Message::ToggleWorkAreaOffsetHover(hover) => {
                self.work_area_offset_hovered = hover;
            },
            Message::Workspace(idx, message) => {
                if let Some(workspace) = self.config.workspaces.get_mut(idx) {
                    let action = workspace.update(message);
                    match action {
                        workspace::Action::None => {},
                        workspace::Action::ToggleExpanded(idx) => {
                            self.hovered_workspaces.entry(idx).and_modify(|expanded| *expanded = !*expanded).or_insert(true);
                        },
                    }
                }
            },
            Message::SetSubScreenMonitor => {
                self.sub_screen = SubScreen::Monitor;
            },
            Message::SetSubScreenWorkspaces => {
                self.sub_screen = SubScreen::Workspaces;
                self.show_workspaces_hovered = false;
            },
            Message::SetSubScreenWorkspace(idx) => {
                self.sub_screen = SubScreen::Workspace(idx);
                self.hovered_workspaces.entry(idx).and_modify(|hovered| *hovered = false).or_insert(false);
            },
            Message::ToggleWorkspacesHover(hover) => {
                self.show_workspaces_hovered = hover;
            },
            Message::ToggleWorkspaceHover(idx, hover) => {
                self.hovered_workspaces.entry(idx).and_modify(|hovered| *hovered = hover).or_insert(true);
            },
            Message::Nothing => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        match self.sub_screen {
            SubScreen::Monitor => self.monitor_view(),
            SubScreen::Workspaces => self.workspaces_view(),
            SubScreen::Workspace(idx) => self.workspace_view(idx),
        }
    }

    pub fn monitor_view(&self) -> Element<Message> {
        opt_helpers::sub_section_view(
            text!("Monitor [{}]:", self.index).size(18).into(),
            [
                opt_helpers::expandable(
                    "Window Based Work Area Offset",
                    Some("Window based work area offset (default: None)"),
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            self.config.window_based_work_area_offset.map_or(0, |r| r.left),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetLeft(value)),
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            self.config.window_based_work_area_offset.map_or(0, |r| r.top),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetTop(value)),
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            self.config.window_based_work_area_offset.map_or(0, |r| r.bottom),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetBottom(value)),
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            self.config.window_based_work_area_offset.map_or(0, |r| r.right),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetRight(value)),
                        ),
                    ],
                    self.window_based_work_area_offset_expanded,
                    self.window_based_work_area_offset_hovered,
                    Message::ToggleWindowBasedWorkAreaOffsetExpand,
                    Message::ToggleWindowBasedWorkAreaOffsetHover,
                ),
                opt_helpers::number(
                    "Window Based Work Area Offset Limit",
                    Some("Open window limit after which the window based work area offset will no longer be applied (default: 1)"),
                    self.config.window_based_work_area_offset_limit.unwrap_or(1).try_into().unwrap_or_default(),
                    move |value| {
                        Message::ConfigChange(
                            
                            ConfigChange::WindowBasedWorkAreaOffsetLimit(value),
                        )
                    },
                ),
                opt_helpers::expandable(
                    "Work Area Offset",
                    Some("Monitor-specific work area offset (default: None)"),
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            self.config.work_area_offset.map_or(0, |r| r.left),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetLeft(value)),
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            self.config.work_area_offset.map_or(0, |r| r.top),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetTop(value)),
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            self.config.work_area_offset.map_or(0, |r| r.bottom),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetBottom(value)),
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            self.config.work_area_offset.map_or(0, |r| r.right),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetRight(value)),
                        ),
                    ],
                    self.work_area_offset_expanded,
                    self.work_area_offset_hovered,
                    Message::ToggleWorkAreaOffsetExpand,
                    Message::ToggleWorkAreaOffsetHover,
                ),
                opt_helpers::opt_button(
                    "Workspaces",
                    None,
                    self.show_workspaces_hovered,
                    Message::SetSubScreenWorkspaces,
                    Message::ToggleWorkspacesHover,
                ),
            ]
        )
    }

    pub fn workspaces_view(&self) -> Element<Message> {
        opt_helpers::sub_section_view(
            row![
                nav_button(text!("Monitor [{}] ", self.index), Message::SetSubScreenMonitor),
                text("> Workspaces").size(18)
            ].into(),
            self.config.workspaces
                .iter()
                .enumerate().
                map(|(i, w)| {
                    let title = text!("Workspace [{}] - \"{}\":", i, w.name);
                    opt_helpers::opt_button(
                        title,
                        None,
                        self.hovered_workspaces[&i],
                        Message::SetSubScreenWorkspace(i),
                        |v| Message::ToggleWorkspaceHover(i, v),
                    )
                }),
        )
    }

    pub fn workspace_view(&self, idx: usize) -> Element<Message> {
        opt_helpers::sub_section_view(
            row![
                nav_button(text!("Monitor [{}] ", self.index), Message::SetSubScreenMonitor),
                nav_button(text("> Workspaces"), Message::SetSubScreenWorkspaces),
                text!(" > Workspace [{}] - \"{}\"", idx, self.config.workspaces[idx].name).size(18),
            ].into(),
            [self.config.workspaces[idx].view(idx, self.hovered_workspaces[&idx]).map(move |m| Message::Workspace(idx, m))],
        )
    }
}

fn nav_button<'a>(content: impl Into<iced::widget::Text<'a>>, on_press: Message) -> iced::widget::Button<'a, Message> {
    button(content.into().size(18)).on_press(on_press).padding(0).style(button::text)
}
