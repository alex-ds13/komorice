use std::{path::PathBuf, sync::Arc};

use crate::{
    config::{ConfigHelpersAction, GlobalConfigChangeType, MonitorConfigChangeType},
    widget::opt_helpers,
    Komofig, Message, NONE_STR,
};

use iced::{
    widget::{text, Space},
    Element,
    Length::Shrink,
};
use komorebi::{
    CrossBoundaryBehaviour, FocusFollowsMouseImplementation, HidingBehaviour, MoveBehaviour,
    WindowContainerBehaviour,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref CROSS_BOUNDARY_BEHAVIOUR_OPTIONS: [Arc<str>; 2] = [
        Arc::from(CrossBoundaryBehaviour::Monitor.to_string()),
        Arc::from(CrossBoundaryBehaviour::Workspace.to_string()),
    ];
    static ref FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS: [Arc<str>; 3] = [
        Arc::clone(&NONE_STR),
        Arc::from(FocusFollowsMouseImplementation::Windows.to_string()),
        Arc::from(FocusFollowsMouseImplementation::Komorebi.to_string()),
    ];
    static ref FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS1: [DisplayOption<FocusFollowsMouseImplementation>; 3] = [
        DisplayOption(None),
        DisplayOption(Some(FocusFollowsMouseImplementation::Windows)),
        DisplayOption(Some(FocusFollowsMouseImplementation::Komorebi)),
    ];
    static ref HIDING_BEHAVIOUR_OPTIONS: [Arc<str>; 3] = [
        Arc::from(HidingBehaviour::Cloak.to_string()),
        Arc::from(HidingBehaviour::Hide.to_string()),
        Arc::from(HidingBehaviour::Minimize.to_string()),
    ];
}

#[derive(Clone, Debug, PartialEq)]
struct DisplayOption<T>(pub Option<T>);

impl<T: std::fmt::Display> std::fmt::Display for DisplayOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            Some(ref v) => write!(f, "{}", v),
            None => write!(f, "{}", *NONE_STR),
        }
    }
}

pub fn view(app: &Komofig) -> Element<Message> {
    view_general(app)
}

fn view_general(app: &Komofig) -> Element<Message> {
    if let Some(config) = &app.config {
        opt_helpers::section_view(
            "General:",
            [
                opt_helpers::input(
                    "App Specific Configuration Path",
                    Some("Path to applications.json from komorebi-application-specific-configurations (default: None)"),
                    "",
                    config.app_specific_configuration_path.as_ref().map_or("", |p| p.to_str().unwrap_or_default()),
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::AppSpecificConfigurationPath(Some(PathBuf::from(value)))),
                    None
                ),
                opt_helpers::choose(
                    "Cross Boundary Behaviour",
                    Some("Determine what happens when an action is called on a window at a monitor boundary (default: Monitor)"),
                    &CROSS_BOUNDARY_BEHAVIOUR_OPTIONS[..],
                    Some(&app.config_strs.as_ref().unwrap().global_config_strs.cross_boundary_behaviour),
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::CrossBoundaryBehaviour(selected)),
                ),
                opt_helpers::choose(
                    "Cross Monitor Move Behaviour",
                    Some("Determine what happens when a window is moved across a monitor boundary (default: Swap)"),
                    [MoveBehaviour::Swap, MoveBehaviour::Insert, MoveBehaviour::NoOp],
                    config.cross_monitor_move_behaviour,
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::CrossMonitorMoveBehaviour(selected)),
                ),
                opt_helpers::input(
                    "Default Container Padding",
                    Some("Global default container padding (default: 10)"),
                    "",
                    &app.config_strs.as_ref().unwrap().global_config_strs.default_container_padding,
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::DefaultContainerPadding(value)),
                    None
                ),
                opt_helpers::input(
                    "Default Workspace Padding",
                    Some("Global default workspace padding (default: 10)"),
                    "",
                    &app.config_strs.as_ref().unwrap().global_config_strs.default_workspace_padding,
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::DefaultWorkspacePadding(value)),
                    None
                ),
                opt_helpers::toggle(
                    "Float Override",
                    Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: false)"),
                    config.float_override.unwrap_or_default(),
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::FloatOverride(value))
                ),
                opt_helpers::choose(
                    "Focus Follows Mouse",
                    Some("END OF LIFE FEATURE: Determine focus follows mouse implementation (default: None)"),
                    &FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS[..],
                    Some(&app.config_strs.as_ref().unwrap().global_config_strs.focus_follows_mouse),
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::FocusFollowsMouse(selected)),
                ),
                opt_helpers::choose(
                    "Focus Follows Mouse",
                    Some("END OF LIFE FEATURE: Determine focus follows mouse implementation (default: None)"),
                    &FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS1[..],
                    Some(DisplayOption(config.focus_follows_mouse)),
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::FocusFollowsMouse1(selected.0)),
                ),
                opt_helpers::expandable(
                    "Global Work Area Offset",
                    None,
                    [
                        opt_helpers::input(
                            "left",
                            None,
                            "",
                            &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_left,
                            Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetLeft(value))),
                            None,
                        ),
                        opt_helpers::input(
                            "top",
                            None,
                            "",
                            &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_top,
                            Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetTop(value))),
                            None,
                        ),
                        opt_helpers::input(
                            "bottom",
                            None,
                            "",
                            &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_bottom,
                            Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetBottom(value))),
                            None,
                        ),
                        opt_helpers::input(
                            "right",
                            None,
                            "",
                            &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_right,
                            Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetRight(value))),
                            None,
                        ),
                    ],
                    app.config_helpers.global_work_area_offset_expanded,
                    Message::ConfigHelpers(ConfigHelpersAction::ToggleGlobalWorkAreaOffsetExpand)
                ),
                opt_helpers::toggle(
                    "Mouse Follows Focus",
                    Some("Enable or disable mouse follows focus (default: true)"),
                    config.mouse_follows_focus.unwrap_or(true),
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::MouseFollowsFocus(value))
                ),
                opt_helpers::input(
                    "Resize Delta",
                    Some("Delta to resize windows by (default 50)"),
                    "",
                    &app.config_strs.as_ref().unwrap().global_config_strs.resize_delta,
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::ResizeDelta(value)),
                    None
                ),
                opt_helpers::toggle(
                    "Transparency",
                    Some("Add transparency to unfocused windows (default: false)"),
                    config.transparency.unwrap_or_default(),
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::Transparency(value))
                ),
                opt_helpers::input(
                    "Transparency Alpha",
                    Some("Alpha value for unfocused window transparency [[0-255]] (default: 200)\n\n\
                                       Value must be greater or equal to 0.0"),
                    "",
                    &app.config_strs.as_ref().unwrap().global_config_strs.transparency_alpha,
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::TransparencyAlpha(value)),
                    None
                ),
                opt_helpers::choose(
                    "Window Container Behaviour",
                    Some("Determine what happens when a new window is opened (default: Create)"),
                    [WindowContainerBehaviour::Create, WindowContainerBehaviour::Append],
                    config.window_container_behaviour,
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::WindowContainerBehaviour(selected)),
                ),
                opt_helpers::choose(
                    "Window Hiding Behaviour",
                    Some("Which Windows signal to use when hiding windows (default: Cloak)"),
                    &HIDING_BEHAVIOUR_OPTIONS[..],
                    Some(&app.config_strs.as_ref().unwrap().global_config_strs.window_hiding_behaviour),
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::WindowHidingBehaviour(selected)),
                ),
            ],
        )
    } else {
        Space::new(Shrink, Shrink).into()
    }
}

