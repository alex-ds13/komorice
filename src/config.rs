use crate::apperror::{AppError, AppErrorKind};
use crate::{config, Message};

use std::sync::Arc;
use std::time::Duration;

use async_std::channel::{self, Receiver};
use iced::futures::{SinkExt, StreamExt};
use iced::Subscription;
use komorebi_client::{
    AnimationStyle, AnimationsConfig, BorderColours, BorderImplementation, BorderStyle, Colour,
    CrossBoundaryBehaviour, DefaultLayout, HidingBehaviour, KomorebiTheme, MonitorConfig,
    MoveBehaviour, OperationBehaviour, PerAnimationPrefixConfig, Rgb, StackbarConfig,
    StackbarLabel, StackbarMode, StaticConfig, TabsConfig, WindowContainerBehaviour,
    WorkspaceConfig,
};
use komorebi_themes::{Base16, Base16Value, Catppuccin, CatppuccinValue};
use lazy_static::lazy_static;
use notify_debouncer_mini::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer,
};

lazy_static! {
    pub static ref DEFAULT_CONFIG: StaticConfig = StaticConfig {
        invisible_borders: None,
        minimum_window_width: None,
        minimum_window_height: None,
        resize_delta: Some(50),
        window_container_behaviour: Some(WindowContainerBehaviour::Create),
        float_override: Some(false),
        cross_monitor_move_behaviour: Some(MoveBehaviour::Swap),
        cross_boundary_behaviour: Some(CrossBoundaryBehaviour::Monitor),
        unmanaged_window_operation_behaviour: Some(OperationBehaviour::Op),
        focus_follows_mouse: None,
        mouse_follows_focus: Some(true),
        app_specific_configuration_path: None,
        border_width: Some(8),
        border_offset: Some(-1),
        border: Some(false),
        border_colours: Some(BorderColours {
            single: Some(Colour::Rgb(Rgb::new(66, 165, 245))),
            stack: Some(Colour::Rgb(Rgb::new(0, 165, 66))),
            monocle: Some(Colour::Rgb(Rgb::new(255, 51, 153))),
            floating: Some(Colour::Rgb(Rgb::new(245, 245, 165))),
            unfocused: Some(Colour::Rgb(Rgb::new(128, 128, 128))),
        }),
        border_style: Some(BorderStyle::default()),
        border_z_order: None,
        border_implementation: Some(BorderImplementation::default()),
        transparency: Some(false),
        transparency_alpha: Some(200),
        transparency_ignore_rules: Some(Vec::new()),
        default_workspace_padding: Some(10),
        default_container_padding: Some(10),
        monitors: None,
        window_hiding_behaviour: Some(HidingBehaviour::Cloak),
        global_work_area_offset: None,
        ignore_rules: Some(Vec::new()),
        manage_rules: Some(Vec::new()),
        floating_applications: Some(Vec::new()),
        border_overflow_applications: Some(Vec::new()),
        tray_and_multi_window_applications: Some(Vec::new()),
        layered_applications: Some(Vec::new()),
        object_name_change_applications: Some(Vec::new()),
        monitor_index_preferences: None,
        display_index_preferences: None,
        stackbar: Some(StackbarConfig {
            height: Some(40),
            label: Some(StackbarLabel::Process),
            mode: Some(StackbarMode::OnStack),
            tabs: Some(TabsConfig {
                width: Some(200),
                focused_text: Some(Colour::Rgb(Rgb::new(0xFF, 0xFF, 0xFF))),
                unfocused_text: Some(Colour::Rgb(Rgb::new(0xB3, 0xB3, 0xB3))),
                background: Some(Colour::Rgb(Rgb::new(0x33, 0x33, 0x33))),
                font_family: None,
                font_size: None,
            }),
        }),
        animation: Some(AnimationsConfig {
            enabled: PerAnimationPrefixConfig::Global(false),
            duration: Some(PerAnimationPrefixConfig::Global(250)),
            style: Some(PerAnimationPrefixConfig::Global(AnimationStyle::Linear)),
            fps: Some(60),
        }),
        theme: None,
        slow_application_identifiers: Some(Vec::new()),
        slow_application_compensation_time: Some(20),
        bar_configurations: Some(Vec::new()),
        remove_titlebar_applications: Some(Vec::new()),
    };
    pub static ref DEFAULT_MONITOR_CONFIG: MonitorConfig = MonitorConfig {
        workspaces: Vec::new(),
        work_area_offset: None,
        window_based_work_area_offset: None,
        window_based_work_area_offset_limit: Some(1),
    };
    pub static ref DEFAULT_WORKSPACE_CONFIG: WorkspaceConfig = WorkspaceConfig {
        name: String::new(),
        layout: Some(DefaultLayout::BSP),
        custom_layout: None,
        layout_rules: None,
        custom_layout_rules: None,
        container_padding: None,
        workspace_padding: None,
        initial_workspace_rules: Some(Vec::new()),
        workspace_rules: Some(Vec::new()),
        apply_window_based_work_area_offset: Some(true),
        window_container_behaviour: None,
        float_override: None,
    };
    pub static ref DEFAULT_CATPPUCCIN_THEME: KomorebiTheme = KomorebiTheme::Catppuccin {
        name: Catppuccin::Macchiato,
        single_border: Some(CatppuccinValue::Blue),
        stack_border: Some(CatppuccinValue::Green),
        monocle_border: Some(CatppuccinValue::Pink),
        floating_border: Some(CatppuccinValue::Yellow),
        unfocused_border: Some(CatppuccinValue::Base),
        stackbar_focused_text: Some(CatppuccinValue::Green),
        stackbar_unfocused_text: Some(CatppuccinValue::Text),
        stackbar_background: Some(CatppuccinValue::Base),
        bar_accent: Some(CatppuccinValue::Blue),
    };
    pub static ref DEFAULT_BASE16_THEME: KomorebiTheme = KomorebiTheme::Base16 {
        name: Base16::Ashes,
        single_border: Some(Base16Value::Base0D),
        stack_border: Some(Base16Value::Base0B),
        monocle_border: Some(Base16Value::Base0F),
        floating_border: Some(Base16Value::Base09),
        unfocused_border: Some(Base16Value::Base01),
        stackbar_focused_text: Some(Base16Value::Base0B),
        stackbar_unfocused_text: Some(Base16Value::Base05),
        stackbar_background: Some(Base16Value::Base01),
        bar_accent: Some(Base16Value::Base0D),
    };
}

