use std::{path::PathBuf, sync::Arc};

use crate::{
    config::{ConfigHelpersAction, GlobalConfigChangeType},
    widget::opt_helpers::{self, InputParameters},
    Komofig, Message, NONE_STR,
};

use iced::{widget::Space, Element, Length::Shrink};
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
                opt_helpers::input_col(
                    "Global Work Area Offset",
                    None,
                    [
                        InputParameters {
                            name: "top",
                            placeholder: "",
                            value: &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_top,
                            on_change: Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetTop(value))),
                            on_submit: None,
                        },
                        InputParameters {
                            name: "bottom",
                            placeholder: "",
                            value: &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_bottom,
                            on_change: Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetBottom(value))),
                            on_submit: None,
                        },
                        InputParameters {
                            name: "right",
                            placeholder: "",
                            value: &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_right,
                            on_change: Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetRight(value))),
                            on_submit: None,
                        },
                        InputParameters {
                            name: "left",
                            placeholder: "",
                            value: &app.config_strs.as_ref().unwrap().global_config_strs.global_work_area_offset_left,
                            on_change: Box::new(|value| Message::GlobalConfigChanged(GlobalConfigChangeType::GlobalWorkAreaOffsetLeft(value))),
                            on_submit: None,
                        },
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