pub fn view_monitor(app: &Komofig, monitor_idx: usize) -> Element<Message> {
    if let (Some(m_config_strs), Some(_m_config)) = (
        app.config_strs
            .as_ref()
            .and_then(|c| c.monitors_config_strs.get(&monitor_idx)),
        app.config
            .as_ref()
            .and_then(|c| c.monitors.as_ref().and_then(|m| m.get(monitor_idx))),
    ) {
        opt_helpers::section_view(
            text!("Monitor [{}]:", monitor_idx),
            [
                opt_helpers::expandable(
                    "Window Based Work Area Offset",
                    Some("Window based work area offset (default: None)"),
                    [
                        opt_helpers::input(
                            "left",
                            None,
                            "",
                            &m_config_strs.window_based_work_area_offset_left,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WindowBasedWorkAreaOffsetLeft(value),
                                )
                            }),
                            None,
                        ),
                        opt_helpers::input(
                            "top",
                            None,
                            "",
                            &m_config_strs.window_based_work_area_offset_top,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WindowBasedWorkAreaOffsetTop(value),
                                )
                            }),
                            None,
                        ),
                        opt_helpers::input(
                            "bottom",
                            None,
                            "",
                            &m_config_strs.window_based_work_area_offset_bottom,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WindowBasedWorkAreaOffsetBottom(value),
                                )
                            }),
                            None,
                        ),
                        opt_helpers::input(
                            "right",
                            None,
                            "",
                            &m_config_strs.window_based_work_area_offset_right,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WindowBasedWorkAreaOffsetRight(value),
                                )
                            }),
                            None,
                        ),
                    ],
                    app.config_helpers
                        .monitors_window_based_work_area_offset_expanded[&monitor_idx],
                    Message::ConfigHelpers(
                        ConfigHelpersAction::ToggleMonitorWindowBasedWorkAreaOffsetExpand(
                            monitor_idx,
                        ),
                    ),
                ),
                opt_helpers::input(
                    "Window Based Work Area Offset Limit",
                    Some("Open window limit after which the window based work area offset will no longer be applied (default: 1)"),
                    "",
                    &m_config_strs.window_based_work_area_offset_limit,
                    move |value| {
                        Message::MonitorConfigChanged(
                            monitor_idx,
                            MonitorConfigChangeType::WindowBasedWorkAreaOffsetLimit(value),
                        )
                    },
                    None,
                ),
                opt_helpers::expandable(
                    "Work Area Offset",
                    Some("Monitor-specific work area offset (default: None)"),
                    [
                        opt_helpers::input(
                            "left",
                            None,
                            "",
                            &m_config_strs.work_area_offset_left,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WorkAreaOffsetLeft(value),
                                )
                            }),
                            None,
                        ),
                        opt_helpers::input(
                            "top",
                            None,
                            "",
                            &m_config_strs.work_area_offset_top,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WorkAreaOffsetTop(value),
                                )
                            }),
                            None,
                        ),
                        opt_helpers::input(
                            "bottom",
                            None,
                            "",
                            &m_config_strs.work_area_offset_bottom,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WorkAreaOffsetBottom(value),
                                )
                            }),
                            None,
                        ),
                        opt_helpers::input(
                            "right",
                            None,
                            "",
                            &m_config_strs.work_area_offset_right,
                            Box::new(move |value| {
                                Message::MonitorConfigChanged(
                                    monitor_idx,
                                    MonitorConfigChangeType::WorkAreaOffsetRight(value),
                                )
                            }),
                            None,
                        ),
                    ],
                    app.config_helpers.monitors_work_area_offset_expanded[&monitor_idx],
                    Message::ConfigHelpers(ConfigHelpersAction::ToggleMonitorWorkAreaOffsetExpand(
                        monitor_idx,
                    )),
                ),
            ],
        )
    } else {
        Space::new(Shrink, Shrink).into()
    }
}