/// Adds any missing `MonitorConfig` on `config`. It checks the physical amount of monitors, if the
/// amount of `MonitorConfig` on the `config` is less than the physical amount then it adds default
/// `MonitorConfig` until it has the same amount.
/// Returns a bool that says wether changes were made or not.
pub fn fill_monitors(config: &mut StaticConfig) -> bool {
    let monitors = crate::monitors::get_displays();
    if let Some(config_monitors) = &mut config.monitors {
        let physical_monitors_len = monitors.len();
        let config_monitors_len = config_monitors.len();
        if physical_monitors_len <= config_monitors_len {
            return false;
        }
        for _ in 0..(physical_monitors_len - config_monitors_len) {
            config_monitors.push(MonitorConfig {
                workspaces: vec![DEFAULT_WORKSPACE_CONFIG.clone()],
                ..*DEFAULT_MONITOR_CONFIG
            })
        }
    } else {
        config.monitors = Some(
            monitors
                .iter()
                .map(|_| komorebi_client::MonitorConfig {
                    workspaces: vec![komorebi_client::WorkspaceConfig {
                        name: String::new(),
                        layout: Some(komorebi_client::DefaultLayout::BSP),
                        custom_layout: None,
                        layout_rules: None,
                        custom_layout_rules: None,
                        container_padding: None,
                        workspace_padding: None,
                        initial_workspace_rules: None,
                        workspace_rules: None,
                        apply_window_based_work_area_offset: None,
                        window_container_behaviour: None,
                        float_override: None,
                    }],
                    work_area_offset: None,
                    window_based_work_area_offset: None,
                    window_based_work_area_offset_limit: None,
                })
                .collect(),
        );
    }
    true
}

/// It checks the value against the default config value. If the value is the default value, then
/// we change value to `None` so that it doesn't show on the final config file. Otherwise, we keep
/// value as it was.
pub fn sanitize_value<T: Clone + PartialEq>(
    value: Option<T>,
    getter: impl Fn(&StaticConfig) -> &Option<T>,
) -> Option<T> {
    let default_value = getter(&DEFAULT_CONFIG);
    if value == *default_value {
        None
    } else {
        value
    }
}

