use super::workspace::{self, WorkspaceScreen};

use crate::{
    config::{DEFAULT_CONFIG, DEFAULT_MONITOR_CONFIG, DEFAULT_WORKSPACE_CONFIG},
    monitors::TitleLink,
    screen::{
        Modal,
        wallpaper::{self, WallpaperScreen},
    },
    widget::opt_helpers::{self, description_text as t},
};

use std::collections::HashMap;

use iced::{
    Element, Subscription, Task,
    widget::{
        Id,
        operation::{self, AbsoluteOffset},
        span, text,
        text::Span,
    },
};
use komorebi_client::{FloatingLayerBehaviour, MonitorConfig, Rect, Wallpaper, WorkspaceConfig};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEFAULT_MONITOR: Monitor = Default::default();
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    Workspace(usize, workspace::Message),
    Wallpaper(wallpaper::Message),
    SetSubScreenMonitorWallpaper,
    SetSubScreenWorkspaces,
    SetSubScreenWorkspace(usize),
    DeleteWorkspace(usize),
    AddWorkspaceUp(usize),
    AddWorkspaceDown(usize),
    MoveUpWorkspace(usize),
    MoveDownWorkspace(usize),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    ContainerPadding(Option<i32>),
    WorkspacePadding(Option<i32>),
    WindowBasedWorkAreaOffset(Option<Rect>),
    WindowBasedWorkAreaOffsetTop(i32),
    WindowBasedWorkAreaOffsetBottom(i32),
    WindowBasedWorkAreaOffsetRight(i32),
    WindowBasedWorkAreaOffsetLeft(i32),
    WindowBasedWorkAreaOffsetLimit(isize),
    WorkAreaOffset(Option<Rect>),
    WorkAreaOffsetTop(i32),
    WorkAreaOffsetBottom(i32),
    WorkAreaOffsetRight(i32),
    WorkAreaOffsetLeft(i32),
    FloatingLayerBehaviour(Option<FloatingLayerBehaviour>),
    Wallpaper(Option<Wallpaper>),
}

#[derive(Clone, Debug, Default)]
pub enum SubScreen {
    #[default]
    Monitor,
    MonitorWallpaper,
    Workspaces,
    Workspace(usize),
    WorkspaceWallpaper(usize),
    WorkspaceRules(usize),
    InitialWorkspaceRules(usize),
}

pub struct MonitorView<'a, M> {
    pub title: Vec<Span<'a, TitleLink>>,
    pub contents: Vec<Element<'a, M>>,
    pub modal: Option<Modal<'a, M>>,
}

impl<'a, M> MonitorView<'a, M> {
    pub fn new(title: Vec<Span<'a, TitleLink>>, contents: Vec<Element<'a, M>>) -> Self {
        Self {
            title,
            contents,
            modal: None,
        }
    }

    pub fn modal(mut self, element: Option<impl Into<Element<'a, M>>>, close_message: M) -> Self {
        self.modal = Some(Modal::new(element, close_message));
        self
    }

