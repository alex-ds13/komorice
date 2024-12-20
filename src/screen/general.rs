use crate::{utils::DisplayOption, widget::opt_helpers};

use std::{collections::HashMap, path::PathBuf};

use iced::{Element, Task};
use komorebi_client::{
    CrossBoundaryBehaviour, FocusFollowsMouseImplementation, HidingBehaviour, MoveBehaviour,
    OperationBehaviour, Rect, StaticConfig, WindowContainerBehaviour,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS: [DisplayOption<FocusFollowsMouseImplementation>; 3] = [
        DisplayOption(None),
        DisplayOption(Some(FocusFollowsMouseImplementation::Windows)),
        DisplayOption(Some(FocusFollowsMouseImplementation::Komorebi)),
    ];
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleGlobalWorkAreaOffsetExpand,
    ToggleGlobalWorkAreaOffsetHover(bool),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    AppSpecificConfigurationPath(Option<PathBuf>),
    CrossBoundaryBehaviour(CrossBoundaryBehaviour),
    CrossMonitorMoveBehaviour(MoveBehaviour),
    DefaultContainerPadding(i32),
    DefaultWorkspacePadding(i32),
    DisplayIndexPreferences(HashMap<usize, String>),
    FloatOverride(bool),
    FocusFollowsMouse(Option<FocusFollowsMouseImplementation>),
    GlobalWorkAreaOffset(Rect),
    GlobalWorkAreaOffsetTop(i32),
    GlobalWorkAreaOffsetBottom(i32),
    GlobalWorkAreaOffsetRight(i32),
    GlobalWorkAreaOffsetLeft(i32),
    MouseFollowsFocus(bool),
    ResizeDelta(i32),
    SlowApplicationCompensationTime(i32),
    UnmanagedWindowBehaviour(OperationBehaviour),
    WindowContainerBehaviour(WindowContainerBehaviour),
    WindowHidingBehaviour(HidingBehaviour),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct General {
    pub global_work_area_offset_expanded: bool,
    pub global_work_area_offset_hovered: bool,
}

impl General {
    pub fn update(
        &mut self,
        message: Message,
        config: &mut StaticConfig,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::AppSpecificConfigurationPath(path) => {
                    config.app_specific_configuration_path = path;
                }
                ConfigChange::CrossBoundaryBehaviour(value) => {
                    config.cross_boundary_behaviour = Some(value);
                }
                ConfigChange::CrossMonitorMoveBehaviour(value) => {
                    config.cross_monitor_move_behaviour = Some(value);
                }
                ConfigChange::DefaultContainerPadding(value) => {
                    config.default_container_padding = Some(value);
                }
                ConfigChange::DefaultWorkspacePadding(value) => {
                    config.default_workspace_padding = Some(value);
                }
                ConfigChange::DisplayIndexPreferences(value) => {
                    config.display_index_preferences = Some(value);
                }
                ConfigChange::FloatOverride(value) => {
                    config.float_override = Some(value);
                }
                ConfigChange::FocusFollowsMouse(value) => {
                    config.focus_follows_mouse = value;
                }
                ConfigChange::GlobalWorkAreaOffset(value) => {
                    config.global_work_area_offset = Some(value);
                }
                ConfigChange::GlobalWorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut config.global_work_area_offset {
                        offset.top = value;
                    } else {
                        config.global_work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: value,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::GlobalWorkAreaOffsetBottom(value) => {
                    if let Some(offset) = &mut config.global_work_area_offset {
                        offset.bottom = value;
                    } else {
                        config.global_work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: value,
                        });
                    }
                }
                ConfigChange::GlobalWorkAreaOffsetRight(value) => {
                    if let Some(offset) = &mut config.global_work_area_offset {
                        offset.right = value;
                    } else {
                        config.global_work_area_offset = Some(komorebi::Rect {
                            left: 0,
                            top: 0,
                            right: value,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::GlobalWorkAreaOffsetLeft(value) => {
                    if let Some(offset) = &mut config.global_work_area_offset {
                        offset.left = value;
                    } else {
                        config.global_work_area_offset = Some(komorebi::Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::MouseFollowsFocus(value) => {
                    config.mouse_follows_focus = Some(value);
                }
                ConfigChange::ResizeDelta(value) => {
                    config.resize_delta = Some(value);
                }
                ConfigChange::SlowApplicationCompensationTime(value) => {
                    if let Ok(value) = value.try_into() {
                        config.slow_application_compensation_time = Some(value);
                    }
                }
                ConfigChange::UnmanagedWindowBehaviour(value) => {
                    config.unmanaged_window_operation_behaviour = Some(value);
                }
                ConfigChange::WindowContainerBehaviour(value) => {
                    config.window_container_behaviour = Some(value);
                }
                ConfigChange::WindowHidingBehaviour(value) => {
                    config.window_hiding_behaviour = Some(value);
                }
            },
            Message::ToggleGlobalWorkAreaOffsetExpand => {
                self.global_work_area_offset_expanded = !self.global_work_area_offset_expanded;
            }
            Message::ToggleGlobalWorkAreaOffsetHover(hover) => {
                self.global_work_area_offset_hovered = hover;
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: &'a StaticConfig) -> Element<'a, Message> {
        opt_helpers::section_view(
            "General:",
            [
                opt_helpers::input(
                    "App Specific Configuration Path",
                    Some("Path to applications.json from komorebi-application-specific-configurations (default: None)"),
                    "",
                    config.app_specific_configuration_path.as_ref().map_or("", |p| p.to_str().unwrap_or_default()),
                    |value| Message::ConfigChange(ConfigChange::AppSpecificConfigurationPath(Some(PathBuf::from(value)))),
                    None
                ),
                opt_helpers::choose(
                    "Cross Boundary Behaviour",
                    Some("Determine what happens when an action is called on a window at a monitor boundary (default: Monitor)"),
                    [CrossBoundaryBehaviour::Monitor, CrossBoundaryBehaviour::Workspace],
                    config.cross_boundary_behaviour,
                    |selected| Message::ConfigChange(ConfigChange::CrossBoundaryBehaviour(selected)),
                ),
                opt_helpers::choose(
                    "Cross Monitor Move Behaviour",
                    Some("Determine what happens when a window is moved across a monitor boundary (default: Swap)"),
                    [MoveBehaviour::Swap, MoveBehaviour::Insert, MoveBehaviour::NoOp],
                    config.cross_monitor_move_behaviour,
                    |selected| Message::ConfigChange(ConfigChange::CrossMonitorMoveBehaviour(selected)),
                ),
                opt_helpers::number(
                    "Default Container Padding",
                    Some("Global default container padding (default: 10)"),
                    config.default_container_padding.unwrap_or(10),
                    |value| Message::ConfigChange(ConfigChange::DefaultContainerPadding(value)),
                ),
                opt_helpers::number(
                    "Default Workspace Padding",
                    Some("Global default workspace padding (default: 10)"),
                    config.default_workspace_padding.unwrap_or(10),
                    |value| Message::ConfigChange(ConfigChange::DefaultWorkspacePadding(value)),
                ),
                opt_helpers::toggle(
                    "Float Override",
                    Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: false)"),
                    config.float_override.unwrap_or_default(),
                    |value| Message::ConfigChange(ConfigChange::FloatOverride(value))
                ),
                opt_helpers::choose(
                    "Focus Follows Mouse",
                    Some("END OF LIFE FEATURE: Determine focus follows mouse implementation (default: None)"),
                    &FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS[..],
                    Some(DisplayOption(config.focus_follows_mouse)),
                    |selected| Message::ConfigChange(ConfigChange::FocusFollowsMouse(selected.0)),
                ),
                opt_helpers::expandable(
                    "Global Work Area Offset",
                    None,
                    [
                        opt_helpers::number(
                            "left",
                            None,
                            config.global_work_area_offset.map_or(0, |r| r.left),
                            |value| Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetLeft(value)),
                        ),
                        opt_helpers::number(
                            "top",
                            None,
                            config.global_work_area_offset.map_or(0, |r| r.top),
                            |value| Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetTop(value)),
                        ),
                        opt_helpers::number(
                            "bottom",
                            None,
                            config.global_work_area_offset.map_or(0, |r| r.bottom),
                            |value| Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetBottom(value)),
                        ),
                        opt_helpers::number(
                            "right",
                            None,
                            config.global_work_area_offset.map_or(0, |r| r.right),
                            |value| Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetRight(value)),
                        ),
                    ],
                    self.global_work_area_offset_expanded,
                    self.global_work_area_offset_hovered,
                    Message::ToggleGlobalWorkAreaOffsetExpand,
                    Message::ToggleGlobalWorkAreaOffsetHover,
                ),
                opt_helpers::toggle(
                    "Mouse Follows Focus",
                    Some("Enable or disable mouse follows focus (default: true)"),
                    config.mouse_follows_focus.unwrap_or(true),
                    |value| Message::ConfigChange(ConfigChange::MouseFollowsFocus(value))
                ),
                opt_helpers::number(
                    "Resize Delta",
                    Some("Delta to resize windows by (default 50)"),
                    config.resize_delta.unwrap_or(50),
                    |value| Message::ConfigChange(ConfigChange::ResizeDelta(value)),
                ),
                opt_helpers::number(
                    "Slow Application Compensation Time",
                    Some("How long to wait when compensating for slow applications, \
                    in milliseconds (default: 20)\n\n\
                    Value must be greater or equal to 0."
                    ),
                    config.slow_application_compensation_time.and_then(|v| v.try_into().ok()).unwrap_or(20),
                    |value| Message::ConfigChange(ConfigChange::SlowApplicationCompensationTime(value)),
                ),
                opt_helpers::choose(
                    "Unmanaged Window Behaviour",
                    Some("Determine what happens when commands are sent while an unmanaged window is in the foreground (default: Op)"),
                    [OperationBehaviour::Op, OperationBehaviour::NoOp],
                    Some(config.unmanaged_window_operation_behaviour.unwrap_or(OperationBehaviour::Op)),
                    |selected| Message::ConfigChange(ConfigChange::UnmanagedWindowBehaviour(selected)),
                ),
                opt_helpers::choose(
                    "Window Container Behaviour",
                    Some("Determine what happens when a new window is opened (default: Create)"),
                    [WindowContainerBehaviour::Create, WindowContainerBehaviour::Append],
                    Some(config.window_container_behaviour.unwrap_or(WindowContainerBehaviour::Create)),
                    |selected| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(selected)),
                ),
                opt_helpers::choose(
                    "Window Hiding Behaviour",
                    Some("Which Windows signal to use when hiding windows (default: Cloak)"),
                    [HidingBehaviour::Cloak, HidingBehaviour::Hide, HidingBehaviour::Minimize],
                    config.window_hiding_behaviour,
                    |selected| Message::ConfigChange(ConfigChange::WindowHidingBehaviour(selected)),
                ),
            ],
        )
    }
}