/// Merge the `DEFAULT_CONFIG` values on `config`. For each value that is `None` on `config`
/// it uses the corresponding value from `DEFAULT_CONFIG`.
/// It returns a new `StaticConfig` with the result.
pub fn merge_default(config: StaticConfig) -> StaticConfig {
    StaticConfig {
        invisible_borders: config
            .invisible_borders
            .or(DEFAULT_CONFIG.invisible_borders),
        minimum_window_width: config
            .minimum_window_width
            .or(DEFAULT_CONFIG.minimum_window_width),
        minimum_window_height: config
            .minimum_window_height
            .or(DEFAULT_CONFIG.minimum_window_height),
        resize_delta: config.resize_delta.or(DEFAULT_CONFIG.resize_delta),
        window_container_behaviour: config
            .window_container_behaviour
            .or(DEFAULT_CONFIG.window_container_behaviour),
        float_override: config.float_override.or(DEFAULT_CONFIG.float_override),
        cross_monitor_move_behaviour: config
            .cross_monitor_move_behaviour
            .or(DEFAULT_CONFIG.cross_monitor_move_behaviour),
        cross_boundary_behaviour: config
            .cross_boundary_behaviour
            .or(DEFAULT_CONFIG.cross_boundary_behaviour),
        unmanaged_window_operation_behaviour: config
            .unmanaged_window_operation_behaviour
            .or(DEFAULT_CONFIG.unmanaged_window_operation_behaviour),
        focus_follows_mouse: config
            .focus_follows_mouse
            .or(DEFAULT_CONFIG.focus_follows_mouse),
        mouse_follows_focus: config
            .mouse_follows_focus
            .or(DEFAULT_CONFIG.mouse_follows_focus),
        app_specific_configuration_path: config
            .app_specific_configuration_path
            .as_ref()
            .cloned()
            .or(DEFAULT_CONFIG.app_specific_configuration_path.clone()),
        border_width: config.border_width.or(DEFAULT_CONFIG.border_width),
        border_offset: config.border_offset.or(DEFAULT_CONFIG.border_offset),
        border: config.border.or(DEFAULT_CONFIG.border),
        border_colours: config.border_colours.map(|bc| BorderColours {
            single: bc.single.or(DEFAULT_CONFIG
                .border_colours
                .as_ref()
                .and_then(|bc| bc.single)),
            stack: bc.stack.or(DEFAULT_CONFIG
                .border_colours
                .as_ref()
                .and_then(|bc| bc.stack)),
            monocle: bc.monocle.or(DEFAULT_CONFIG
                .border_colours
                .as_ref()
                .and_then(|bc| bc.monocle)),
            floating: bc.floating.or(DEFAULT_CONFIG
                .border_colours
                .as_ref()
                .and_then(|bc| bc.floating)),
            unfocused: bc.unfocused.or(DEFAULT_CONFIG
                .border_colours
                .as_ref()
                .and_then(|bc| bc.unfocused)),
        }),
        border_style: config.border_style.or(DEFAULT_CONFIG.border_style),
        border_z_order: config.border_z_order.or(DEFAULT_CONFIG.border_z_order),
        border_implementation: config
            .border_implementation
            .or(DEFAULT_CONFIG.border_implementation),
        transparency: config.transparency.or(DEFAULT_CONFIG.transparency),
        transparency_alpha: config
            .transparency_alpha
            .or(DEFAULT_CONFIG.transparency_alpha),
        transparency_ignore_rules: config
            .transparency_ignore_rules
            .or(DEFAULT_CONFIG.transparency_ignore_rules.clone()),
        default_workspace_padding: config
            .default_workspace_padding
            .or(DEFAULT_CONFIG.default_workspace_padding),
        default_container_padding: config
            .default_container_padding
            .or(DEFAULT_CONFIG.default_container_padding),
        monitors: config.monitors.map(|ms| {
            ms.into_iter()
                .map(|m| MonitorConfig {
                    workspaces: m
                        .workspaces
                        .into_iter()
                        .map(|w| WorkspaceConfig {
                            name: w.name,
                            layout: w.layout,
                            custom_layout: w
                                .custom_layout
                                .or(DEFAULT_WORKSPACE_CONFIG.custom_layout.clone()),
                            layout_rules: w
                                .layout_rules
                                .or(DEFAULT_WORKSPACE_CONFIG.layout_rules.clone()),
                            custom_layout_rules: w
                                .custom_layout_rules
                                .or(DEFAULT_WORKSPACE_CONFIG.custom_layout_rules.clone()),
                            container_padding: w
                                .container_padding
                                .or(DEFAULT_WORKSPACE_CONFIG.container_padding),
                            workspace_padding: w
                                .workspace_padding
                                .or(DEFAULT_WORKSPACE_CONFIG.workspace_padding),
                            initial_workspace_rules: w
                                .initial_workspace_rules
                                .or(DEFAULT_WORKSPACE_CONFIG.initial_workspace_rules.clone()),
                            workspace_rules: w
                                .workspace_rules
                                .or(DEFAULT_WORKSPACE_CONFIG.workspace_rules.clone()),
                            apply_window_based_work_area_offset: w
                                .apply_window_based_work_area_offset
                                .or(DEFAULT_WORKSPACE_CONFIG.apply_window_based_work_area_offset),
                            window_container_behaviour: w
                                .window_container_behaviour
                                .or(DEFAULT_WORKSPACE_CONFIG.window_container_behaviour),
                            float_override: w
                                .float_override
                                .or(DEFAULT_WORKSPACE_CONFIG.float_override),
                        })
                        .collect(),
                    work_area_offset: m
                        .work_area_offset
                        .or(DEFAULT_MONITOR_CONFIG.work_area_offset),
                    window_based_work_area_offset: m
                        .window_based_work_area_offset
                        .or(DEFAULT_MONITOR_CONFIG.window_based_work_area_offset),
                    window_based_work_area_offset_limit: m
                        .window_based_work_area_offset_limit
                        .or(DEFAULT_MONITOR_CONFIG.window_based_work_area_offset_limit),
                })
                .collect()
        }),
        window_hiding_behaviour: config
            .window_hiding_behaviour
            .or(DEFAULT_CONFIG.window_hiding_behaviour),
        global_work_area_offset: config
            .global_work_area_offset
            .or(DEFAULT_CONFIG.global_work_area_offset),
        ignore_rules: config.ignore_rules.or(DEFAULT_CONFIG.ignore_rules.clone()),
        manage_rules: config.manage_rules.or(DEFAULT_CONFIG.manage_rules.clone()),
        floating_applications: config
            .floating_applications
            .or(DEFAULT_CONFIG.floating_applications.clone()),
        border_overflow_applications: config
            .border_overflow_applications
            .or(DEFAULT_CONFIG.border_overflow_applications.clone()),
        tray_and_multi_window_applications: config
            .tray_and_multi_window_applications
            .or(DEFAULT_CONFIG.tray_and_multi_window_applications.clone()),
        layered_applications: config
            .layered_applications
            .or(DEFAULT_CONFIG.layered_applications.clone()),
        object_name_change_applications: config
            .object_name_change_applications
            .or(DEFAULT_CONFIG.object_name_change_applications.clone()),
        monitor_index_preferences: config
            .monitor_index_preferences
            .or(DEFAULT_CONFIG.monitor_index_preferences.clone()),
        display_index_preferences: config
            .display_index_preferences
            .or(DEFAULT_CONFIG.display_index_preferences.clone()),
        stackbar: config.stackbar.map(|s| StackbarConfig {
            height: s
                .height
                .or(DEFAULT_CONFIG.stackbar.as_ref().and_then(|s| s.height)),
            label: s
                .label
                .or(DEFAULT_CONFIG.stackbar.as_ref().and_then(|s| s.label)),
            mode: s
                .mode
                .or(DEFAULT_CONFIG.stackbar.as_ref().and_then(|s| s.mode)),
            tabs: s.tabs.map(|t| TabsConfig {
                width: t.width.or(DEFAULT_CONFIG
                    .stackbar
                    .as_ref()
                    .and_then(|s| s.tabs.as_ref().and_then(|t| t.width))),
                focused_text: t.focused_text.or(DEFAULT_CONFIG
                    .stackbar
                    .as_ref()
                    .and_then(|s| s.tabs.as_ref().and_then(|t| t.focused_text))),
                unfocused_text: t.unfocused_text.or(DEFAULT_CONFIG
                    .stackbar
                    .as_ref()
                    .and_then(|s| s.tabs.as_ref().and_then(|t| t.unfocused_text))),
                background: t.background.or(DEFAULT_CONFIG
                    .stackbar
                    .as_ref()
                    .and_then(|s| s.tabs.as_ref().and_then(|t| t.background))),
                font_family: t.font_family.as_ref().cloned().or(DEFAULT_CONFIG
                    .stackbar
                    .as_ref()
                    .and_then(|s| s.tabs.as_ref().and_then(|t| t.font_family.clone()))),
                font_size: t.font_size.or(DEFAULT_CONFIG
                    .stackbar
                    .as_ref()
                    .and_then(|s| s.tabs.as_ref().and_then(|t| t.font_size))),
            }),
        }),
        animation: config.animation.map(|a| AnimationsConfig {
            enabled: a.enabled,
            duration: a.duration.or(DEFAULT_CONFIG
                .animation
                .as_ref()
                .and_then(|a| a.duration.clone())),
            style: a.style.or(DEFAULT_CONFIG
                .animation
                .as_ref()
                .and_then(|a| a.style.clone())),
            fps: a
                .fps
                .or(DEFAULT_CONFIG.animation.as_ref().and_then(|a| a.fps)),
        }),
        theme: config
            .theme
            .map(|t| match t {
                KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                } => {
                    if let KomorebiTheme::Catppuccin {
                        name: _,
                        single_border: d_single_border,
                        stack_border: d_stack_border,
                        monocle_border: d_monocle_border,
                        floating_border: d_floating_border,
                        unfocused_border: d_unfocused_border,
                        stackbar_focused_text: d_stackbar_focused_text,
                        stackbar_unfocused_text: d_stackbar_unfocused_text,
                        stackbar_background: d_stackbar_background,
                        bar_accent: d_bar_accent,
                    } = *DEFAULT_CATPPUCCIN_THEME
                    {
                        KomorebiTheme::Catppuccin {
                            name,
                            single_border: single_border.or(d_single_border),
                            stack_border: stack_border.or(d_stack_border),
                            monocle_border: monocle_border.or(d_monocle_border),
                            floating_border: floating_border.or(d_floating_border),
                            unfocused_border: unfocused_border.or(d_unfocused_border),
                            stackbar_focused_text: stackbar_focused_text
                                .or(d_stackbar_focused_text),
                            stackbar_unfocused_text: stackbar_unfocused_text
                                .or(d_stackbar_unfocused_text),
                            stackbar_background: stackbar_background.or(d_stackbar_background),
                            bar_accent: bar_accent.or(d_bar_accent),
                        }
                    } else {
                        KomorebiTheme::Catppuccin {
                            name,
                            single_border,
                            stack_border,
                            monocle_border,
                            floating_border,
                            unfocused_border,
                            stackbar_focused_text,
                            stackbar_unfocused_text,
                            stackbar_background,
                            bar_accent,
                        }
                    }
                }
                KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                } => {
                    if let KomorebiTheme::Base16 {
                        name: _,
                        single_border: d_single_border,
                        stack_border: d_stack_border,
                        monocle_border: d_monocle_border,
                        floating_border: d_floating_border,
                        unfocused_border: d_unfocused_border,
                        stackbar_focused_text: d_stackbar_focused_text,
                        stackbar_unfocused_text: d_stackbar_unfocused_text,
                        stackbar_background: d_stackbar_background,
                        bar_accent: d_bar_accent,
                    } = *DEFAULT_BASE16_THEME
                    {
                        KomorebiTheme::Base16 {
                            name,
                            single_border: single_border.or(d_single_border),
                            stack_border: stack_border.or(d_stack_border),
                            monocle_border: monocle_border.or(d_monocle_border),
                            floating_border: floating_border.or(d_floating_border),
                            unfocused_border: unfocused_border.or(d_unfocused_border),
                            stackbar_focused_text: stackbar_focused_text
                                .or(d_stackbar_focused_text),
                            stackbar_unfocused_text: stackbar_unfocused_text
                                .or(d_stackbar_unfocused_text),
                            stackbar_background: stackbar_background.or(d_stackbar_background),
                            bar_accent: bar_accent.or(d_bar_accent),
                        }
                    } else {
                        KomorebiTheme::Base16 {
                            name,
                            single_border,
                            stack_border,
                            monocle_border,
                            floating_border,
                            unfocused_border,
                            stackbar_focused_text,
                            stackbar_unfocused_text,
                            stackbar_background,
                            bar_accent,
                        }
                    }
                }
            })
            .or(DEFAULT_CONFIG.theme),
        slow_application_identifiers: config
            .slow_application_identifiers
            .or(DEFAULT_CONFIG.slow_application_identifiers.clone()),
        slow_application_compensation_time: config
            .slow_application_compensation_time
            .or(DEFAULT_CONFIG.slow_application_compensation_time),
        bar_configurations: config
            .bar_configurations
            .or(DEFAULT_CONFIG.bar_configurations.clone()),
        remove_titlebar_applications: config
            .remove_titlebar_applications
            .or(DEFAULT_CONFIG.remove_titlebar_applications.clone()),
    }
}

