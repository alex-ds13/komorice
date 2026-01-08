use crate::screen::View;
use crate::widget::opt_helpers::description_text as t;
use crate::widget::{self, icons};
use crate::{
    BOLD_FONT, ITALIC_FONT,
    config::DEFAULT_CONFIG,
    utils::DisplayOption,
    widget::opt_helpers::{self, DisableArgs},
};

use std::path::PathBuf;

use iced::{
    Element, Task, padding,
    widget::{button, center, column, container, opaque, pick_list, row, rule, text},
};
use komorebi_client::{
    AppSpecificConfigurationPath, AspectRatio, CrossBoundaryBehaviour, FloatingLayerBehaviour,
    FocusFollowsMouseImplementation, HidingBehaviour, MoveBehaviour, OperationBehaviour, Placement,
    PredefinedAspectRatio, Rect, StaticConfig, WindowContainerBehaviour, WindowHandlingBehaviour,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref FOCUS_FOLLOWS_MOUSE_IMPLEMENTATION_OPTIONS: [DisplayOption<FocusFollowsMouseImplementation>; 3] = [
        DisplayOption(None),
        DisplayOption(Some(FocusFollowsMouseImplementation::Komorebi)),
        DisplayOption(Some(FocusFollowsMouseImplementation::Windows)),
    ];
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    PickAscFile(AscKind),
    PickBarConfigFile(BarConfigKind),
    PickedFile(PathBuf),
    ClosedFilePicker,
    Nothing,
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    AppSpecificConfigurationPath(Option<AppSpecificConfigurationPath>),
    AscPathChange(usize, String),
    NewAscPathChange(String),
    AddNewAscPath,
    RemoveAscPath(AscKind),
    BarConfigurations(Option<Vec<PathBuf>>),
    BarConfigPathChange(usize, String),
    NewBarConfigPathChange(String),
    AddNewBarConfigPath,
    RemoveBarConfigPath(usize),
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
    SlowApplicationCompensationTime(Option<u64>),
    UnmanagedWindowBehaviour(Option<OperationBehaviour>),
    WindowContainerBehaviour(Option<WindowContainerBehaviour>),
    WindowHidingBehaviour(Option<HidingBehaviour>),
    FloatingWindowAspectRatio(Option<AspectRatio>),
    FloatingWindowAspectRatioWidth(i32),
    FloatingWindowAspectRatioHeight(i32),
    FloatingLayerBehaviour(Option<FloatingLayerBehaviour>),
    ToggleFloatPlacement(Option<Placement>),
    FloatingLayerPlacement(Option<Placement>),
    FloatOverridePlacement(Option<Placement>),
    FloatRulePlacement(Option<Placement>),
    WindowHandlingBehaviour(Option<WindowHandlingBehaviour>),
}

#[derive(Clone, Debug)]
pub enum AscKind {
    ExistingSingle,
    ExistingMultiple(usize),
    New,
}

#[derive(Clone, Debug)]
pub enum BarConfigKind {
    Existing(usize),
    New,
}

