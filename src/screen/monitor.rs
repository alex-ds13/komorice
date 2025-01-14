use super::workspace::{self, WorkspaceScreen};

use crate::{config::DEFAULT_MONITOR_CONFIG, widget::opt_helpers};

use std::collections::HashMap;

use iced::{
    widget::{button, row, text},
    Element, Subscription, Task,
};
use komorebi_client::{MonitorConfig, Rect, WorkspaceConfig};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEFAULT_MONITOR: Monitor = Default::default();
}

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
    DeleteWorkspace(usize),
    MoveUpWorkspace(usize),
    MoveDownWorkspace(usize),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    WindowBasedWorkAreaOffset(Option<Rect>),
    WindowBasedWorkAreaOffsetTop(i32),
    WindowBasedWorkAreaOffsetBottom(i32),
    WindowBasedWorkAreaOffsetRight(i32),
    WindowBasedWorkAreaOffsetLeft(i32),
    WindowBasedWorkAreaOffsetLimit(i32),
    WorkAreaOffset(Option<Rect>),
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
    WorkspaceRules(usize),
    InitialWorkspaceRules(usize),
}

#[derive(Clone, Debug, Default)]
pub struct Monitor {
    pub index: usize,
    pub sub_screen: SubScreen,
    pub window_based_work_area_offset_expanded: bool,
    pub window_based_work_area_offset_hovered: bool,
    pub work_area_offset_expanded: bool,
    pub work_area_offset_hovered: bool,
    pub workspaces_button_hovered: bool,
    pub workspaces: HashMap<usize, workspace::Workspace>,
}