/// Unmerge the `DEFAULT_CONFIG` values from `config`. For each value that is equal to
/// `DEFAULT_CONFIG` on `config` it changes it to `None`, this way we simplify the `config` so that
/// when written to file it will only have the necessary lines.
/// It returns a new `StaticConfig` with the result.
pub fn unmerge_default(config: StaticConfig) -> StaticConfig {
    StaticConfig {
        invisible_borders: config
            .invisible_borders
            .and_then(|v| (DEFAULT_CONFIG.invisible_borders != Some(v)).then_some(v)),
        minimum_window_width: config
            .minimum_window_width
            .and_then(|v| (DEFAULT_CONFIG.minimum_window_width != Some(v)).then_some(v)),
        minimum_window_height: config
            .minimum_window_height
            .and_then(|v| (DEFAULT_CONFIG.minimum_window_height != Some(v)).then_some(v)),
        resize_delta: config
            .resize_delta
            .and_then(|v| (DEFAULT_CONFIG.resize_delta != Some(v)).then_some(v)),
        window_container_behaviour: config
            .window_container_behaviour
            .and_then(|v| (DEFAULT_CONFIG.window_container_behaviour != Some(v)).then_some(v)),
        float_override: config
            .float_override
            .and_then(|v| (DEFAULT_CONFIG.float_override != Some(v)).then_some(v)),
        cross_monitor_move_behaviour: config
            .cross_monitor_move_behaviour
            .and_then(|v| (DEFAULT_CONFIG.cross_monitor_move_behaviour != Some(v)).then_some(v)),
        cross_boundary_behaviour: config
            .cross_boundary_behaviour
            .and_then(|v| (DEFAULT_CONFIG.cross_boundary_behaviour != Some(v)).then_some(v)),
        unmanaged_window_operation_behaviour: config.unmanaged_window_operation_behaviour.and_then(
            |v| (DEFAULT_CONFIG.unmanaged_window_operation_behaviour != Some(v)).then_some(v),
        ),
        focus_follows_mouse: config
            .focus_follows_mouse
            .and_then(|v| (DEFAULT_CONFIG.focus_follows_mouse != Some(v)).then_some(v)),
        mouse_follows_focus: config
            .mouse_follows_focus
            .and_then(|v| (DEFAULT_CONFIG.mouse_follows_focus != Some(v)).then_some(v)),
        app_specific_configuration_path: config.app_specific_configuration_path.and_then(|v| {
            (DEFAULT_CONFIG.app_specific_configuration_path.as_ref() != Some(&v)).then_some(v)
        }),
        border_width: config
            .border_width
            .and_then(|v| (DEFAULT_CONFIG.border_width != Some(v)).then_some(v)),
        border_offset: config
            .border_offset
            .and_then(|v| (DEFAULT_CONFIG.border_offset != Some(v)).then_some(v)),
        border: config
            .border
            .and_then(|v| (DEFAULT_CONFIG.border != Some(v)).then_some(v)),
        border_colours: config.border_colours.map(|bc| BorderColours {
            single: bc.single.and_then(|v| {
                (DEFAULT_CONFIG
                    .border_colours
                    .as_ref()
                    .and_then(|bc| bc.single)
                    != Some(v))
                .then_some(v)
            }),
            stack: bc.stack.and_then(|v| {
                (DEFAULT_CONFIG
                    .border_colours
                    .as_ref()
                    .and_then(|bc| bc.stack)
                    != Some(v))
                .then_some(v)
            }),
            monocle: bc.monocle.and_then(|v| {
                (DEFAULT_CONFIG
                    .border_colours
                    .as_ref()
                    .and_then(|bc| bc.monocle)
                    != Some(v))
                .then_some(v)
            }),
            floating: bc.floating.and_then(|v| {
                (DEFAULT_CONFIG
                    .border_colours
                    .as_ref()
                    .and_then(|bc| bc.floating)
                    != Some(v))
                .then_some(v)
            }),
            unfocused: bc.unfocused.and_then(|v| {
                (DEFAULT_CONFIG
                    .border_colours
                    .as_ref()
                    .and_then(|bc| bc.unfocused)
                    != Some(v))
                .then_some(v)
            }),
        }),
        border_style: config
            .border_style
            .and_then(|v| (DEFAULT_CONFIG.border_style != Some(v)).then_some(v)),
        border_z_order: config
            .border_z_order
            .and_then(|v| (DEFAULT_CONFIG.border_z_order != Some(v)).then_some(v)),
        border_implementation: config
            .border_implementation
            .and_then(|v| (DEFAULT_CONFIG.border_implementation != Some(v)).then_some(v)),
        transparency: config
            .transparency
            .and_then(|v| (DEFAULT_CONFIG.transparency != Some(v)).then_some(v)),
        transparency_alpha: config
            .transparency_alpha
            .and_then(|v| (DEFAULT_CONFIG.transparency_alpha != Some(v)).then_some(v)),
        transparency_ignore_rules: config.transparency_ignore_rules.and_then(|v| {
            (DEFAULT_CONFIG.transparency_ignore_rules.as_ref() != Some(&v)).then_some(v)
        }),
        default_workspace_padding: config
            .default_workspace_padding
            .and_then(|v| (DEFAULT_CONFIG.default_workspace_padding != Some(v)).then_some(v)),
        default_container_padding: config
            .default_container_padding
            .and_then(|v| (DEFAULT_CONFIG.default_container_padding != Some(v)).then_some(v)),
        monitors: config.monitors.map(|ms| {
            ms.into_iter()
                .map(|m| MonitorConfig {
                    workspaces: m
                        .workspaces
                        .into_iter()
                        .map(|ws| WorkspaceConfig {
                            name: ws.name,
                            layout: ws.layout,
                            custom_layout: ws.custom_layout.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.custom_layout.as_ref() != Some(&v))
                                    .then_some(v)
                            }),
                            layout_rules: ws.layout_rules.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.layout_rules.as_ref() != Some(&v))
                                    .then_some(v)
                            }),
                            custom_layout_rules: ws.custom_layout_rules.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.custom_layout_rules.as_ref() != Some(&v))
                                    .then_some(v)
                            }),
                            container_padding: ws.container_padding.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.container_padding != Some(v)).then_some(v)
                            }),
                            workspace_padding: ws.workspace_padding.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.workspace_padding != Some(v)).then_some(v)
                            }),
                            initial_workspace_rules: ws.initial_workspace_rules.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.initial_workspace_rules.as_ref()
                                    != Some(&v))
                                .then_some(v)
                            }),
                            workspace_rules: ws.workspace_rules.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.workspace_rules.as_ref() != Some(&v))
                                    .then_some(v)
                            }),
                            apply_window_based_work_area_offset: ws
                                .apply_window_based_work_area_offset
                                .and_then(|v| {
                                    (DEFAULT_WORKSPACE_CONFIG.apply_window_based_work_area_offset
                                        != Some(v))
                                    .then_some(v)
                                }),
                            window_container_behaviour: ws.window_container_behaviour.and_then(
                                |v| {
                                    (DEFAULT_WORKSPACE_CONFIG.window_container_behaviour != Some(v))
                                        .then_some(v)
                                },
                            ),
                            float_override: ws.float_override.and_then(|v| {
                                (DEFAULT_WORKSPACE_CONFIG.float_override != Some(v)).then_some(v)
                            }),
                        })
                        .collect(),
                    work_area_offset: m.work_area_offset.and_then(|v| {
                        (DEFAULT_MONITOR_CONFIG.work_area_offset != Some(v)).then_some(v)
                    }),
                    window_based_work_area_offset: m.window_based_work_area_offset.and_then(|v| {
                        (DEFAULT_MONITOR_CONFIG.window_based_work_area_offset != Some(v))
                            .then_some(v)
                    }),
                    window_based_work_area_offset_limit: m
                        .window_based_work_area_offset_limit
                        .and_then(|v| {
                            (DEFAULT_MONITOR_CONFIG.window_based_work_area_offset_limit != Some(v))
                                .then_some(v)
                        }),
                })
                .collect()
        }),
        window_hiding_behaviour: config
            .window_hiding_behaviour
            .and_then(|v| (DEFAULT_CONFIG.window_hiding_behaviour != Some(v)).then_some(v)),
        global_work_area_offset: config
            .global_work_area_offset
            .and_then(|v| (DEFAULT_CONFIG.global_work_area_offset != Some(v)).then_some(v)),
        ignore_rules: config
            .ignore_rules
            .and_then(|v| (DEFAULT_CONFIG.ignore_rules.as_ref() != Some(&v)).then_some(v)),
        manage_rules: config
            .manage_rules
            .and_then(|v| (DEFAULT_CONFIG.manage_rules.as_ref() != Some(&v)).then_some(v)),
        floating_applications: config
            .floating_applications
            .and_then(|v| (DEFAULT_CONFIG.floating_applications.as_ref() != Some(&v)).then_some(v)),
        border_overflow_applications: config.border_overflow_applications.and_then(|v| {
            (DEFAULT_CONFIG.border_overflow_applications.as_ref() != Some(&v)).then_some(v)
        }),
        tray_and_multi_window_applications: config.tray_and_multi_window_applications.and_then(
            |v| {
                (DEFAULT_CONFIG.tray_and_multi_window_applications.as_ref() != Some(&v))
                    .then_some(v)
            },
        ),
        layered_applications: config
            .layered_applications
            .and_then(|v| (DEFAULT_CONFIG.layered_applications.as_ref() != Some(&v)).then_some(v)),
        object_name_change_applications: config.object_name_change_applications.and_then(|v| {
            (DEFAULT_CONFIG.object_name_change_applications.as_ref() != Some(&v)).then_some(v)
        }),
        monitor_index_preferences: config.monitor_index_preferences.and_then(|v| {
            (DEFAULT_CONFIG.monitor_index_preferences.as_ref() != Some(&v)).then_some(v)
        }),
        display_index_preferences: config.display_index_preferences.and_then(|v| {
            (DEFAULT_CONFIG.display_index_preferences.as_ref() != Some(&v)).then_some(v)
        }),
        stackbar: config.stackbar.map(|s| StackbarConfig {
            height: s.height.and_then(|v| {
                (DEFAULT_CONFIG.stackbar.as_ref().and_then(|s| s.height) != Some(v)).then_some(v)
            }),
            label: s.label.and_then(|v| {
                (DEFAULT_CONFIG.stackbar.as_ref().and_then(|s| s.label) != Some(v)).then_some(v)
            }),
            mode: s.mode.and_then(|v| {
                (DEFAULT_CONFIG.stackbar.as_ref().and_then(|s| s.mode) != Some(v)).then_some(v)
            }),
            tabs: s.tabs.map(|t| TabsConfig {
                width: t.width.and_then(|v| {
                    (DEFAULT_CONFIG
                        .stackbar
                        .as_ref()
                        .and_then(|s| s.tabs.as_ref().and_then(|t| t.width))
                        != Some(v))
                    .then_some(v)
                }),
                focused_text: t.focused_text.and_then(|v| {
                    (DEFAULT_CONFIG
                        .stackbar
                        .as_ref()
                        .and_then(|s| s.tabs.as_ref().and_then(|t| t.focused_text))
                        != Some(v))
                    .then_some(v)
                }),
                unfocused_text: t.unfocused_text.and_then(|v| {
                    (DEFAULT_CONFIG
                        .stackbar
                        .as_ref()
                        .and_then(|s| s.tabs.as_ref().and_then(|t| t.unfocused_text))
                        != Some(v))
                    .then_some(v)
                }),
                background: t.background.and_then(|v| {
                    (DEFAULT_CONFIG
                        .stackbar
                        .as_ref()
                        .and_then(|s| s.tabs.as_ref().and_then(|t| t.background))
                        != Some(v))
                    .then_some(v)
                }),
                font_family: t.font_family.and_then(|v| {
                    (DEFAULT_CONFIG
                        .stackbar
                        .as_ref()
                        .and_then(|s| s.tabs.as_ref().and_then(|t| t.font_family.as_ref()))
                        != Some(&v))
                    .then_some(v)
                }),
                font_size: t.font_size.and_then(|v| {
                    (DEFAULT_CONFIG
                        .stackbar
                        .as_ref()
                        .and_then(|s| s.tabs.as_ref().and_then(|t| t.font_size))
                        != Some(v))
                    .then_some(v)
                }),
            }),
        }),
        animation: config.animation.map(|a| AnimationsConfig {
            enabled: a.enabled,
            duration: a.duration.and_then(|v| {
                (DEFAULT_CONFIG
                    .animation
                    .as_ref()
                    .and_then(|a| a.duration.as_ref())
                    != Some(&v))
                .then_some(v)
            }),
            style: a.style.and_then(|v| {
                (DEFAULT_CONFIG
                    .animation
                    .as_ref()
                    .and_then(|a| a.style.as_ref())
                    != Some(&v))
                .then_some(v)
            }),
            fps: a.fps.and_then(|v| {
                (DEFAULT_CONFIG
                    .animation
                    .as_ref()
                    .and_then(|a| a.fps.as_ref())
                    != Some(&v))
                .then_some(v)
            }),
        }),
        theme: config.theme.map(|v| match v {
            KomorebiTheme::Catppuccin {
                name,
                single_border,
                stack_border,
                monocle_border,
                floating_border,
                unfocused_border,
                stackbar_focused_text,
                stackbar_unfocused_text,
                stackbar_background,
                bar_accent,
            } => {
                if let KomorebiTheme::Catppuccin {
                    name: _,
                    single_border: d_single_border,
                    stack_border: d_stack_border,
                    monocle_border: d_monocle_border,
                    floating_border: d_floating_border,
                    unfocused_border: d_unfocused_border,
                    stackbar_focused_text: d_stackbar_focused_text,
                    stackbar_unfocused_text: d_stackbar_unfocused_text,
                    stackbar_background: d_stackbar_background,
                    bar_accent: d_bar_accent,
                } = *DEFAULT_CATPPUCCIN_THEME
                {
                    KomorebiTheme::Catppuccin {
                        name,
                        single_border: single_border.and_then(|v| (Some(v) != d_single_border).then_some(v)),
                        stack_border: stack_border.and_then(|v| (Some(v) != d_stack_border).then_some(v)),
                        monocle_border: monocle_border.and_then(|v| (Some(v) != d_monocle_border).then_some(v)),
                        floating_border: floating_border.and_then(|v| (Some(v) != d_floating_border).then_some(v)),
                        unfocused_border: unfocused_border.and_then(|v| (Some(v) != d_unfocused_border).then_some(v)),
                        stackbar_focused_text: stackbar_focused_text.and_then(|v| (Some(v) != d_stackbar_focused_text).then_some(v)),
                        stackbar_unfocused_text: stackbar_unfocused_text.and_then(|v| (Some(v) != d_stackbar_unfocused_text).then_some(v)),
                        stackbar_background: stackbar_background.and_then(|v| (Some(v) != d_stackbar_background).then_some(v)),
                        bar_accent: bar_accent.and_then(|v| (Some(v) != d_bar_accent).then_some(v)),
                    }
                } else {
                    KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    }
                }
            }
            KomorebiTheme::Base16 {
                name,
                single_border,
                stack_border,
                monocle_border,
                floating_border,
                unfocused_border,
                stackbar_focused_text,
                stackbar_unfocused_text,
                stackbar_background,
                bar_accent,
            } => {
                if let KomorebiTheme::Base16 {
                    name: _,
                    single_border: d_single_border,
                    stack_border: d_stack_border,
                    monocle_border: d_monocle_border,
                    floating_border: d_floating_border,
                    unfocused_border: d_unfocused_border,
                    stackbar_focused_text: d_stackbar_focused_text,
                    stackbar_unfocused_text: d_stackbar_unfocused_text,
                    stackbar_background: d_stackbar_background,
                    bar_accent: d_bar_accent,
                } = *DEFAULT_BASE16_THEME
                {
                    KomorebiTheme::Base16 {
                        name,
                        single_border: single_border.and_then(|v| (Some(v) != d_single_border).then_some(v)),
                        stack_border: stack_border.and_then(|v| (Some(v) != d_stack_border).then_some(v)),
                        monocle_border: monocle_border.and_then(|v| (Some(v) != d_monocle_border).then_some(v)),
                        floating_border: floating_border.and_then(|v| (Some(v) != d_floating_border).then_some(v)),
                        unfocused_border: unfocused_border.and_then(|v| (Some(v) != d_unfocused_border).then_some(v)),
                        stackbar_focused_text: stackbar_focused_text.and_then(|v| (Some(v) != d_stackbar_focused_text).then_some(v)),
                        stackbar_unfocused_text: stackbar_unfocused_text.and_then(|v| (Some(v) != d_stackbar_unfocused_text).then_some(v)),
                        stackbar_background: stackbar_background.and_then(|v| (Some(v) != d_stackbar_background).then_some(v)),
                        bar_accent: bar_accent.and_then(|v| (Some(v) != d_bar_accent).then_some(v)),
                    }
                } else {
                    KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    }
                }
            },
        }).and_then(|v| (DEFAULT_CONFIG.theme != Some(v)).then_some(v)),
        slow_application_identifiers: config.slow_application_identifiers.and_then(|v| {
            (DEFAULT_CONFIG.slow_application_identifiers.as_ref() != Some(&v)).then_some(v)
        }),
        slow_application_compensation_time: config.slow_application_compensation_time.and_then(
            |v| (DEFAULT_CONFIG.slow_application_compensation_time != Some(v)).then_some(v),
        ),
        bar_configurations: config
            .bar_configurations
            .and_then(|v| (DEFAULT_CONFIG.bar_configurations.as_ref() != Some(&v)).then_some(v)),
        remove_titlebar_applications: config
            .remove_titlebar_applications
            .and_then(|v| (DEFAULT_CONFIG.remove_titlebar_applications.as_ref() != Some(&v)).then_some(v)),
    }
}

