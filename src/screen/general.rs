use crate::{utils::DisplayOption, widget::opt_helpers};

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use iced::{widget::{scrollable, Space}, Element, Length::Shrink, Task};
use komorebi::{
    CrossBoundaryBehaviour, FocusFollowsMouseImplementation, HidingBehaviour, MoveBehaviour,
    WindowContainerBehaviour,
};
use komorebi_client::StaticConfig;
use lazy_static::lazy_static;

lazy_static! {
    static ref CROSS_BOUNDARY_BEHAVIOUR_OPTIONS: [Arc<str>; 2] = [
        Arc::from(CrossBoundaryBehaviour::Monitor.to_string()),
        Arc::from(CrossBoundaryBehaviour::Workspace.to_string()),
    ];
    static ref FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS: [DisplayOption<FocusFollowsMouseImplementation>; 3] = [
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

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleGlobalWorkAreaOffsetExpand,
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    AppSpecificConfigurationPath(Option<PathBuf>),
    CrossBoundaryBehaviour(Arc<str>), // maps komorebi::CrossBoundaryBehaviour to String on GlobalConfigStrs
    CrossMonitorMoveBehaviour(komorebi::MoveBehaviour),
    DefaultContainerPadding(i32),
    DefaultWorkspacePadding(i32),
    DisplayIndexPreferences(HashMap<usize, String>),
    FloatOverride(bool),
    FocusFollowsMouse(Option<komorebi::FocusFollowsMouseImplementation>),
    GlobalWorkAreaOffset(komorebi::Rect),
    GlobalWorkAreaOffsetTop(i32),
    GlobalWorkAreaOffsetBottom(i32),
    GlobalWorkAreaOffsetRight(i32),
    GlobalWorkAreaOffsetLeft(i32),
    MouseFollowsFocus(bool),
    ResizeDelta(i32),
    Transparency(bool),
    TransparencyAlpha(i32),
    UnmanagedWindowBehaviour(komorebi::OperationBehaviour),
    WindowContainerBehaviour(komorebi::WindowContainerBehaviour),
    WindowHidingBehaviour(Arc<str>), // maps komorebi::HidingBehaviour to String on GlobalConfigStrs
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct General {
    pub global_work_area_offset_expanded: bool,
    pub cross_boundary_behaviour: Arc<str>,
    pub window_hiding_behaviour: Arc<str>,
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
                    let behaviour = match value {
                        ref s if **s == *komorebi::CrossBoundaryBehaviour::Monitor.to_string() => {
                            Some(komorebi::CrossBoundaryBehaviour::Monitor)
                        }
                        ref s
                            if **s == *komorebi::CrossBoundaryBehaviour::Workspace.to_string() =>
                        {
                            Some(komorebi::CrossBoundaryBehaviour::Workspace)
                        }
                        _ => None,
                    };
                    config.cross_boundary_behaviour = behaviour;
                    self.cross_boundary_behaviour = value;
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
                ConfigChange::Transparency(value) => {
                    config.transparency = Some(value);
                }
                ConfigChange::TransparencyAlpha(value) => {
                    config.transparency_alpha = Some(value.try_into().unwrap_or(0));
                }
                ConfigChange::UnmanagedWindowBehaviour(value) => {
                    config.unmanaged_window_operation_behaviour = Some(value);
                }
                ConfigChange::WindowContainerBehaviour(value) => {
                    config.window_container_behaviour = Some(value);
                }
                ConfigChange::WindowHidingBehaviour(value) => {
                    let behaviour = match value {
                        ref s if **s == *komorebi::HidingBehaviour::Cloak.to_string() => {
                            Some(komorebi::HidingBehaviour::Cloak)
                        }
                        ref s if **s == *komorebi::HidingBehaviour::Hide.to_string() => {
                            Some(komorebi::HidingBehaviour::Hide)
                        }
                        ref s if **s == *komorebi::HidingBehaviour::Minimize.to_string() => {
                            Some(komorebi::HidingBehaviour::Minimize)
                        }
                        _ => None,
                    };
                    config.window_hiding_behaviour = behaviour;
                    self.window_hiding_behaviour = value;
                }
            },
            Message::ToggleGlobalWorkAreaOffsetExpand => {
                self.global_work_area_offset_expanded = !self.global_work_area_offset_expanded;
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: &'a Option<StaticConfig>) -> Element<'a, Message> {
        if let Some(config) = config {
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
                        &CROSS_BOUNDARY_BEHAVIOUR_OPTIONS[..],
                        Some(&self.cross_boundary_behaviour),
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
                        Message::ToggleGlobalWorkAreaOffsetExpand,
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
                    opt_helpers::toggle(
                        "Transparency",
                        Some("Add transparency to unfocused windows (default: false)"),
                        config.transparency.unwrap_or_default(),
                        |value| Message::ConfigChange(ConfigChange::Transparency(value))
                    ),
                    opt_helpers::number(
                        "Transparency Alpha",
                        Some("Alpha value for unfocused window transparency [[0-255]] (default: 200)\n\n\
                            Value must be greater or equal to 0.0"),
                            config.transparency_alpha.unwrap_or(200).into(),
                            |value| Message::ConfigChange(ConfigChange::TransparencyAlpha(value)),
                    ),
                    opt_helpers::choose(
                        "Window Container Behaviour",
                        Some("Determine what happens when a new window is opened (default: Create)"),
                        [WindowContainerBehaviour::Create, WindowContainerBehaviour::Append],
                        config.window_container_behaviour,
                        |selected| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(selected)),
                    ),
                    opt_helpers::choose(
                        "Window Hiding Behaviour",
                        Some("Which Windows signal to use when hiding windows (default: Cloak)"),
                        &HIDING_BEHAVIOUR_OPTIONS[..],
                        Some(&self.window_hiding_behaviour),
                        |selected| Message::ConfigChange(ConfigChange::WindowHidingBehaviour(selected)),
                    ),
                ],
            )
        } else {
            Space::new(Shrink, Shrink).into()
        }
    }
}