#[derive(Clone, Debug)]
pub enum FilePicker {
    Asc(AscKind),
    BarConfig(BarConfigKind),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct General {
    pub new_asc_path: String,
    pub new_bar_config_path: String,
    pub file_picker: Option<FilePicker>,
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
                ConfigChange::AscPathChange(idx, value) => {
                    if let Some(asc) = &mut config.app_specific_configuration_path {
                        match asc {
                            AppSpecificConfigurationPath::Single(path_buf) => {
                                if idx == 0 {
                                    if value.is_empty() {
                                        config.app_specific_configuration_path = None;
                                    } else {
                                        *path_buf = PathBuf::from(value);
                                    }
                                }
                            }
                            AppSpecificConfigurationPath::Multiple(paths) => {
                                if idx < paths.len() {
                                    if value.is_empty() {
                                        paths.remove(idx);
                                        if paths.len() == 1 {
                                            *asc = AppSpecificConfigurationPath::Single(
                                                paths.remove(0),
                                            );
                                        }
                                    } else {
                                        paths[idx] = PathBuf::from(value);
                                    }
                                }
                            }
                        }
                    }
                }
                ConfigChange::NewAscPathChange(value) => {
                    self.new_asc_path = value;
                }
                ConfigChange::AddNewAscPath => {
                    if let Some(asc) = &mut config.app_specific_configuration_path {
                        match asc {
                            AppSpecificConfigurationPath::Single(path_buf) => {
                                let value = PathBuf::from(std::mem::take(&mut self.new_asc_path));
                                let paths = vec![path_buf.clone(), value];
                                *asc = AppSpecificConfigurationPath::Multiple(paths);
                            }
                            AppSpecificConfigurationPath::Multiple(paths) => {
                                let value = PathBuf::from(std::mem::take(&mut self.new_asc_path));
                                paths.push(value);
                            }
                        }
                    } else {
                        let value = std::mem::take(&mut self.new_asc_path);
                        config.app_specific_configuration_path =
                            Some(AppSpecificConfigurationPath::Single(PathBuf::from(value)));
                    }
                }
                ConfigChange::RemoveAscPath(asc_kind) => match asc_kind {
                    AscKind::ExistingSingle => config.app_specific_configuration_path = None,
                    AscKind::ExistingMultiple(idx) => {
                        if let Some(AppSpecificConfigurationPath::Multiple(paths)) =
                            config.app_specific_configuration_path.as_mut()
                            && idx < paths.len()
                        {
                            paths.remove(idx);
                            if paths.is_empty() {
                                config.app_specific_configuration_path = None;
                            } else if paths.len() == 1 {
                                config.app_specific_configuration_path =
                                    Some(AppSpecificConfigurationPath::Single(paths[0].clone()));
                            }
                        }
                    }
                    AscKind::New => {}
                },
                ConfigChange::BarConfigurations(bar_configs) => {
                    config.bar_configurations = bar_configs;
                }
                ConfigChange::BarConfigPathChange(idx, value) => {
                    if let Some(bar_configs) = &mut config.bar_configurations
                        && idx < bar_configs.len()
                    {
                        if value.is_empty() {
                            bar_configs.remove(idx);
                        } else {
                            bar_configs[idx] = PathBuf::from(value);
                        }
                    }
                }
                ConfigChange::NewBarConfigPathChange(value) => {
                    self.new_bar_config_path = value;
                }
                ConfigChange::AddNewBarConfigPath => {
                    if let Some(bar_configs) = &mut config.bar_configurations {
                        let value = PathBuf::from(std::mem::take(&mut self.new_bar_config_path));
                        bar_configs.push(value);
                    } else {
                        let value = std::mem::take(&mut self.new_bar_config_path);
                        config.bar_configurations = Some(vec![PathBuf::from(value)]);
                    }
                }
                ConfigChange::RemoveBarConfigPath(idx) => {
                    if let Some(bar_configs) = &mut config.bar_configurations
                        && idx < bar_configs.len()
                    {
                        bar_configs.remove(idx);
                        if bar_configs.is_empty() {
                            config.bar_configurations = DEFAULT_CONFIG.bar_configurations.clone();
                        }
                    }
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
                ConfigChange::FloatingWindowAspectRatio(value) => {
                    config.floating_window_aspect_ratio = value;
                }
                ConfigChange::FloatingWindowAspectRatioWidth(value) => {
                    let ratio = if let Some(ratio) = config.floating_window_aspect_ratio {
                        match ratio {
                            AspectRatio::Predefined(_) => AspectRatio::Custom(value, 3),
                            AspectRatio::Custom(_, h) => AspectRatio::Custom(value, h),
                        }
                    } else {
                        AspectRatio::Custom(value, 3)
                    };
                    config.floating_window_aspect_ratio = Some(ratio);
                }
                ConfigChange::FloatingWindowAspectRatioHeight(value) => {
                    let ratio = if let Some(ratio) = config.floating_window_aspect_ratio {
                        match ratio {
                            AspectRatio::Predefined(_) => AspectRatio::Custom(4, value),
                            AspectRatio::Custom(w, _) => AspectRatio::Custom(w, value),
                        }
                    } else {
                        AspectRatio::Custom(4, value)
                    };
                    config.floating_window_aspect_ratio = Some(ratio);
                }
                ConfigChange::FloatingLayerBehaviour(value) => {
                    config.floating_layer_behaviour = value;
                }
                ConfigChange::ToggleFloatPlacement(placement) => {
                    config.toggle_float_placement = placement;
                }
                ConfigChange::FloatingLayerPlacement(placement) => {
                    config.floating_layer_placement = placement;
                }
                ConfigChange::FloatOverridePlacement(placement) => {
                    config.float_override_placement = placement;
                }
                ConfigChange::FloatRulePlacement(placement) => {
                    config.float_rule_placement = placement;
                }
                ConfigChange::WindowHandlingBehaviour(value) => {
                    config.window_handling_behaviour = value;
                }
            },
            Message::PickAscFile(asc_file) => {
                self.file_picker = Some(FilePicker::Asc(asc_file));
                return (Action::None, pick_file());
            }
            Message::PickBarConfigFile(bar_config_file) => {
                self.file_picker = Some(FilePicker::BarConfig(bar_config_file));
                return (Action::None, pick_file());
            }
            Message::PickedFile(path) => {
                if let Some(picker) = self.file_picker.take() {
                    match picker {
                        FilePicker::Asc(asc_file) => match asc_file {
                            AscKind::ExistingSingle => {
                                config.app_specific_configuration_path =
                                    Some(AppSpecificConfigurationPath::Single(path));
                            }
                            AscKind::ExistingMultiple(idx) => {
                                if let Some(AppSpecificConfigurationPath::Multiple(paths)) =
                                    config.app_specific_configuration_path.as_mut()
                                    && let Some(p) = paths.get_mut(idx)
                                {
                                    *p = path;
                                }
                            }
                            AscKind::New => {
                                self.new_asc_path = format!("{}", path.display());
                            }
                        },
                        FilePicker::BarConfig(bar_config_file) => match bar_config_file {
                            BarConfigKind::Existing(idx) => {
                                if let Some(bar_configs) = config.bar_configurations.as_mut()
                                    && let Some(bar_config) = bar_configs.get_mut(idx)
                                {
                                    *bar_config = path;
                                }
                            }
                            BarConfigKind::New => {
                                self.new_bar_config_path = format!("{}", path.display());
                            }
                        },
                    }
                }
            }
            Message::ClosedFilePicker => self.file_picker = None,
            Message::Nothing => {}
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: &'a StaticConfig, show_advanced: bool) -> View<'a, Message> {
        let asc_path = opt_helpers::expandable(
            "App Specific Configuration Path",
            Some(
                "Path to applications.json from komorebi-application-specific-configurations (default: None)",
            ),
            || self.asc_children(&config.app_specific_configuration_path),
            config.app_specific_configuration_path
                != DEFAULT_CONFIG.app_specific_configuration_path,
            Message::ConfigChange(ConfigChange::AppSpecificConfigurationPath(
                DEFAULT_CONFIG.app_specific_configuration_path.clone(),
            )),
            None,
        );
        let bar_configurations = opt_helpers::expandable(
            "Bar Configurations",
            Some(
                "Komorebi status bar configuration files for multiple instances on different monitors. (default: None)",
            ),
            || self.bar_configurations_children(&config.bar_configurations),
            config.bar_configurations != DEFAULT_CONFIG.bar_configurations,
            Message::ConfigChange(ConfigChange::BarConfigurations(
                DEFAULT_CONFIG.bar_configurations.clone(),
            )),
            None,
        );
        let cross_boundary_behaviour = opt_helpers::choose_with_disable_default(
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
        );
        let cross_monitor_move_behaviour = opt_helpers::choose_with_disable_default(
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
        );
        let default_container_padding = opt_helpers::number_with_disable_default_option(
            "Default Container Padding",
            Some("Global default container padding (default: 10)"),
            config
                .default_container_padding
                .or(DEFAULT_CONFIG.default_container_padding),
            DEFAULT_CONFIG.default_container_padding,
            |value| Message::ConfigChange(ConfigChange::DefaultContainerPadding(value)),
            None,
        );
        let default_workspace_padding = opt_helpers::number_with_disable_default_option(
            "Default Workspace Padding",
            Some("Global default workspace padding (default: 10)"),
            config
                .default_workspace_padding
                .or(DEFAULT_CONFIG.default_workspace_padding),
            DEFAULT_CONFIG.default_workspace_padding,
            |value| Message::ConfigChange(ConfigChange::DefaultWorkspacePadding(value)),
            None,
        );
        let float_override = opt_helpers::toggle_with_disable_default(
            "Float Override",
            Some(
                "Enable or disable float override, which makes it so every new window opens in floating mode (default: false)",
            ),
            config.float_override.or(DEFAULT_CONFIG.float_override),
            DEFAULT_CONFIG.float_override,
            |value| Message::ConfigChange(ConfigChange::FloatOverride(value)),
            None,
        );
        let focus_follows_mouse = opt_helpers::choose_with_disable_default(
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
        );
        let global_work_area_offset = opt_helpers::expandable(
            "Global Work Area Offset",
            Some("Global work area (space used for tiling) offset (default: None)"),
            || {
                [
                    opt_helpers::number(
                        "left",
                        None,
                        config.global_work_area_offset.map_or(0, |r| r.left),
                        |value| {
                            Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetLeft(value))
                        },
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
                        |value| {
                            Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetBottom(value))
                        },
                    ),
                    opt_helpers::number(
                        "right",
                        None,
                        config.global_work_area_offset.map_or(0, |r| r.right),
                        |value| {
                            Message::ConfigChange(ConfigChange::GlobalWorkAreaOffsetRight(value))
                        },
                    ),
                ]
            },
            config.global_work_area_offset.is_some(),
            Message::ConfigChange(ConfigChange::GlobalWorkAreaOffset(None)),
            Some(DisableArgs {
                disable: config.global_work_area_offset.is_none(),
                label: Some("None"),
                on_toggle: |v: bool| {
                    Message::ConfigChange(ConfigChange::GlobalWorkAreaOffset(
                        (!v).then_some(Rect::default()),
                    ))
                },
            }),
        );
        let mouse_follows_focus = opt_helpers::toggle_with_disable_default(
            "Mouse Follows Focus",
            Some("Enable or disable mouse follows focus (default: true)"),
            config
                .mouse_follows_focus
                .or(DEFAULT_CONFIG.mouse_follows_focus),
            DEFAULT_CONFIG.mouse_follows_focus,
            |value| Message::ConfigChange(ConfigChange::MouseFollowsFocus(value)),
            None,
        );
        let resize_delta = opt_helpers::number_with_disable_default_option(
            "Resize Delta",
            Some("Delta to resize windows by (default 50)"),
            config.resize_delta.or(DEFAULT_CONFIG.resize_delta),
            DEFAULT_CONFIG.resize_delta,
            |value| Message::ConfigChange(ConfigChange::ResizeDelta(value)),
            None,
        );
        let slow_application_comp_time = opt_helpers::number_with_disable_default_option(
            "Slow Application Compensation Time",
            Some(
                "How long to wait when compensating for slow applications, \
            in milliseconds (default: 20)\n\n\
            Value must be greater or equal to 0.",
            ),
            config
                .slow_application_compensation_time
                .or(DEFAULT_CONFIG.slow_application_compensation_time),
            DEFAULT_CONFIG.slow_application_compensation_time,
            |value| Message::ConfigChange(ConfigChange::SlowApplicationCompensationTime(value)),
            None,
        );
        let unmanaged_window_behavior = opt_helpers::choose_with_disable_default(
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
        );
        let window_container_behaviour = opt_helpers::choose_with_disable_default(
            "Window Container Behaviour",
            Some("Determine what happens when a new window is opened (default: Create)"),
            vec![
                t("Selected: 'Create' -> Create a new container for each new window").into(),
                t("Selected: 'Append' -> Append new windows to the focused window container")
                    .into(),
            ],
            [
                WindowContainerBehaviour::Create,
                WindowContainerBehaviour::Append,
            ],
            config
                .window_container_behaviour
                .or(DEFAULT_CONFIG.window_container_behaviour),
            |selected| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(selected)),
            DEFAULT_CONFIG.window_container_behaviour,
            None,
        );
        let window_hiding_behaviour = opt_helpers::choose_with_disable_default(
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
        );
        let floating_window_aspect_ratio = opt_helpers::expandable_custom(
            "Floating Window Aspect Ratio",
            get_aspect_ratio_description(
                config
                    .floating_window_aspect_ratio
                    .or(DEFAULT_CONFIG.floating_window_aspect_ratio)
                    .map(Into::<crate::komo_interop::aspect_ratio::AspectRatio>::into),
            ),
            |_, _| {
                pick_list(
                    [
                        crate::komo_interop::aspect_ratio::AspectRatio::Standard,
                        crate::komo_interop::aspect_ratio::AspectRatio::Widescreen,
                        crate::komo_interop::aspect_ratio::AspectRatio::Ultrawide,
                        crate::komo_interop::aspect_ratio::AspectRatio::Custom(
                            config
                                .floating_window_aspect_ratio
                                .map_or(4, |ar| match ar {
                                    AspectRatio::Predefined(predefined_aspect_ratio) => {
                                        match predefined_aspect_ratio {
                                            PredefinedAspectRatio::Ultrawide => 21,
                                            PredefinedAspectRatio::Widescreen => 16,
                                            PredefinedAspectRatio::Standard => 4,
                                        }
                                    }
                                    AspectRatio::Custom(w, _) => w,
                                }),
                            config
                                .floating_window_aspect_ratio
                                .map_or(3, |ar| match ar {
                                    AspectRatio::Predefined(predefined_aspect_ratio) => {
                                        match predefined_aspect_ratio {
                                            PredefinedAspectRatio::Ultrawide => 9,
                                            PredefinedAspectRatio::Widescreen => 9,
                                            PredefinedAspectRatio::Standard => 3,
                                        }
                                    }
                                    AspectRatio::Custom(_, h) => h,
                                }),
                        ),
                    ],
                    config
                        .floating_window_aspect_ratio
                        .or(DEFAULT_CONFIG.floating_window_aspect_ratio)
                        .map(Into::<crate::komo_interop::aspect_ratio::AspectRatio>::into),
                    |selected| {
                        Message::ConfigChange(ConfigChange::FloatingWindowAspectRatio(Some(
                            selected.into(),
                        )))
                    },
                )
            },
            || {
                [
                    opt_helpers::number(
                        "width:",
                        None,
                        config
                            .floating_window_aspect_ratio
                            .map_or(0, |ar| match ar {
                                AspectRatio::Predefined(predefined_aspect_ratio) => {
                                    match predefined_aspect_ratio {
                                        PredefinedAspectRatio::Ultrawide => 21,
                                        PredefinedAspectRatio::Widescreen => 16,
                                        PredefinedAspectRatio::Standard => 4,
                                    }
                                }
                                AspectRatio::Custom(w, _) => w,
                            }),
                        |v| Message::ConfigChange(ConfigChange::FloatingWindowAspectRatioWidth(v)),
                    ),
                    opt_helpers::number(
                        "height:",
                        None,
                        config
                            .floating_window_aspect_ratio
                            .map_or(0, |ar| match ar {
                                AspectRatio::Predefined(predefined_aspect_ratio) => {
                                    match predefined_aspect_ratio {
                                        PredefinedAspectRatio::Ultrawide => 9,
                                        PredefinedAspectRatio::Widescreen => 9,
                                        PredefinedAspectRatio::Standard => 3,
                                    }
                                }
                                AspectRatio::Custom(_, h) => h,
                            }),
                        |v| Message::ConfigChange(ConfigChange::FloatingWindowAspectRatioHeight(v)),
                    ),
                ]
            },
            config.floating_window_aspect_ratio != DEFAULT_CONFIG.floating_window_aspect_ratio,
            matches!(
                config.floating_window_aspect_ratio,
                Some(AspectRatio::Custom(_, _))
            ),
            Message::ConfigChange(ConfigChange::FloatingWindowAspectRatio(
                DEFAULT_CONFIG.floating_window_aspect_ratio,
            )),
            None,
        );
        let floating_layer_behaviour = || {
            opt_helpers::choose_with_disable_default(
            "Floating Layer Behaviour",
            Some("Determines what happens to a new window when on the `FloatingLayer` (default: Tile)"),
            vec![
                t("Selected: 'Tile' -> Tile new windows (unless they match a float rule or float override is active)").into(),
                t("Selected: 'Float' -> Float new windows").into(),
            ],
            [FloatingLayerBehaviour::Tile, FloatingLayerBehaviour::Float],
            config.floating_layer_behaviour.or(DEFAULT_CONFIG.floating_layer_behaviour),
            |v| Message::ConfigChange(ConfigChange::FloatingLayerBehaviour(v)),
            DEFAULT_CONFIG.floating_layer_behaviour,
            None,
        )
        };
        let toggle_float_placement = opt_helpers::choose_with_disable_default(
            "Toggle Float Placement",
            Some("Determines the placement of a window when toggling to float (default: CenterAndResize)"),
            vec![
                t("Selected: 'None' -> Does not change the size or position of the window").into(),
                t("Selected: 'Center' -> Center the window without changing the size").into(),
                t("Selected: 'CenterAndResize' -> Center the window and resize it according to the `AspectRatio`").into(),
            ],
            [Placement::None, Placement::Center, Placement::CenterAndResize],
            config.toggle_float_placement.or(DEFAULT_CONFIG.toggle_float_placement),
            |v| Message::ConfigChange(ConfigChange::ToggleFloatPlacement(v)),
            DEFAULT_CONFIG.toggle_float_placement,
            None,
        );
        let floating_layer_placement = opt_helpers::choose_with_disable_default(
            "Floating Layer Placement",
            Some("Determines the `Placement` to be used when spawning a window on the floating layer with the \
            `FloatingLayerBehaviour` set to `FloatingLayerBehaviour::Float` (default: Center)"),
            vec![
                t("Selected: 'None' -> Does not change the size or position of the window").into(),
                t("Selected: 'Center' -> Center the window without changing the size").into(),
                t("Selected: 'CenterAndResize' -> Center the window and resize it according to the `AspectRatio`").into(),
            ],
            [Placement::None, Placement::Center, Placement::CenterAndResize],
            config.floating_layer_placement.or(DEFAULT_CONFIG.floating_layer_placement),
            |v| Message::ConfigChange(ConfigChange::FloatingLayerPlacement(v)),
            DEFAULT_CONFIG.floating_layer_placement,
            None,
        );
        let float_override_placement = opt_helpers::choose_with_disable_default(
            "Float Override Placement",
            Some("Determines the `Placement` to be used when spawning a window with float override active (default: None)"),
            vec![
                t("Selected: 'None' -> Does not change the size or position of the window").into(),
                t("Selected: 'Center' -> Center the window without changing the size").into(),
                t("Selected: 'CenterAndResize' -> Center the window and resize it according to the `AspectRatio`").into(),
            ],
            [Placement::None, Placement::Center, Placement::CenterAndResize],
            config.float_override_placement.or(DEFAULT_CONFIG.float_override_placement),
            |v| Message::ConfigChange(ConfigChange::FloatOverridePlacement(v)),
            DEFAULT_CONFIG.float_override_placement,
            None,
        );
        let float_rule_placement = opt_helpers::choose_with_disable_default(
            "Float Rule Placement",
            Some("Determines the `Placement` to be used when spawning a window that matches a 'floating_applications' rule (default: None)"),
            vec![
                t("Selected: 'None' -> Does not change the size or position of the window").into(),
                t("Selected: 'Center' -> Center the window without changing the size").into(),
                t("Selected: 'CenterAndResize' -> Center the window and resize it according to the `AspectRatio`").into(),
            ],
            [Placement::None, Placement::Center, Placement::CenterAndResize],
            config.float_rule_placement.or(DEFAULT_CONFIG.float_rule_placement),
            |v| Message::ConfigChange(ConfigChange::FloatRulePlacement(v)),
            DEFAULT_CONFIG.float_rule_placement,
            None,
        );
        let window_handling_behaviour = opt_helpers::choose_with_disable_default(
            "Window Handling Behaviour",
            Some("Which Windows API behaviour to use when manipulating windows (default: Sync)"),
            vec![
                t("Selected: 'Sync' -> Uses synchronous Windows API to manipulate windows.\n\
                If a window hangs when komorebi is trying to access it it might hang komorebi as well.").into(),
                t("Selected: 'Async' -> Uses asynchronous Windows API to manipulate windows.\n\
                Hanging windows don't block komorebi.").into(),
            ],
            [WindowHandlingBehaviour::Sync, WindowHandlingBehaviour::Async],
            config.window_handling_behaviour.or(DEFAULT_CONFIG.window_handling_behaviour),
            |v| Message::ConfigChange(ConfigChange::WindowHandlingBehaviour(v)),
            DEFAULT_CONFIG.window_handling_behaviour,
            None,
        );
        let mut contents = vec![
            asc_path,
            bar_configurations,
            mouse_follows_focus,
            resize_delta,
            opt_helpers::sub_section_view(
                title("Gaps/Padding:"),
                [
                    default_container_padding,
                    default_workspace_padding,
                    global_work_area_offset,
                ],
            ),
            opt_helpers::sub_section_view(
                title("Behaviours:"),
                [
                    cross_boundary_behaviour,
                    cross_monitor_move_behaviour,
                    unmanaged_window_behavior,
                    window_container_behaviour,
                    window_hiding_behaviour,
                    window_handling_behaviour,
                    floating_layer_behaviour(),
                ],
            ),
            opt_helpers::sub_section_view(
                title("Floating Windows:"),
                [
                    float_override,
                    floating_window_aspect_ratio,
                    floating_layer_behaviour(),
                    toggle_float_placement,
                    floating_layer_placement,
                    float_override_placement,
                    float_rule_placement,
                ],
            ),
        ];

        if show_advanced {
            contents.push(opt_helpers::sub_section_view(
                row![
                    text("Advanced:").size(18).font(*BOLD_FONT),
                    text("(You shouldn't need to mess with these ones...)")
                        .size(12)
                        .font(*ITALIC_FONT)
                ]
                .padding(padding::top(20))
                .spacing(5)
                .align_y(iced::Center),
                [focus_follows_mouse, slow_application_comp_time],
            ));
        }

        let modal = self.file_picker.is_some().then(|| opaque(center("")));
        View::new(opt_helpers::section_view("General:", contents)).modal(modal, Message::Nothing)
    }

    fn asc_children<'a>(
        &'a self,
        asc_path: &'a Option<AppSpecificConfigurationPath>,
    ) -> Vec<Element<'a, Message>> {
        let mut elements = Vec::new();
        elements.push(text("Current paths:").into());
        if let Some(asc) = asc_path {
            match asc {
                AppSpecificConfigurationPath::Single(path_buf) => {
                    let file_picker = button(icons::folder())
                        .style(subtle)
                        .on_press(Message::PickAscFile(AscKind::ExistingSingle));
                    let path_input = widget::input(
                        "",
                        path_buf.to_str().unwrap_or_default(),
                        |v| Message::ConfigChange(ConfigChange::AscPathChange(0, v)),
                        None,
                    );
                    let remove = button(icons::delete().style(text::danger))
                        .style(subtle)
                        .on_press(Message::ConfigChange(ConfigChange::RemoveAscPath(
                            AscKind::ExistingSingle,
                        )));
                    elements.push(row![file_picker, path_input, remove].spacing(10).into());
                }
                AppSpecificConfigurationPath::Multiple(paths) => {
                    for (idx, path_buf) in paths.iter().enumerate() {
                        let file_picker = button(icons::folder())
                            .style(subtle)
                            .on_press(Message::PickAscFile(AscKind::ExistingMultiple(idx)));
                        let path_input = widget::input(
                            "",
                            path_buf.to_str().unwrap_or_default(),
                            move |v| Message::ConfigChange(ConfigChange::AscPathChange(idx, v)),
                            None,
                        );
                        let remove = button(icons::delete().style(text::danger))
                            .style(subtle)
                            .on_press(Message::ConfigChange(ConfigChange::RemoveAscPath(
                                AscKind::ExistingMultiple(idx),
                            )));
                        elements.push(row![file_picker, path_input, remove].spacing(10).into());
                    }
                }
            }
        }
        let add_new_msg = (!self.new_asc_path.is_empty())
            .then_some(Message::ConfigChange(ConfigChange::AddNewAscPath));
        let is_enabled = add_new_msg.is_some();

        let file_picker = button(icons::folder())
            .style(subtle)
            .on_press(Message::PickAscFile(AscKind::New));
        let new_path = widget::input(
            "",
            &self.new_asc_path,
            |v| Message::ConfigChange(ConfigChange::NewAscPathChange(v)),
            add_new_msg.clone(),
        );
        let add_button = button(icons::plus().style(move |t| {
            let color = if is_enabled {
                t.palette().primary.into()
            } else {
                t.extended_palette().secondary.base.color.into()
            };
            text::Style { color }
        }))
        .on_press_maybe(add_new_msg)
        .style(button::text);
        let new_path_row = row![file_picker, new_path, add_button].spacing(10);

        let new_path_col = column![rule::horizontal(2), text("Add new path:"), new_path_row]
            .spacing(10)
            .padding(padding::top(10));
        elements.push(new_path_col.into());
        elements
    }

    fn bar_configurations_children<'a>(
        &'a self,
        bar_configs: &'a Option<Vec<PathBuf>>,
    ) -> Vec<Element<'a, Message>> {
        let mut elements = Vec::new();
        elements.push(text("Current configurations:").into());
        for (idx, path_buf) in bar_configs.iter().flatten().enumerate() {
            let file_picker = button(icons::folder())
                .style(subtle)
                .on_press(Message::PickBarConfigFile(BarConfigKind::Existing(idx)));
            let path_input = widget::input(
                "",
                path_buf.to_str().unwrap_or_default(),
                move |v| Message::ConfigChange(ConfigChange::BarConfigPathChange(idx, v)),
                None,
            );
            let remove = button(icons::delete().style(text::danger))
                .style(subtle)
                .on_press(Message::ConfigChange(ConfigChange::RemoveBarConfigPath(
                    idx,
                )));
            elements.push(row![file_picker, path_input, remove].spacing(10).into());
        }
        let add_new_msg = (!self.new_bar_config_path.is_empty())
            .then_some(Message::ConfigChange(ConfigChange::AddNewBarConfigPath));
        let is_enabled = add_new_msg.is_some();

        let file_picker = button(icons::folder())
            .style(subtle)
            .on_press(Message::PickBarConfigFile(BarConfigKind::New));
        let new_path = widget::input(
            "",
            &self.new_bar_config_path,
            |v| Message::ConfigChange(ConfigChange::NewBarConfigPathChange(v)),
            add_new_msg.clone(),
        );
        let add_button = button(icons::plus().style(move |t| {
            let color = if is_enabled {
                t.palette().primary.into()
            } else {
                t.extended_palette().secondary.base.color.into()
            };
            text::Style { color }
        }))
        .on_press_maybe(add_new_msg)
        .style(button::text);
        let new_path_row = row![file_picker, new_path, add_button].spacing(10);

        let new_path_col = column![
            rule::horizontal(2),
            text("Add new configuration:"),
            new_path_row
        ]
        .spacing(10)
        .padding(padding::top(10));
        elements.push(new_path_col.into());
        elements
    }
}