pub trait ChangeConfig {
    fn set_optional<T: Clone + PartialEq>(
        &mut self,
        value: Option<T>,
        getter: impl Fn(&mut Self) -> &mut Option<T>,
        getter_ref: impl Fn(&Self) -> &Option<T>,
    );

    fn change_config(&mut self, f: impl Fn(&mut Self)) {
        f(self);
    }

    fn change_monitor_config(&mut self, idx: usize, f: impl Fn(&mut MonitorConfig));

    fn change_workspace_config(
        &mut self,
        monitor_idx: usize,
        workspace_idx: usize,
        f: impl Fn(&mut WorkspaceConfig),
    );
}

impl ChangeConfig for StaticConfig {
    fn set_optional<T: Clone + PartialEq>(
        &mut self,
        value: Option<T>,
        getter: impl Fn(&mut Self) -> &mut Option<T>,
        getter_ref: impl Fn(&Self) -> &Option<T>,
    ) {
        let sanitized_value = sanitize_value(value, getter_ref);
        *getter(self) = sanitized_value;
    }

    fn change_monitor_config(&mut self, idx: usize, f: impl Fn(&mut MonitorConfig)) {
        if let Some(monitors) = &mut self.monitors {
            if let Some(monitor) = monitors.get_mut(idx) {
                f(monitor);
            } else {
                monitors.reserve(idx + 1 - monitors.len());
                for _ in monitors.len()..(idx + 1) {
                    monitors.push(MonitorConfig {
                        workspaces: Vec::new(),
                        work_area_offset: None,
                        window_based_work_area_offset: None,
                        window_based_work_area_offset_limit: None,
                    });
                }
                f(&mut monitors[idx]);
            }
        } else {
            let mut monitors = vec![
                komorebi::MonitorConfig {
                    workspaces: Vec::new(),
                    work_area_offset: None,
                    window_based_work_area_offset: None,
                    window_based_work_area_offset_limit: None,
                };
                idx + 1
            ];
            f(&mut monitors[idx]);
            self.monitors = Some(monitors);
        }
    }