impl Monitor {
    pub fn update(&mut self, message: Message, config: &mut MonitorConfig) -> Task<Message> {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::WindowBasedWorkAreaOffset(rect) => {
                    config.window_based_work_area_offset = rect;
                }
                ConfigChange::WindowBasedWorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut config.window_based_work_area_offset {
                        offset.top = value;
                    } else {
                        config.window_based_work_area_offset = Some(Rect {
                            left: 0,
                            top: value,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetBottom(value) => {
                    if let Some(offset) = &mut config.window_based_work_area_offset {
                        offset.bottom = value;
                    } else {
                        config.window_based_work_area_offset = Some(Rect {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: value,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetRight(value) => {
                    if let Some(offset) = &mut config.window_based_work_area_offset {
                        offset.right = value;
                    } else {
                        config.window_based_work_area_offset = Some(Rect {
                            left: 0,
                            top: 0,
                            right: value,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetLeft(value) => {
                    if let Some(offset) = &mut config.window_based_work_area_offset {
                        offset.left = value;
                    } else {
                        config.window_based_work_area_offset = Some(Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WindowBasedWorkAreaOffsetLimit(limit) => {
                    config.window_based_work_area_offset_limit =
                        Some(limit.try_into().unwrap_or_default());
                }
                ConfigChange::WorkAreaOffset(rect) => {
                    config.work_area_offset = rect;
                }
                ConfigChange::WorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut config.work_area_offset {
                        offset.top = value;
                    } else {
                        config.work_area_offset = Some(Rect {
                            left: 0,
                            top: value,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetBottom(value) => {
                    if let Some(offset) = &mut config.work_area_offset {
                        offset.bottom = value;
                    } else {
                        config.work_area_offset = Some(Rect {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: value,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetRight(value) => {
                    if let Some(offset) = &mut config.work_area_offset {
                        offset.right = value;
                    } else {
                        config.work_area_offset = Some(Rect {
                            left: 0,
                            top: 0,
                            right: value,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetLeft(value) => {
                    if let Some(offset) = &mut config.work_area_offset {
                        offset.left = value;
                    } else {
                        config.work_area_offset = Some(Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
            },
            Message::ToggleWindowBasedWorkAreaOffsetExpand => {
                self.window_based_work_area_offset_expanded =
                    !self.window_based_work_area_offset_expanded;
            }
            Message::ToggleWindowBasedWorkAreaOffsetHover(hover) => {
                self.window_based_work_area_offset_hovered = hover;
            }
            Message::ToggleWorkAreaOffsetExpand => {
                self.work_area_offset_expanded = !self.work_area_offset_expanded;
            }
            Message::ToggleWorkAreaOffsetHover(hover) => {
                self.work_area_offset_hovered = hover;
            }
            Message::Workspace(idx, message) => {
                if let (Some(workspace_config), Some(workspace)) = (
                    config.workspaces.get_mut(idx),
                    self.workspaces.get_mut(&idx),
                ) {
                    let (action, task) = workspace_config.update(workspace, message);
                    match action {
                        workspace::Action::None => {}
                        workspace::Action::ScreenChange(ws_screen) => match ws_screen {
                            workspace::Screen::Workspace => {
                                self.sub_screen = SubScreen::Workspace(idx)
                            }
                            workspace::Screen::WorkspaceRules => {
                                self.sub_screen = SubScreen::WorkspaceRules(idx)
                            }
                            workspace::Screen::InitialWorkspaceRules => {
                                self.sub_screen = SubScreen::InitialWorkspaceRules(idx)
                            }
                        },
                    }
                    return task.map(move |m| Message::Workspace(idx, m));
                }
            }
            Message::SetSubScreenMonitor => {
                self.sub_screen = SubScreen::Monitor;
                return iced::widget::scrollable::scroll_to(
                    iced::widget::scrollable::Id::new("monitors_scrollable"),
                    iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            Message::SetSubScreenWorkspaces => {
                self.sub_screen = SubScreen::Workspaces;
                self.workspaces_button_hovered = false;
                return iced::widget::scrollable::scroll_to(
                    iced::widget::scrollable::Id::new("monitors_scrollable"),
                    iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            Message::SetSubScreenWorkspace(idx) => {
                self.sub_screen = SubScreen::Workspace(idx);
                self.workspaces
                    .entry(idx)
                    .and_modify(|ws| ws.is_hovered = false)
                    .or_default()
                    .screen = workspace::Screen::Workspace;
                return iced::widget::scrollable::scroll_to(
                    iced::widget::scrollable::Id::new("monitors_scrollable"),
                    iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                );
            }
            Message::ToggleWorkspacesHover(hover) => {
                self.workspaces_button_hovered = hover;
            }
            Message::ToggleWorkspaceHover(idx, hover) => {
                let ws = self.workspaces.entry(idx).or_default();
                ws.is_hovered = hover;
            }
            Message::DeleteWorkspace(idx) => {
                config.workspaces.remove(idx);
                println!("REMOVED: {idx}");
                if idx < self.workspaces.len() - 1 {
                    for i in (self.workspaces.len() - 1)..(idx + 1) {
                        println!("MOVING: {i} to {}", i - 1);
                        if let Some(mut w) = self.workspaces.remove(&i) {
                            w.index = i - 1;
                            self.workspaces.insert(i - 1, w);
                        }
                    }
                } else {
                    self.workspaces.remove(&idx);
                }
                let ws = self.workspaces.entry(idx).or_default();
                ws.is_hovered = true;
            }
            Message::MoveUpWorkspace(idx) => {
                let new_idx = if idx == 0 {
                    self.workspaces.len() - 1
                } else {
                    idx - 1
                };
                if let (Some(mut current), Some(mut target)) = (
                    self.workspaces.remove(&idx),
                    self.workspaces.remove(&new_idx),
                ) {
                    current.index = new_idx;
                    target.index = idx;
                    self.workspaces.insert(new_idx, current);
                    self.workspaces.insert(idx, target);
                    config.workspaces.swap(idx, new_idx);
                    let ws = self.workspaces.entry(new_idx).or_default();
                    ws.is_hovered = false;
                    let ws = self.workspaces.entry(idx).or_default();
                    ws.is_hovered = true;
                }
            }
            Message::MoveDownWorkspace(idx) => {
                let new_idx = (idx + 1) % self.workspaces.len();
                if let (Some(mut current), Some(mut target)) = (
                    self.workspaces.remove(&idx),
                    self.workspaces.remove(&new_idx),
                ) {
                    current.index = new_idx;
                    target.index = idx;
                    self.workspaces.insert(new_idx, current);
                    self.workspaces.insert(idx, target);
                    config.workspaces.swap(idx, new_idx);
                    let ws = self.workspaces.entry(new_idx).or_default();
                    ws.is_hovered = false;
                    let ws = self.workspaces.entry(idx).or_default();
                    ws.is_hovered = true;
                }
            }
        }
        Task::none()
    }

    pub fn view<'a>(&'a self, config: &'a MonitorConfig) -> Element<'a, Message> {
        match self.sub_screen {
            SubScreen::Monitor => self.monitor_view(config),
            SubScreen::Workspaces => self.workspaces_view(&config.workspaces),
            SubScreen::Workspace(idx) => self.workspace_view(idx, &config.workspaces[idx]),
            SubScreen::WorkspaceRules(idx) => {
                self.workspace_rules_view(idx, &config.workspaces[idx])
            }
            SubScreen::InitialWorkspaceRules(idx) => {
                self.initial_workspace_rules_view(idx, &config.workspaces[idx])
            }
        }
    }

    pub fn monitor_view(&self, config: &MonitorConfig) -> Element<Message> {
        opt_helpers::sub_section_view(
            text!("Monitor [{}]:", self.index).size(18).into(),
            [
                opt_helpers::expandable_with_disable_default(
                    "Window Based Work Area Offset",
                    Some("Window based work area offset (default: global)"),
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.left),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetLeft(value)),
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.top),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetTop(value)),
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.bottom),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetBottom(value)),
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.right),
                            move |value| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetRight(value)),
                        ),
                    ],
                    self.window_based_work_area_offset_expanded,
                    self.window_based_work_area_offset_hovered,
                    Message::ToggleWindowBasedWorkAreaOffsetExpand,
                    Message::ToggleWindowBasedWorkAreaOffsetHover,
                    config.window_based_work_area_offset.is_some(),
                    Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffset(None)),
                    Some(opt_helpers::DisableArgs {
                        disable: config.window_based_work_area_offset.is_none(),
                        label: Some("Global"),
                        on_toggle: |v| Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffset((!v).then_some(Rect::default()))),
                    }),
                ),
                opt_helpers::number_with_disable_default(
                    "Window Based Work Area Offset Limit",
                    Some("Open window limit after which the window based work area offset will no longer be applied (default: 1)"),
                    config.window_based_work_area_offset_limit.unwrap_or(1).try_into().unwrap_or_default(),
                    DEFAULT_MONITOR_CONFIG.window_based_work_area_offset_limit.unwrap_or(1).try_into().unwrap_or_default(),
                    move |value| {
                        Message::ConfigChange(
                            ConfigChange::WindowBasedWorkAreaOffsetLimit(value),
                        )
                    },
                    None,
                ),
                opt_helpers::expandable_with_disable_default(
                    "Work Area Offset",
                    Some("Monitor-specific work area offset (default: global)"),
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            config.work_area_offset.map_or(0, |r| r.left),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetLeft(value)),
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            config.work_area_offset.map_or(0, |r| r.top),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetTop(value)),
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            config.work_area_offset.map_or(0, |r| r.bottom),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetBottom(value)),
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            config.work_area_offset.map_or(0, |r| r.right),
                            move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetRight(value)),
                        ),
                    ],
                    self.work_area_offset_expanded,
                    self.work_area_offset_hovered,
                    Message::ToggleWorkAreaOffsetExpand,
                    Message::ToggleWorkAreaOffsetHover,
                    config.work_area_offset.is_some(),
                    Message::ConfigChange(ConfigChange::WorkAreaOffset(None)),
                    Some(opt_helpers::DisableArgs {
                        disable: config.work_area_offset.is_none(),
                        label: Some("Global"),
                        on_toggle: |v| Message::ConfigChange(ConfigChange::WorkAreaOffset((!v).then_some(Rect::default()))),
                    }),
                ),
                opt_helpers::opt_button(
                    "Workspaces",
                    None,
                    self.workspaces_button_hovered,
                    Message::SetSubScreenWorkspaces,
                    Message::ToggleWorkspacesHover,
                ),
            ]
        )
    }

    pub fn workspaces_view(&self, workspaces: &[WorkspaceConfig]) -> Element<Message> {
        opt_helpers::sub_section_view(
            row![
                nav_button(
                    text!("Monitor [{}] ", self.index),
                    Message::SetSubScreenMonitor
                ),
                text("> Workspaces").size(18)
            ]
            .into(),
            workspaces.iter().enumerate().map(|(i, w)| {
                let title = text!("Workspace [{}] - \"{}\":", i, w.name);
                opt_helpers::opt_button_add_move(
                    title,
                    None,
                    self.workspaces[&i].is_hovered,
                    i > 0,
                    i < workspaces.len() - 1,
                    Message::SetSubScreenWorkspace(i),
                    Message::DeleteWorkspace(i),
                    Message::MoveUpWorkspace(i),
                    Message::MoveDownWorkspace(i),
                    |v| Message::ToggleWorkspaceHover(i, v),
                )
            }),
        )
    }

    pub fn workspace_view<'a>(
        &'a self,
        idx: usize,
        workspace: &'a WorkspaceConfig,
    ) -> Element<'a, Message> {
        opt_helpers::sub_section_view(
            row![
                nav_button(
                    text!("Monitor [{}] ", self.index),
                    Message::SetSubScreenMonitor
                ),
                nav_button(text("> Workspaces"), Message::SetSubScreenWorkspaces),
                text!(" > Workspace [{}] - \"{}\"", idx, workspace.name).size(18),
            ]
            .into(),
            [workspace
                .view(&self.workspaces[&idx])
                .map(move |m| Message::Workspace(idx, m))],
        )
    }

    pub fn workspace_rules_view<'a>(
        &'a self,
        idx: usize,
        workspace: &'a WorkspaceConfig,
    ) -> Element<'a, Message> {
        opt_helpers::sub_section_view(
            row![
                nav_button(
                    text!("Monitor [{}] ", self.index),
                    Message::SetSubScreenMonitor
                ),
                nav_button(text("> Workspaces"), Message::SetSubScreenWorkspaces),
                nav_button(
                    text!(" > Workspace [{}] - \"{}\"", idx, workspace.name),
                    Message::SetSubScreenWorkspace(idx)
                ),
                text("> Workspace Rules").size(18),
            ]
            .into(),
            [workspace
                .view(&self.workspaces[&idx])
                .map(move |m| Message::Workspace(idx, m))],
        )
    }

    pub fn initial_workspace_rules_view<'a>(
        &'a self,
        idx: usize,
        workspace: &'a WorkspaceConfig,
    ) -> Element<'a, Message> {
        opt_helpers::sub_section_view(
            row![
                nav_button(
                    text!("Monitor [{}] ", self.index),
                    Message::SetSubScreenMonitor
                ),
                nav_button(text("> Workspaces"), Message::SetSubScreenWorkspaces),
                nav_button(
                    text!(" > Workspace [{}] - \"{}\"", idx, workspace.name),
                    Message::SetSubScreenWorkspace(idx)
                ),
                text("> Initial Workspace Rules").size(18),
            ]
            .into(),
            [workspace
                .view(&self.workspaces[&idx])
                .map(move |m| Message::Workspace(idx, m))],
        )
    }

    pub fn subscription(&self) -> Subscription<(usize, usize, Message)> {
        match self.sub_screen {
            SubScreen::Monitor | SubScreen::Workspaces | SubScreen::Workspace(_) => {
                Subscription::none()
            }
            SubScreen::WorkspaceRules(ws_idx) | SubScreen::InitialWorkspaceRules(ws_idx) => {
                let workspace = &self.workspaces[&ws_idx];
                workspace
                    .subscription()
                    .with(self.index)
                    .map(|(m_idx, (ws_idx, m))| (m_idx, ws_idx, Message::Workspace(ws_idx, m)))
            }
        }
    }
}

fn nav_button<'a>(
    content: impl Into<iced::widget::Text<'a>>,
    on_press: Message,
) -> iced::widget::Button<'a, Message> {
    button(content.into().size(18))
        .on_press(on_press)
        .padding(0)
        .style(button::text)
}
