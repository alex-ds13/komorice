use super::workspace::{self, WorkspaceScreen};

use crate::widget::opt_helpers;

use std::collections::HashMap;

use iced::{padding, widget::{column, horizontal_rule, text, Column}, Element, Task};
use komorebi::MonitorConfig;

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleWindowBasedWorkAreaOffsetExpand,
    ToggleWindowBasedWorkAreaOffsetHover(bool),
    ToggleWorkAreaOffsetExpand,
    ToggleWorkAreaOffsetHover(bool),
    Workspace(usize, workspace::Message),
    ToggleWorkspacesExpand,
    ToggleWorkspacesHover(bool),
    ToggleWorkspaceExpanded(usize),
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

pub struct Monitor {
    pub index: usize,
    pub config: MonitorConfig,
    pub window_based_work_area_offset_expanded: bool,
    pub window_based_work_area_offset_hovered: bool,
    pub work_area_offset_expanded: bool,
    pub work_area_offset_hovered: bool,
    pub show_workspaces: bool,
    pub show_workspaces_hovered: bool,
    pub expanded_workspaces: HashMap<usize, bool>,
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
                            self.expanded_workspaces.entry(idx).and_modify(|expanded| *expanded = !*expanded).or_insert(true);
                        },
                    }
                }
            },
            Message::ToggleWorkspacesExpand => {
                self.show_workspaces = !self.show_workspaces;
            },
            Message::ToggleWorkspacesHover(hover) => {
                self.show_workspaces_hovered = hover;
            },
            Message::ToggleWorkspaceExpanded(idx) => {
                self.expanded_workspaces.entry(idx).and_modify(|expanded| *expanded = !*expanded).or_insert(true);
            },
            Message::Nothing => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let section_title = text!("Monitor [{}]:", self.index).size(18.0);
        let contents = [
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
            opt_helpers::expandable(
                "Workspaces",
                None,
                self.config.workspaces
                    .iter()
                    .enumerate().
                    map(|(i, w)| {
                        let title = text!("Workspace [{}] - \"{}\":", i, w.name).size(20);
                        opt_helpers::expandable(
                            title,
                            None,
                            [w.view(i, self.expanded_workspaces[&i]).map(move |m| Message::Workspace(i, m))],
                            self.expanded_workspaces[&i],
                            false,
                            Message::ToggleWorkspaceExpanded(i),
                            |_| Message::Nothing,
                        )
                    }),
                self.show_workspaces,
                self.show_workspaces_hovered,
                Message::ToggleWorkspacesExpand,
                Message::ToggleWorkspacesHover,
            ),
        ];
        column![
            section_title,
            horizontal_rule(2.0),
            Column::with_children(contents)
                .padding(padding::top(10).bottom(10))
                .spacing(10),
        ]
        .spacing(10)
        .into()
    }
}