    fn change_workspace_config(
        &mut self,
        monitor_idx: usize,
        workspace_idx: usize,
        f: impl Fn(&mut WorkspaceConfig),
    ) {
        self.change_monitor_config(monitor_idx, |monitor| {
            if let Some(workspace) = monitor.workspaces.get_mut(workspace_idx) {
                f(workspace);
            } else {
                monitor
                    .workspaces
                    .reserve(workspace_idx + 1 - monitor.workspaces.len());
                for _ in monitor.workspaces.len()..(workspace_idx + 1) {
                    monitor.workspaces.push(WorkspaceConfig {
                        name: String::default(),
                        layout: None,
                        custom_layout: None,
                        layout_rules: None,
                        custom_layout_rules: None,
                        container_padding: None,
                        workspace_padding: None,
                        initial_workspace_rules: None,
                        workspace_rules: None,
                        apply_window_based_work_area_offset: None,
                        window_container_behaviour: None,
                        float_override: None,
                    });
                }
                f(&mut monitor.workspaces[workspace_idx]);
            }
        });
    }
}

enum State {
    Starting,
    Ready(Data),
}

struct Data {
    debouncer: Debouncer<ReadDirectoryChangesWatcher>,
    receiver: Receiver<Input>,
    ignore_event: usize,
}

