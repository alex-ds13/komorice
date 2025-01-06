use crate::widget::opt_helpers::description_text as t;
use crate::{
    config::DEFAULT_CONFIG,
    utils::DisplayOption,
    widget::opt_helpers::{self, DisableArgs},
};

use std::path::PathBuf;

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
    CrossBoundaryBehaviour(Option<CrossBoundaryBehaviour>),
    CrossMonitorMoveBehaviour(Option<MoveBehaviour>),
    DefaultContainerPadding(Option<i32>),
    DefaultWorkspacePadding(Option<i32>),
    FloatOverride(Option<bool>),
    FocusFollowsMouse(Option<FocusFollowsMouseImplementation>),
    GlobalWorkAreaOffset(Option<Rect>),
    GlobalWorkAreaOffsetTop(i32),
    GlobalWorkAreaOffsetBottom(i32),
    GlobalWorkAreaOffsetRight(i32),
    GlobalWorkAreaOffsetLeft(i32),
    MouseFollowsFocus(Option<bool>),
    ResizeDelta(Option<i32>),
    SlowApplicationCompensationTime(Option<i32>),
    UnmanagedWindowBehaviour(Option<OperationBehaviour>),
    WindowContainerBehaviour(Option<WindowContainerBehaviour>),
    WindowHidingBehaviour(Option<HidingBehaviour>),
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
                    config.cross_boundary_behaviour = value;
                }
                ConfigChange::CrossMonitorMoveBehaviour(value) => {
                    config.cross_monitor_move_behaviour = value;
                }
                ConfigChange::DefaultContainerPadding(value) => {
                    config.default_container_padding = value;
                }
                ConfigChange::DefaultWorkspacePadding(value) => {
                    config.default_workspace_padding = value;
                }
                ConfigChange::FloatOverride(value) => {
                    config.float_override = value;
                }
                ConfigChange::FocusFollowsMouse(value) => {
                    config.focus_follows_mouse = value;
                }
                ConfigChange::GlobalWorkAreaOffset(value) => {
                    config.global_work_area_offset = value;
                }
                ConfigChange::GlobalWorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut config.global_work_area_offset {
                        offset.top = value;
                    } else {
                        config.global_work_area_offset = Some(Rect {
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
                        config.global_work_area_offset = Some(Rect {
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
                        config.global_work_area_offset = Some(Rect {
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
                        config.global_work_area_offset = Some(Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::MouseFollowsFocus(value) => {
                    config.mouse_follows_focus = value;
                }
                ConfigChange::ResizeDelta(value) => {
                    config.resize_delta = value;
                }
                ConfigChange::SlowApplicationCompensationTime(value) => {
                    let value = value.and_then(|v| {
                        v.try_into()
                            .ok()
                            .or(config.slow_application_compensation_time)
                    });
                    config.slow_application_compensation_time = value;
                }
                ConfigChange::UnmanagedWindowBehaviour(value) => {
                    config.unmanaged_window_operation_behaviour = value;
                }
                ConfigChange::WindowContainerBehaviour(value) => {
                    config.window_container_behaviour = value;
                }
                ConfigChange::WindowHidingBehaviour(value) => {
                    config.window_hiding_behaviour = value;
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
                opt_helpers::input_with_disable_default(
                    "App Specific Configuration Path",
                    Some("Path to applications.json from komorebi-application-specific-configurations (default: None)"),
                    "",
                    config.app_specific_configuration_path.as_ref().map_or("", |p| p.to_str().unwrap_or_default()),
                    DEFAULT_CONFIG.app_specific_configuration_path.as_ref().map_or("".into(), |p| p.display().to_string()),
                    |value| Message::ConfigChange(ConfigChange::AppSpecificConfigurationPath(Some(PathBuf::from(value)))),
                    None,
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Cross Boundary Behaviour",
                    Some("Determine what happens when an action is called on a window at a monitor boundary (default: Monitor)"),
                    vec![
                        t("Selected: 'Monitor' -> Attempt to perform actions across a monitor boundary").into(),
                        t("Selected: 'Workspace' -> Attempt to perform actions across a workspace boundary").into(),
                    ],
                    [CrossBoundaryBehaviour::Monitor, CrossBoundaryBehaviour::Workspace],
                    config.cross_boundary_behaviour.or(DEFAULT_CONFIG.cross_boundary_behaviour),
                    |selected| Message::ConfigChange(ConfigChange::CrossBoundaryBehaviour(selected)),
                    DEFAULT_CONFIG.cross_boundary_behaviour,
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Cross Monitor Move Behaviour",
                    Some("Determine what happens when a window is moved across a monitor boundary (default: Swap)"),
                    vec![
                        t("Selected: 'Swap' -> Swap the window container with the window container at the edge of the adjacent monitor").into(),
                        t("Selected: 'Insert' -> Insert the window container into the focused workspace on the adjacent monitor").into(),
                        t("Selected: 'NoOp' -> Do nothing if trying to move a window container in the direction of an adjacent monitor").into(),
                    ],
                    [MoveBehaviour::Swap, MoveBehaviour::Insert, MoveBehaviour::NoOp],
                    config.cross_monitor_move_behaviour.or(DEFAULT_CONFIG.cross_monitor_move_behaviour),
                    |selected| Message::ConfigChange(ConfigChange::CrossMonitorMoveBehaviour(selected)),
                    DEFAULT_CONFIG.cross_monitor_move_behaviour,
                    None,
                ),
                opt_helpers::number_with_disable_default_option(
                    "Default Container Padding",
                    Some("Global default container padding (default: 10)"),
                    config.default_container_padding.or(DEFAULT_CONFIG.default_container_padding),
                    DEFAULT_CONFIG.default_container_padding,
                    |value| Message::ConfigChange(ConfigChange::DefaultContainerPadding(value)),
                    None,
                ),
                opt_helpers::number_with_disable_default_option(
                    "Default Workspace Padding",
                    Some("Global default workspace padding (default: 10)"),
                    config.default_workspace_padding.or(DEFAULT_CONFIG.default_workspace_padding),
                    DEFAULT_CONFIG.default_workspace_padding,
                    |value| Message::ConfigChange(ConfigChange::DefaultWorkspacePadding(value)),
                    None,
                ),
                opt_helpers::toggle_with_disable_default(
                    "Float Override",
                    Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: false)"),
                    config.float_override.or(DEFAULT_CONFIG.float_override),
                    DEFAULT_CONFIG.float_override,
                    |value| Message::ConfigChange(ConfigChange::FloatOverride(value)),
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Focus Follows Mouse",
                    Some("END OF LIFE FEATURE: Determine focus follows mouse implementation (default: None)\n\
                    Use 'https://github.com/LGUG2Z/masir' instead"),
                    vec![
                        t("Selected: '[None]' -> No focus follows mouse is performed").into(),
                        t("Selected: 'Komorebi' -> A custom FFM implementation (slightly more CPU-intensive)").into(),
                        t("Selected: 'Windows' -> The native (legacy) Windows FFM implementation").into(),
                    ],
                    &FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS[..],
                    Some(DisplayOption(config.focus_follows_mouse)),
                    |selected| Message::ConfigChange(ConfigChange::FocusFollowsMouse(selected.and_then(|v| v.0))),
                    Some(DisplayOption(DEFAULT_CONFIG.focus_follows_mouse)),
                    None,
                ),
                opt_helpers::expandable_with_disable_default(
                    "Global Work Area Offset",
                    Some("Global work area (space used for tiling) offset (default: None)"),
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
                    config.global_work_area_offset.is_some(),
                    Message::ConfigChange(ConfigChange::GlobalWorkAreaOffset(None)),
                    Some(DisableArgs {
                        disable: config.global_work_area_offset.is_none(),
                        label: Some("None"),
                        on_toggle: |v| Message::ConfigChange(ConfigChange::GlobalWorkAreaOffset((!v).then_some(Rect::default()))),
                    }),
                ),
                opt_helpers::toggle_with_disable_default(
                    "Mouse Follows Focus",
                    Some("Enable or disable mouse follows focus (default: true)"),
                    config.mouse_follows_focus.or(DEFAULT_CONFIG.mouse_follows_focus),
                    DEFAULT_CONFIG.mouse_follows_focus,
                    |value| Message::ConfigChange(ConfigChange::MouseFollowsFocus(value)),
                    None,
                ),
                opt_helpers::number_with_disable_default_option(
                    "Resize Delta",
                    Some("Delta to resize windows by (default 50)"),
                    config.resize_delta.or(DEFAULT_CONFIG.resize_delta),
                    DEFAULT_CONFIG.resize_delta,
                    |value| Message::ConfigChange(ConfigChange::ResizeDelta(value)),
                    None,
                ),
                opt_helpers::number_with_disable_default_option(
                    "Slow Application Compensation Time",
                    Some("How long to wait when compensating for slow applications, \
                    in milliseconds (default: 20)\n\n\
                    Value must be greater or equal to 0."
                    ),
                    (config.slow_application_compensation_time.or(DEFAULT_CONFIG.slow_application_compensation_time)).and_then(|v| v.try_into().ok()),
                    DEFAULT_CONFIG.slow_application_compensation_time.and_then(|v| v.try_into().ok()),
                    |value| Message::ConfigChange(ConfigChange::SlowApplicationCompensationTime(value)),
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Unmanaged Window Behaviour",
                    Some("Determine what happens when commands are sent while an unmanaged window is in the foreground (default: Op)"),
                    vec![
                        t("Selected: 'Op' -> Process komorebic commands on temporarily unmanaged/floated windows").into(),
                        t("Selected: 'NoOp' -> Ignore komorebic commands on temporarily unmanaged/floated windows").into(),
                    ],
                    [OperationBehaviour::Op, OperationBehaviour::NoOp],
                    config.unmanaged_window_operation_behaviour.or(DEFAULT_CONFIG.unmanaged_window_operation_behaviour),
                    |selected| Message::ConfigChange(ConfigChange::UnmanagedWindowBehaviour(selected)),
                    DEFAULT_CONFIG.unmanaged_window_operation_behaviour,
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Window Container Behaviour",
                    Some("Determine what happens when a new window is opened (default: Create)"),
                    vec![
                        t("Selected: 'Create' -> Create a new container for each new window").into(),
                        t("Selected: 'Append' -> Append new windows to the focused window container").into(),
                    ],
                    [WindowContainerBehaviour::Create, WindowContainerBehaviour::Append],
                    config.window_container_behaviour.or(DEFAULT_CONFIG.window_container_behaviour),
                    |selected| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(selected)),
                    DEFAULT_CONFIG.window_container_behaviour,
                    None,
                ),
                opt_helpers::choose_with_disable_default(
                    "Window Hiding Behaviour",
                    Some("Which Windows signal to use when hiding windows (default: Cloak)"),
                    vec![
                        t("Selected: 'Cloak' -> Use the undocumented SetCloak Win32 function to hide windows when switching workspaces").into(),
                        t("Selected: 'Hide' -> Use the SW_HIDE flag to hide windows when switching workspaces (has issues with Electron apps)").into(),
                        t("Selected: 'Minimize' -> Use the SW_MINIMIZE flag to hide windows when switching workspaces (has issues with frequent workspace switching)").into(),
                    ],
                    [HidingBehaviour::Cloak, HidingBehaviour::Hide, HidingBehaviour::Minimize],
                    config.window_hiding_behaviour.or(DEFAULT_CONFIG.window_hiding_behaviour),
                    |selected| Message::ConfigChange(ConfigChange::WindowHidingBehaviour(selected)),
                    DEFAULT_CONFIG.window_hiding_behaviour,
                    None,
                ),
            ],
        )
    }
}