    pub fn map<B>(self, f: impl Fn(M) -> B + Clone + 'a) -> MonitorView<'a, B>
    where
        M: 'a,
        B: 'a,
    {
        MonitorView {
            title: self.title,
            contents: self
                .contents
                .into_iter()
                .map(|el| el.map(f.clone()))
                .collect(),
            modal: self.modal.map(|modal| modal.map(f)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Monitor {
    pub index: usize,
    pub sub_screen: SubScreen,
    pub workspaces: HashMap<usize, workspace::Workspace>,
    pub wallpaper: WallpaperScreen,
}

impl Monitor {
    pub fn update(&mut self, message: Message, config: &mut MonitorConfig) -> Task<Message> {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::ContainerPadding(value) => {
                    config.container_padding = value;
                }
                ConfigChange::WorkspacePadding(value) => {
                    config.workspace_padding = value;
                }
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
                    config.window_based_work_area_offset_limit = Some(limit);
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
                ConfigChange::FloatingLayerBehaviour(value) => {
                    config.floating_layer_behaviour = value
                }
                ConfigChange::Wallpaper(wallpaper) => config.wallpaper = wallpaper,
            },
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
                            workspace::Screen::WorkspaceWallpaper => {
                                self.sub_screen = SubScreen::WorkspaceWallpaper(idx)
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
            Message::Wallpaper(message) => {
                if let Some(wp_config) = config.wallpaper.as_mut() {
                    return self
                        .wallpaper
                        .update(wp_config, message)
                        .map(Message::Wallpaper);
                }
            }
            Message::SetSubScreenWorkspaces => {
                return self.set_subscreen(SubScreen::Workspaces);
            }
            Message::SetSubScreenMonitorWallpaper => {
                return self.set_subscreen(SubScreen::MonitorWallpaper);
            }
            Message::SetSubScreenWorkspace(idx) => {
                return self.set_subscreen(SubScreen::Workspace(idx));
            }
            Message::DeleteWorkspace(idx) => {
                config.workspaces.remove(idx);
                if idx < self.workspaces.len() - 1 {
                    for i in (self.workspaces.len() - 1)..(idx + 1) {
                        if let Some(mut w) = self.workspaces.remove(&i) {
                            w.index = i - 1;
                            self.workspaces.insert(i - 1, w);
                        }
                    }
                } else {
                    self.workspaces.remove(&idx);
                }
            }
            Message::AddWorkspaceUp(idx) => {
                config
                    .workspaces
                    .insert(idx, DEFAULT_WORKSPACE_CONFIG.clone());
                let mut previous_ws = self.workspaces.insert(
                    idx,
                    workspace::Workspace {
                        index: idx,
                        ..Default::default()
                    },
                );
                for i in (idx + 1)..(self.workspaces.len() + 1) {
                    if let Some(mut w) = previous_ws {
                        w.index = i;
                        previous_ws = self.workspaces.insert(i, w)
                    }
                }
            }
            Message::AddWorkspaceDown(idx) => {
                if idx + 1 >= config.workspaces.len() {
                    config.workspaces.push(DEFAULT_WORKSPACE_CONFIG.clone());
                } else {
                    config
                        .workspaces
                        .insert(idx + 1, DEFAULT_WORKSPACE_CONFIG.clone());
                }
                let mut previous_ws = self.workspaces.insert(
                    idx + 1,
                    workspace::Workspace {
                        index: idx + 1,
                        ..Default::default()
                    },
                );
                for i in (idx + 2)..(self.workspaces.len() + 1) {
                    if let Some(mut w) = previous_ws {
                        w.index = i;
                        previous_ws = self.workspaces.insert(i, w)
                    }
                }
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
                }
            }
        }
        Task::none()
    }

    pub fn view<'a>(&'a self, config: &'a MonitorConfig) -> MonitorView<'a, Message> {
        match self.sub_screen {
            SubScreen::Monitor => self.monitor_view(config),
            SubScreen::MonitorWallpaper => self.monitor_wallpaper_view(config.wallpaper.as_ref()),
            SubScreen::Workspaces => self.workspaces_view(&config.workspaces),
            SubScreen::Workspace(idx) | SubScreen::WorkspaceWallpaper(idx) => {
                self.workspace_view(idx, &config.workspaces[idx])
            }
            SubScreen::WorkspaceRules(idx) => {
                self.workspace_rules_view(idx, &config.workspaces[idx])
            }
            SubScreen::InitialWorkspaceRules(idx) => {
                self.initial_workspace_rules_view(idx, &config.workspaces[idx])
            }
        }
    }

    pub fn monitor_view<'a>(&'a self, config: &'a MonitorConfig) -> MonitorView<'a, Message> {
        let title = self.get_sub_section_title(None);
        let contents = vec![
            opt_helpers::number_with_disable_default_option(
                "Container Padding",
                Some("Container padding (default: global)"),
                config.container_padding,
                DEFAULT_MONITOR_CONFIG.container_padding,
                |v| Message::ConfigChange(ConfigChange::ContainerPadding(v)),
                Some(opt_helpers::DisableArgs {
                    disable: config.container_padding.is_none(),
                    label: Some("Global"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::ContainerPadding((!v).then_some(10)))
                    },
                }),
            ),
            opt_helpers::number_with_disable_default_option(
                "Workspace Padding",
                Some("Workspace padding (default: global)"),
                config.workspace_padding,
                DEFAULT_MONITOR_CONFIG.workspace_padding,
                |v| Message::ConfigChange(ConfigChange::WorkspacePadding(v)),
                Some(opt_helpers::DisableArgs {
                    disable: config.workspace_padding.is_none(),
                    label: Some("Global"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::WorkspacePadding((!v).then_some(10)))
                    },
                }),
            ),
            opt_helpers::expandable(
                "Window Based Work Area Offset",
                Some("Window based work area offset (default: global)"),
                || {
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.left),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetLeft(
                                    value,
                                ))
                            },
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.top),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetTop(
                                    value,
                                ))
                            },
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.bottom),
                            move |value| {
                                Message::ConfigChange(
                                    ConfigChange::WindowBasedWorkAreaOffsetBottom(value),
                                )
                            },
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            config.window_based_work_area_offset.map_or(0, |r| r.right),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetRight(
                                    value,
                                ))
                            },
                        ),
                    ]
                },
                config.window_based_work_area_offset.is_some(),
                Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffset(None)),
                Some(opt_helpers::DisableArgs {
                    disable: config.window_based_work_area_offset.is_none(),
                    label: Some("Global"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffset(
                            (!v).then_some(Rect::default()),
                        ))
                    },
                }),
            ),
            opt_helpers::number_with_disable_default(
                "Window Based Work Area Offset Limit",
                Some(
                    "Open window limit after which the window based work area offset will no longer be applied (default: 1)",
                ),
                config.window_based_work_area_offset_limit.unwrap_or(1),
                DEFAULT_MONITOR_CONFIG
                    .window_based_work_area_offset_limit
                    .unwrap_or(1),
                move |value| {
                    Message::ConfigChange(ConfigChange::WindowBasedWorkAreaOffsetLimit(value))
                },
                None,
            ),
            opt_helpers::expandable(
                "Work Area Offset",
                Some("Monitor-specific work area offset (default: global)"),
                || {
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            config.work_area_offset.map_or(0, |r| r.left),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WorkAreaOffsetLeft(value))
                            },
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            config.work_area_offset.map_or(0, |r| r.top),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WorkAreaOffsetTop(value))
                            },
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            config.work_area_offset.map_or(0, |r| r.bottom),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WorkAreaOffsetBottom(value))
                            },
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            config.work_area_offset.map_or(0, |r| r.right),
                            move |value| {
                                Message::ConfigChange(ConfigChange::WorkAreaOffsetRight(value))
                            },
                        ),
                    ]
                },
                config.work_area_offset.is_some(),
                Message::ConfigChange(ConfigChange::WorkAreaOffset(None)),
                Some(opt_helpers::DisableArgs {
                    disable: config.work_area_offset.is_none(),
                    label: Some("Global"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::WorkAreaOffset(
                            (!v).then_some(Rect::default()),
                        ))
                    },
                }),
            ),
            opt_helpers::choose_with_disable_default(
                "Floating Layer Behaviour",
                Some("Determines what happens to a new window when on the `FloatingLayer` (default: global)"),
                vec![
                    t("Selected: 'Tile' -> Tile new windows (unless they match a float rule or float override is active)").into(),
                    t("Selected: 'Float' -> Float new windows").into(),
                ],
                [FloatingLayerBehaviour::Tile, FloatingLayerBehaviour::Float],
                config.floating_layer_behaviour.or(DEFAULT_MONITOR_CONFIG.floating_layer_behaviour),
                |v| Message::ConfigChange(ConfigChange::FloatingLayerBehaviour(v)),
                DEFAULT_MONITOR_CONFIG.floating_layer_behaviour,
                Some(opt_helpers::DisableArgs {
                    disable: config.floating_layer_behaviour.is_none(),
                    label: Some("Global"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::FloatingLayerBehaviour(
                            (!v).then(|| DEFAULT_CONFIG.floating_layer_behaviour).flatten()
                        ))
                    },
                }),
            ),
            opt_helpers::opt_button_disable_default(
                "Wallpaper",
                Some("Specify a wallpaper for this monitor. (default: None)"),
                Message::SetSubScreenMonitorWallpaper,
                config.wallpaper.is_some(),
                Some(Message::ConfigChange(ConfigChange::Wallpaper(None))),
                Some(opt_helpers::DisableArgs {
                    disable: config.wallpaper.is_none(),
                    label: Some("None"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::Wallpaper(
                            (!v).then(|| wallpaper::DEFAULT_WALLPAPER.clone()),
                        ))
                    },
                }),
            ),
            opt_helpers::opt_button("Workspaces", None, Message::SetSubScreenWorkspaces),
        ];

        MonitorView::new(title, contents)
    }

    pub fn monitor_wallpaper_view<'a>(
        &'a self,
        wp_config: Option<&'a Wallpaper>,
    ) -> MonitorView<'a, Message> {
        let title = self.get_sub_section_title(None);
        if let Some(wp_config) = wp_config {
            let wp = self.wallpaper.view(wp_config).map(Message::Wallpaper);
            if let Some(modal) = wp.modal {
                MonitorView::new(title, vec![wp.element]).modal(modal.element, modal.close_message)
            } else {
                MonitorView::new(title, vec![wp.element])
            }
        } else {
            MonitorView::new(title, vec![])
        }
    }

    pub fn workspaces_view(&self, workspaces: &[WorkspaceConfig]) -> MonitorView<'_, Message> {
        let title = self.get_sub_section_title(None);
        let contents = workspaces
            .iter()
            .enumerate()
            .map(|(i, w)| {
                opt_helpers::opt_button_add_move(
                    format!("Workspace [{}] - \"{}\":", i, w.name),
                    None,
                    workspaces.len() > 1,
                    i > 0,
                    i < workspaces.len() - 1,
                    Message::SetSubScreenWorkspace(i),
                    Message::DeleteWorkspace(i),
                    Message::AddWorkspaceUp(i),
                    Message::AddWorkspaceDown(i),
                    Message::MoveUpWorkspace(i),
                    Message::MoveDownWorkspace(i),
                )
            })
            .collect();

        MonitorView::new(title, contents)
    }

    pub fn workspace_view<'a>(
        &'a self,
        idx: usize,
        workspace: &'a WorkspaceConfig,
    ) -> MonitorView<'a, Message> {
        let title = self.get_sub_section_title(Some(workspace));
        let ws_view = workspace
            .view(&self.workspaces[&idx])
            .map(move |m| Message::Workspace(idx, m));
        let contents = vec![ws_view.element];

        if let Some(modal) = ws_view.modal {
            MonitorView::new(title, contents).modal(modal.element, modal.close_message)
        } else {
            MonitorView::new(title, contents)
        }
    }

    pub fn workspace_rules_view<'a>(
        &'a self,
        idx: usize,
        workspace: &'a WorkspaceConfig,
    ) -> MonitorView<'a, Message> {
        let title = self.get_sub_section_title(Some(workspace));
        let contents = vec![
            workspace
                .view(&self.workspaces[&idx])
                .map(move |m| Message::Workspace(idx, m))
                .element,
        ];

        MonitorView::new(title, contents)
    }

    pub fn initial_workspace_rules_view<'a>(
        &'a self,
        idx: usize,
        workspace: &'a WorkspaceConfig,
    ) -> MonitorView<'a, Message> {
        let title = self.get_sub_section_title(Some(workspace));
        let contents = vec![
            workspace
                .view(&self.workspaces[&idx])
                .map(move |m| Message::Workspace(idx, m))
                .element,
        ];

        MonitorView::new(title, contents)
    }

    pub fn subscription(&self) -> Subscription<(usize, usize, Message)> {
        match self.sub_screen {
            SubScreen::Monitor
            | SubScreen::MonitorWallpaper
            | SubScreen::Workspaces
            | SubScreen::Workspace(_)
            | SubScreen::WorkspaceWallpaper(_) => Subscription::none(),
            SubScreen::WorkspaceRules(ws_idx) | SubScreen::InitialWorkspaceRules(ws_idx) => {
                let workspace = &self.workspaces[&ws_idx];
                workspace
                    .subscription()
                    .with(self.index)
                    .map(|(m_idx, (ws_idx, m))| (m_idx, ws_idx, Message::Workspace(ws_idx, m)))
            }
        }
    }

    fn get_sub_section_title(
        &self,
        workspace: Option<&WorkspaceConfig>,
    ) -> Vec<Span<'_, TitleLink>> {
        match self.sub_screen {
            SubScreen::Monitor => vec![span(format!("Monitor [{}]:", self.index))],
            SubScreen::MonitorWallpaper => vec![
                nav_button(
                    format!("Monitor [{}]", self.index),
                    TitleLink::Monitor(self.index, SubScreen::Monitor),
                ),
                span(" > "),
                span("Wallpaper"),
            ],
            SubScreen::Workspaces => vec![
                nav_button(
                    format!("Monitor [{}]", self.index),
                    TitleLink::Monitor(self.index, SubScreen::Monitor),
                ),
                span(" > "),
                span("Workspaces"),
            ],
            SubScreen::Workspace(idx) => vec![
                nav_button(
                    format!("Monitor [{}]", self.index),
                    TitleLink::Monitor(self.index, SubScreen::Monitor),
                ),
                span(" > "),
                nav_button(
                    "Workspaces",
                    TitleLink::Monitor(self.index, SubScreen::Workspaces),
                ),
                span(" > "),
                span(format!(
                    "Workspace [{}] - \"{}\"",
                    idx,
                    workspace.unwrap().name
                )),
            ],
            SubScreen::WorkspaceWallpaper(idx) => vec![
                nav_button(
                    format!("Monitor [{}]", self.index),
                    TitleLink::Monitor(self.index, SubScreen::Monitor),
                ),
                span(" > "),
                nav_button(
                    "Workspaces",
                    TitleLink::Monitor(self.index, SubScreen::Workspaces),
                ),
                span(" > "),
                nav_button(
                    format!("Workspace [{}] - \"{}\"", idx, workspace.unwrap().name),
                    TitleLink::Monitor(self.index, SubScreen::Workspace(idx)),
                ),
                span(" > Wallpaper"),
            ],
            SubScreen::WorkspaceRules(idx) => vec![
                nav_button(
                    format!("Monitor [{}]", self.index),
                    TitleLink::Monitor(self.index, SubScreen::Monitor),
                ),
                span(" > "),
                nav_button(
                    "Workspaces",
                    TitleLink::Monitor(self.index, SubScreen::Workspaces),
                ),
                span(" > "),
                nav_button(
                    format!("Workspace [{}] - \"{}\"", idx, workspace.unwrap().name),
                    TitleLink::Monitor(self.index, SubScreen::Workspace(idx)),
                ),
                span(" > "),
                span("Workspace Rules"),
            ],
            SubScreen::InitialWorkspaceRules(idx) => vec![
                nav_button(
                    format!("Monitor [{}]", self.index),
                    TitleLink::Monitor(self.index, SubScreen::Monitor),
                ),
                span(" > "),
                nav_button(
                    "Workspaces",
                    TitleLink::Monitor(self.index, SubScreen::Workspaces),
                ),
                span(" > "),
                nav_button(
                    format!("Workspace [{}] - \"{}\"", idx, workspace.unwrap().name),
                    TitleLink::Monitor(self.index, SubScreen::Workspace(idx)),
                ),
                span(" > "),
                span("Initial Workspace Rules"),
            ],
        }
    }

    pub fn set_subscreen(&mut self, sub_screen: SubScreen) -> Task<Message> {
        if let SubScreen::Workspace(idx) = &sub_screen {
            self.workspaces.entry(*idx).or_default().screen = workspace::Screen::Workspace;
        }
        self.sub_screen = sub_screen;
        operation::scroll_to(
            Id::new("monitors_scrollable"),
            AbsoluteOffset { x: 0.0, y: 0.0 },
        )
    }
}

fn nav_button<'a>(content: impl text::IntoFragment<'a>, link: TitleLink) -> Span<'a, TitleLink> {
    span(content).link(link)
}