pub enum Input {
    IgnoreNextEvent,
    DebouncerRes(DebounceEventResult),
}

pub fn worker() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(10, move |mut output| async move {
            let mut state = State::Starting;

            loop {
                match state {
                    State::Starting => {
                        let (sender, receiver) = channel::bounded(10);

                        let sender_clone = sender.clone();
                        match output
                            .send(Message::ConfigFileWatcherTx(sender_clone))
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error trying to send the options watcher sender:\n{e:?}");
                            }
                        }

                        let mut debouncer = new_debouncer(Duration::from_millis(250), move |res| {
                            async_std::task::block_on(async {
                                let input = Input::DebouncerRes(res);
                                match sender.send(input).await {
                                    Ok(_) => {}
                                    Err(error) => {
                                        println!(
                                            "Error sending a debounced \
                                                event to the worker channel. \
                                                E: {error:?}"
                                        );
                                    }
                                }
                            })
                        })
                        .unwrap();

                        debouncer
                            .watcher()
                            .watch(&config_path(), RecursiveMode::NonRecursive)
                            .unwrap();

                        state = State::Ready(Data {
                            debouncer,
                            receiver,
                            ignore_event: 0,
                        });
                    }
                    State::Ready(data) => {
                        let Data {
                            mut receiver,
                            debouncer,
                            mut ignore_event,
                        } = data;
                        let input = receiver.select_next_some().await;

                        match input {
                            Input::IgnoreNextEvent => {
                                println!("IgnoreNextEvent");
                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event: ignore_event + 1,
                                });
                            }
                            Input::DebouncerRes(res) => {
                                match res {
                                    Ok(events) => {
                                        events.iter().for_each(|event| {
                                            handle_event(event, &mut ignore_event, &mut output);
                                        });
                                    }
                                    Err(error) => {
                                        println!("Error from file watcher: {error:?}")
                                    }
                                }

                                state = State::Ready(Data {
                                    debouncer,
                                    receiver,
                                    ignore_event,
                                });
                            }
                        }
                    }
                }
            }
        })
    })
}