fn get_aspect_ratio_description(
    selected: Option<crate::komo_interop::aspect_ratio::AspectRatio>,
) -> Option<&'static str> {
    if let Some(selected) = selected {
        match selected {
            crate::komo_interop::aspect_ratio::AspectRatio::Standard => Some(
                "Aspect ratio to resize with when toggling floating mode for a window. (default: Standard (4:3))\n\n\
                Selected: 'Standard (4:3)' -> Use a 4:3 ratio when toggling windows to floating",
            ),
            crate::komo_interop::aspect_ratio::AspectRatio::Widescreen => Some(
                "Aspect ratio to resize with when toggling floating mode for a window. (default: Standard (4:3))\n\n\
                Selected: 'Widescreen (16:9)' -> Use a 16:9 ratio when toggling windows to floating",
            ),
            crate::komo_interop::aspect_ratio::AspectRatio::Ultrawide => Some(
                "Aspect ratio to resize with when toggling floating mode for a window. (default: Standard (4:3))\n\n\
                Selected: 'Ultrawide (21:9)' -> Use a 21:9 ratio when toggling windows to floating",
            ),
            crate::komo_interop::aspect_ratio::AspectRatio::Custom(_, _) => Some(
                "Aspect ratio to resize with when toggling floating mode for a window. (default: Standard (4:3))\n\n\
                Selected: 'Custom' -> Use a custom ratio when toggling windows to floating",
            ),
        }
    } else {
        Some(
            "Aspect ratio to resize with when toggling floating mode for a window. (default: Standard (4:3))",
        )
    }
}

fn title(title: &str) -> container::Container<'_, Message> {
    container(text(title).size(18).font(*BOLD_FONT))
        .padding(padding::top(20))
        .align_y(iced::Center)
}

fn pick_file() -> Task<Message> {
    let (home_dir, _) = crate::config::home_path();
    Task::future(async move {
        rfd::FileDialog::new()
            .add_filter("json", &["json"])
            .set_directory(home_dir.as_path())
            .pick_file()
    })
    .map(|res| match res {
        Some(file) => Message::PickedFile(file),
        None => Message::ClosedFilePicker,
    })
}

fn subtle(theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: None,
            ..button::subtle(theme, status)
        },
        button::Status::Hovered | button::Status::Pressed | button::Status::Disabled => {
            button::subtle(theme, status)
        }
    }
}