fn handle_event(
    event: &DebouncedEvent,
    ignore_event: &mut usize,
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
) {
    // println!("FileWatcher event: {event:?}");
    if let DebouncedEventKind::Any = event.kind {
        if *ignore_event == 0 {
            println!("FileWatcher: loading options");
            async_std::task::block_on(async {
                match load().await {
                    Ok(loaded_options) => {
                        let _ = output
                            .send(Message::LoadedConfig(Arc::new(loaded_options)))
                            .await;
                    }
                    Err(e) => {
                        let _ = output.send(Message::AppError(e)).await;
                    }
                }
            });
        } else {
            println!("FileWatcher: ignoring event");
            *ignore_event = ignore_event.saturating_sub(1);
        }
    }
}

pub async fn load() -> Result<StaticConfig, AppError> {
    use async_std::prelude::*;

    let mut contents = String::new();

    let file_open_res = async_std::fs::File::open(config_path()).await;

    let mut file = match file_open_res {
        Ok(file) => file,
        Err(error) => {
            println!("Failed to find 'komorebi.json' file.\nError: {}", error);
            return Err(AppError {
                title: "Failed to find 'komorebi.json' file.".into(),
                description: None,
                kind: AppErrorKind::Info,
            });
        }
    };

    file.read_to_string(&mut contents)
        .await
        .map_err(|e| AppError {
            title: "Error opening 'komorebi.json' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    serde_json::from_str(&contents).map_err(|e| AppError {
        title: "Error reading 'komorebi.json' file.".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })
}

pub async fn save(config: StaticConfig) -> Result<(), AppError> {
    use async_std::prelude::*;

    let unmerged_config = config::unmerge_default(config);
    let json = serde_json::to_string_pretty(&unmerged_config).map_err(|e| AppError {
        title: "Error writing to 'komorebi.json' file".into(),
        description: Some(e.to_string()),
        kind: AppErrorKind::Error,
    })?;

    let path = config_path();

    // if let Some(dir) = path.parent() {
    //     async_std::fs::create_dir_all(dir)
    //         .await
    //         .map_err(|e| AppError {
    //             title: "Error creating folder for 'komorebi.json' file".into(),
    //             description: Some(e.to_string()),
    //             kind: AppErrorKind::Error,
    //         })?;
    // }

    let mut file = async_std::fs::File::create(path)
        .await
        .map_err(|e| AppError {
            title: "Error creating 'komorebi.json' file.".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    file.write_all(json.as_bytes())
        .await
        .map_err(|e| AppError {
            title: "Error saving 'komorebi.json' file".into(),
            description: Some(e.to_string()),
            kind: AppErrorKind::Error,
        })?;

    // This is a simple way to save at most once every couple seconds
    // async_std::task::sleep(std::time::Duration::from_secs(2)).await;

    Ok(())
}

pub fn config_path() -> std::path::PathBuf {
    let home_dir: std::path::PathBuf = std::env::var("KOMOREBI_CONFIG_HOME").map_or_else(
        |_| dirs::home_dir().expect("there is no home directory"),
        |home_path| {
            let home = std::path::PathBuf::from(&home_path);

            if home.as_path().is_dir() {
                home
            } else {
                panic!("$Env:KOMOREBI_CONFIG_HOME is set to '{home_path}', which is not a valid directory");
            }
        },
    );

    home_dir.join("komorebi.json")
}
