use super::rule;

use crate::config::{DEFAULT_CONFIG, DEFAULT_WORKSPACE_CONFIG};
use crate::komo_interop::layout::{
    LAYOUT_FLIP_OPTIONS, LAYOUT_OPTIONS, LAYOUT_OPTIONS_WITHOUT_NONE, Layout,
};
use crate::screen::{
    View,
    wallpaper::{self, WallpaperScreen},
};
use crate::utils::{DisplayOption, DisplayOptionCustom};
use crate::widget::opt_helpers::description_text as t;
use crate::widget::opt_helpers::to_description_text as td;
use crate::widget::{ICONS, icons, opt_helpers};

use std::collections::{BTreeMap, HashMap};

use iced::widget::{
    Id, button, column, container,
    operation::{self, AbsoluteOffset},
    pick_list, row, rule as ruler, space, text,
};
use iced::{Center, Element, Fill, Subscription, Task};
use komorebi_client::{
    Axis, DefaultLayout, FloatingLayerBehaviour, GridLayoutOptions, LayoutOptions, MatchingRule,
    Rect, ScrollingLayoutOptions, Wallpaper, WindowContainerBehaviour, WorkspaceConfig,
};

#[derive(Clone, Debug)]
pub enum Message {
    SetScreen(Screen),
    ConfigChange(ConfigChange),
    ToggleOverrideGlobal(OverrideConfig),
    ChangeNewLayoutRuleLimit(usize),
    ChangeNewLayoutRuleLayout(Layout),
    AddNewLayoutRule,
    RemoveLayoutRule(usize),
    ChangeNewBehaviourRuleLimit(usize),
    ChangeNewBehaviourRuleBehaviour(WindowContainerBehaviour),
    AddNewBehaviourRule,
    RemoveBehaviourRule(usize),
    Rule(rule::Message),
    Wallpaper(wallpaper::Message),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    ScreenChange(Screen),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    ApplyWindowBasedWorkAreaOffset(Option<bool>),
    ContainerPadding(Option<i32>),
    FloatOverride(Option<bool>),
    Layout(Option<Layout>),
    LayoutFlip(Option<Axis>),
    LayoutRules(Option<HashMap<usize, DefaultLayout>>),
    LayoutRuleLimit((usize, usize)),
    LayoutRuleLayout((usize, Layout)),
    Name(String),
    WindowContainerBehaviour(Option<WindowContainerBehaviour>),
    BehaviourRules(Option<HashMap<usize, WindowContainerBehaviour>>),
    BehaviourRuleLimit((usize, usize)),
    BehaviourRuleBehaviour((usize, WindowContainerBehaviour)),
    WorkspacePadding(Option<i32>),
    FloatingLayerBehaviour(Option<FloatingLayerBehaviour>),
    Tile(Option<bool>),
    WorkAreaOffset(Option<Rect>),
    WorkAreaOffsetTop(i32),
    WorkAreaOffsetBottom(i32),
    WorkAreaOffsetLeft(i32),
    WorkAreaOffsetRight(i32),
    GridOptionRows(Option<usize>),
    ScrollingOptionColumns(Option<usize>),
    ScrollingOptionCenter(Option<bool>),
    Wallpaper(Option<Wallpaper>),
}

#[derive(Clone, Debug)]
pub enum OverrideConfig {
    ContainerPadding(bool),
    FloatOverride(bool),
    WindowContainerBehaviour(bool),
    WorkspacePadding(bool),
    FloatingLayerBehaviour(bool),
    WorkAreaOffset(bool),
}

#[derive(Clone, Debug, Default)]
pub enum Screen {
    #[default]
    Workspace,
    WorkspaceWallpaper,
    WorkspaceRules,
    InitialWorkspaceRules,
}

#[derive(Clone, Debug, Default)]
pub struct Workspace {
    pub index: usize,
    pub screen: Screen,
    pub rule: rule::Rule,
    pub wallpaper: WallpaperScreen,
    pub new_layout_rule_limit: usize,
    pub new_layout_rule_layout: Layout,
    pub new_behaviour_rule_limit: usize,
    pub new_behaviour_rule_behaviour: WindowContainerBehaviour,
}

pub trait WorkspaceScreen {
    fn update(&mut self, workspace: &mut Workspace, message: Message) -> (Action, Task<Message>);

    fn view<'a>(&'a self, workspace: &'a Workspace) -> View<'a, Message>;
}

impl WorkspaceScreen for WorkspaceConfig {
    fn update(&mut self, workspace: &mut Workspace, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::SetScreen(screen) => {
                if matches!(
                    screen,
                    Screen::WorkspaceRules | Screen::InitialWorkspaceRules
                ) {
                    workspace.rule = rule::Rule::new();
                }
                workspace.screen = screen.clone();
                let task = operation::scroll_to(
                    Id::new("monitors_scrollable"),
                    AbsoluteOffset { x: 0.0, y: 0.0 },
                );
                return (Action::ScreenChange(screen), task);
            }
            Message::ConfigChange(change) => match change {
                ConfigChange::ApplyWindowBasedWorkAreaOffset(value) => {
                    self.apply_window_based_work_area_offset = value
                }
                ConfigChange::ContainerPadding(value) => self.container_padding = value,
                ConfigChange::FloatOverride(value) => self.float_override = value,
                ConfigChange::Layout(value) => self.layout = value.map(Into::into),
                ConfigChange::LayoutFlip(value) => self.layout_flip = value,
                ConfigChange::LayoutRules(value) => {
                    self.layout_rules = value;
                }
                ConfigChange::LayoutRuleLimit((previous_limit, new_limit)) => {
                    if let Some(layout_rules) = self.layout_rules.as_mut()
                        && !layout_rules.contains_key(&new_limit)
                        && let Some(layout) = layout_rules.remove(&previous_limit)
                    {
                        layout_rules.insert(new_limit, layout);
                    }
                }
                ConfigChange::LayoutRuleLayout((limit, new_layout)) => {
                    if let Some(layout_rules) = &mut self.layout_rules {
                        let rule_layout = layout_rules.entry(limit).or_insert(DefaultLayout::BSP);
                        *rule_layout = new_layout.into();
                    }
                }
                ConfigChange::Name(value) => self.name = value,
                ConfigChange::WindowContainerBehaviour(value) => {
                    self.window_container_behaviour = value;
                }
                ConfigChange::BehaviourRules(value) => {
                    self.window_container_behaviour_rules = value;
                }
                ConfigChange::BehaviourRuleLimit((previous_limit, new_limit)) => {
                    if let Some(behaviour_rules) = self.window_container_behaviour_rules.as_mut()
                        && !behaviour_rules.contains_key(&new_limit)
                        && let Some(layout) = behaviour_rules.remove(&previous_limit)
                    {
                        behaviour_rules.insert(new_limit, layout);
                    }
                }
                ConfigChange::BehaviourRuleBehaviour((limit, new_behaviour)) => {
                    if let Some(behaviour_rules) = &mut self.window_container_behaviour_rules {
                        let rule_behaviour = behaviour_rules
                            .entry(limit)
                            .or_insert(WindowContainerBehaviour::Create);
                        *rule_behaviour = new_behaviour;
                    }
                }
                ConfigChange::WorkspacePadding(value) => self.workspace_padding = value,
                ConfigChange::FloatingLayerBehaviour(value) => {
                    self.floating_layer_behaviour = value;
                }
                ConfigChange::WorkAreaOffset(value) => self.work_area_offset = value,
                ConfigChange::WorkAreaOffsetTop(value) => {
                    if let Some(offset) = &mut self.work_area_offset {
                        offset.top = value;
                    } else {
                        self.work_area_offset = Some(Rect {
                            left: 0,
                            top: value,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetBottom(value) => {
                    if let Some(offset) = &mut self.work_area_offset {
                        offset.bottom = value;
                    } else {
                        self.work_area_offset = Some(Rect {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: value,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetRight(value) => {
                    if let Some(offset) = &mut self.work_area_offset {
                        offset.right = value;
                    } else {
                        self.work_area_offset = Some(Rect {
                            left: 0,
                            top: 0,
                            right: value,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::WorkAreaOffsetLeft(value) => {
                    if let Some(offset) = &mut self.work_area_offset {
                        offset.left = value;
                    } else {
                        self.work_area_offset = Some(Rect {
                            left: value,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        });
                    }
                }
                ConfigChange::Tile(value) => self.tile = value,
                ConfigChange::GridOptionRows(value) => {
                    if let Some(layout_options) = self.layout_options.as_mut() {
                        if let Some(grid_options) = layout_options.grid.as_mut() {
                            if let Some(rows) = value {
                                grid_options.rows = rows;
                            } else {
                                layout_options.grid = None;
                                if layout_options.scrolling.is_none() {
                                    self.layout_options = None;
                                }
                            }
                        } else if let Some(rows) = value {
                            layout_options.grid = Some(GridLayoutOptions { rows });
                        } else if layout_options.scrolling.is_none() {
                            self.layout_options = None;
                        }
                    } else if let Some(rows) = value {
                        self.layout_options = Some(LayoutOptions {
                            scrolling: None,
                            grid: Some(GridLayoutOptions { rows }),
                        });
                    }
                }
                ConfigChange::ScrollingOptionColumns(value) => {
                    if let Some(layout_options) = self.layout_options.as_mut() {
                        if let Some(scrolling_options) = layout_options.scrolling.as_mut() {
                            if let Some(columns) = value {
                                scrolling_options.columns = columns;
                            } else {
                                scrolling_options.columns = 3;
                                if scrolling_options.center_focused_column.is_none() {
                                    layout_options.scrolling = None;
                                    if layout_options.grid.is_none() {
                                        self.layout_options = None;
                                    }
                                }
                            }
                        } else if let Some(columns) = value {
                            layout_options.scrolling = Some(ScrollingLayoutOptions {
                                columns,
                                center_focused_column: None,
                            });
                        } else if layout_options.grid.is_none() {
                            self.layout_options = None;
                        }
                    } else if let Some(columns) = value {
                        self.layout_options = Some(LayoutOptions {
                            scrolling: Some(ScrollingLayoutOptions {
                                columns,
                                center_focused_column: None,
                            }),
                            grid: None,
                        });
                    }
                }
                ConfigChange::ScrollingOptionCenter(value) => {
                    if let Some(layout_options) = self.layout_options.as_mut() {
                        if let Some(scrolling_options) = layout_options.scrolling.as_mut() {
                            if value.is_some() {
                                scrolling_options.center_focused_column = value;
                            } else {
                                scrolling_options.center_focused_column = None;
                                if layout_options.grid.is_none() && scrolling_options.columns == 3 {
                                    self.layout_options = None;
                                }
                            }
                        } else if value.is_some() {
                            layout_options.scrolling = Some(ScrollingLayoutOptions {
                                columns: 3,
                                center_focused_column: value,
                            });
                        } else if layout_options.grid.is_none() {
                            self.layout_options = None;
                        }
                    } else if value.is_some() {
                        self.layout_options = Some(LayoutOptions {
                            scrolling: Some(ScrollingLayoutOptions {
                                columns: 3,
                                center_focused_column: value,
                            }),
                            grid: None,
                        });
                    }
                }
                ConfigChange::Wallpaper(wallpaper) => self.wallpaper = wallpaper,
            },
            Message::ToggleOverrideGlobal(to_override) => match to_override {
                OverrideConfig::ContainerPadding(disable) => {
                    if disable {
                        self.container_padding = None;
                    } else {
                        self.container_padding = DEFAULT_CONFIG.default_container_padding;
                    }
                }
                OverrideConfig::FloatOverride(disable) => {
                    if disable {
                        self.float_override = None;
                    } else {
                        self.float_override = DEFAULT_CONFIG.float_override;
                    }
                }
                OverrideConfig::WindowContainerBehaviour(disable) => {
                    if disable {
                        self.window_container_behaviour = None;
                    } else {
                        self.window_container_behaviour = DEFAULT_CONFIG.window_container_behaviour;
                    }
                }
                OverrideConfig::WorkspacePadding(disable) => {
                    if disable {
                        self.workspace_padding = None;
                    } else {
                        self.workspace_padding = DEFAULT_CONFIG.default_workspace_padding;
                    }
                }
                OverrideConfig::FloatingLayerBehaviour(disable) => {
                    if disable {
                        self.floating_layer_behaviour = None;
                    } else {
                        self.floating_layer_behaviour = DEFAULT_CONFIG.floating_layer_behaviour;
                    }
                }
                OverrideConfig::WorkAreaOffset(disable) => {
                    if disable {
                        self.work_area_offset = None;
                    } else {
                        self.work_area_offset = DEFAULT_CONFIG.global_work_area_offset;
                    }
                }
            },
            Message::ChangeNewLayoutRuleLimit(limit) => {
                workspace.new_layout_rule_limit = limit;
            }
            Message::ChangeNewLayoutRuleLayout(layout) => {
                workspace.new_layout_rule_layout = layout;
            }
            Message::AddNewLayoutRule => {
                if let Some(layout_rules) = &mut self.layout_rules {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        layout_rules.entry(workspace.new_layout_rule_limit)
                    {
                        e.insert(workspace.new_layout_rule_layout.into());
                        workspace.new_layout_rule_limit = 0;
                        workspace.new_layout_rule_layout = Layout::BSP;
                    }
                } else {
                    let rules = HashMap::from([(
                        workspace.new_layout_rule_limit,
                        workspace.new_layout_rule_layout.into(),
                    )]);
                    self.layout_rules = Some(rules);
                    workspace.new_layout_rule_limit = 0;
                    workspace.new_layout_rule_layout = Layout::BSP;
                }
            }
            Message::RemoveLayoutRule(limit) => {
                if let Some(layout_rules) = &mut self.layout_rules {
                    layout_rules.remove(&limit);
                }
            }
            Message::ChangeNewBehaviourRuleLimit(limit) => {
                workspace.new_behaviour_rule_limit = limit;
            }
            Message::ChangeNewBehaviourRuleBehaviour(behaviour) => {
                workspace.new_behaviour_rule_behaviour = behaviour;
            }
            Message::AddNewBehaviourRule => {
                if let Some(behaviour_rules) = &mut self.window_container_behaviour_rules {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        behaviour_rules.entry(workspace.new_behaviour_rule_limit)
                    {
                        e.insert(workspace.new_behaviour_rule_behaviour);
                        workspace.new_behaviour_rule_limit = 0;
                        workspace.new_behaviour_rule_behaviour = WindowContainerBehaviour::Create;
                    }
                } else {
                    let rules = HashMap::from([(
                        workspace.new_behaviour_rule_limit,
                        workspace.new_behaviour_rule_behaviour,
                    )]);
                    self.window_container_behaviour_rules = Some(rules);
                    workspace.new_behaviour_rule_limit = 0;
                    workspace.new_behaviour_rule_behaviour = WindowContainerBehaviour::Create;
                }
            }
            Message::RemoveBehaviourRule(limit) => {
                if let Some(behaviour_rules) = &mut self.window_container_behaviour_rules {
                    behaviour_rules.remove(&limit);
                }
            }
            Message::Rule(message) => {
                if matches!(
                    workspace.screen,
                    Screen::WorkspaceRules | Screen::InitialWorkspaceRules
                ) {
                    let rules = get_rules_from_config_mut(self, &workspace.screen);
                    let (action, task) = workspace.rule.update(rules, message);
                    let action_task = match action {
                        rule::Action::None => Task::none(),
                    };
                    return (
                        Action::None,
                        Task::batch([task.map(Message::Rule), action_task]),
                    );
                }
            }
            Message::Wallpaper(message) => {
                if let Some(wp_config) = self.wallpaper.as_mut() {
                    return (
                        Action::None,
                        workspace
                            .wallpaper
                            .update(wp_config, message)
                            .map(Message::Wallpaper),
                    );
                }
            }
        }
        (Action::None, Task::none())
    }

    fn view<'a>(&'a self, workspace: &'a Workspace) -> View<'a, Message> {
        match workspace.screen {
            Screen::Workspace => workspace.workspace_view(self).into(),
            Screen::WorkspaceWallpaper => {
                if let Some(wp_config) = self.wallpaper.as_ref() {
                    workspace.wallpaper.view(wp_config).map(Message::Wallpaper)
                } else {
                    View::new(space())
                }
            }
            Screen::WorkspaceRules | Screen::InitialWorkspaceRules => workspace
                .rule
                .view(get_rules_from_config(self, &workspace.screen))
                .map(Message::Rule)
                .into(),
        }
    }
}

impl Workspace {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }

    fn workspace_view<'a>(&'a self, ws_config: &'a WorkspaceConfig) -> Element<'a, Message> {
        let name = opt_helpers::input(
            "Name",
            Some("Name of the workspace. Should be unique."),
            "",
            &ws_config.name,
            |v| Message::ConfigChange(ConfigChange::Name(v)),
            None,
        );
        let layout = opt_helpers::choose_with_disable_default(
            "Layout",
            Some("Layout (default: BSP)"),
            layout_options_descriptions(),
            &LAYOUT_OPTIONS[..],
            Some(DisplayOptionCustom(
                ws_config.layout.map(Into::into),
                "[None] (Floating)",
            )),
            |s| Message::ConfigChange(ConfigChange::Layout(s.and_then(|s| s.0))),
            Some(DisplayOptionCustom(
                DEFAULT_WORKSPACE_CONFIG.layout.map(Into::into),
                "[None] (Floating)",
            )),
            None,
        );
        let grid_option_rows = matches!(ws_config.layout, Some(DefaultLayout::Grid)).then_some(
            opt_helpers::number_with_disable_default_option(
                "Grid Layout Rows",
                Some(
                    "Maximum number of rows per grid column. (default: None)\n\
            When it's 'None' it will try to fit the amount of containers on the smallest square \
            grid possible.",
                ),
                ws_config
                    .layout_options
                    .and_then(|lo| lo.grid.map(|g| g.rows)),
                DEFAULT_WORKSPACE_CONFIG
                    .layout_options
                    .and_then(|lo| lo.grid.map(|g| g.rows)),
                |v| Message::ConfigChange(ConfigChange::GridOptionRows(v)),
                Some(opt_helpers::DisableArgs {
                    disable: ws_config
                        .layout_options
                        .and_then(|lo| lo.grid.map(|g| g.rows))
                        .is_none(),
                    label: Some("None"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::GridOptionRows((!v).then_some(2)))
                    },
                }),
            ),
        );
        let scrolling_option_columns = matches!(ws_config.layout, Some(DefaultLayout::Scrolling))
            .then_some(opt_helpers::number_with_disable_default_option(
                "Scrolling Layout Columns",
                Some("Desired number of visible columns (default: 3)"),
                ws_config
                    .layout_options
                    .and_then(|lo| lo.scrolling.map(|g| g.columns)),
                DEFAULT_WORKSPACE_CONFIG
                    .layout_options
                    .and_then(|lo| lo.scrolling.map(|g| g.columns)),
                |v| Message::ConfigChange(ConfigChange::ScrollingOptionColumns(v)),
                Some(opt_helpers::DisableArgs {
                    disable: ws_config
                        .layout_options
                        .and_then(|lo| lo.scrolling.map(|g| g.columns))
                        .is_none(),
                    label: Some("None"),
                    on_toggle: |v| {
                        Message::ConfigChange(ConfigChange::ScrollingOptionColumns(
                            (!v).then_some(3),
                        ))
                    },
                }),
            ));
        let scrolling_option_center = matches!(ws_config.layout, Some(DefaultLayout::Scrolling)).then_some(opt_helpers::toggle_with_disable_default(
            "Scrolling Layout Center Focused Column",
            Some(
                "With an odd number of visible columns, keep the focused window column centered (default: false)",
            ),
            ws_config
                .layout_options
                .and_then(|lo| lo.scrolling.and_then(|g| g.center_focused_column)),
            DEFAULT_WORKSPACE_CONFIG
                .layout_options
                .and_then(|lo| lo.scrolling.and_then(|g| g.center_focused_column)),
            |v| Message::ConfigChange(ConfigChange::ScrollingOptionCenter(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config
                    .layout_options
                    .and_then(|lo| lo.scrolling.and_then(|g| g.center_focused_column))
                    .is_none(),
                label: Some("None"),
                on_toggle: |v| {
                    Message::ConfigChange(ConfigChange::ScrollingOptionCenter(
                        (!v).then_some(false),
                    ))
                },
            }),
        ));
        let layout_flip = opt_helpers::choose_with_disable_default(
            "Layout Flip",
            Some("Specify an axis on which to flip the selected layout (default: None)"),
            vec![
                space().into(),
                t("Selected: 'Vertical' -> Flip layout on vertical axis").into(),
                t("Selected: 'Horizontal' -> Flip layout on horizontal axis").into(),
                t("Selected: 'HorizontalAndVertical' -> Flip layout on both axis").into(),
            ],
            &LAYOUT_FLIP_OPTIONS[..],
            Some(DisplayOption(ws_config.layout_flip)),
            |v| Message::ConfigChange(ConfigChange::LayoutFlip(v.and_then(|v| v.0))),
            Some(DisplayOption(DEFAULT_WORKSPACE_CONFIG.layout_flip)),
            None,
        );
        let apply_window_based_offset = opt_helpers::toggle_with_disable_default(
            "Apply Window Based Work Area Offset",
            Some("Apply this monitor's window-based work area offset (default: true)"),
            ws_config
                .apply_window_based_work_area_offset
                .or(DEFAULT_WORKSPACE_CONFIG.apply_window_based_work_area_offset),
            DEFAULT_WORKSPACE_CONFIG.apply_window_based_work_area_offset,
            |v| Message::ConfigChange(ConfigChange::ApplyWindowBasedWorkAreaOffset(v)),
            None,
        );
        let container_padding = opt_helpers::number_with_disable_default_option(
            "Container Padding",
            Some("Container padding (default: global)"),
            ws_config.container_padding,
            DEFAULT_WORKSPACE_CONFIG.container_padding,
            |v| Message::ConfigChange(ConfigChange::ContainerPadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.container_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::ContainerPadding(v)),
            }),
        );
        let float_override = opt_helpers::toggle_with_disable_default(
            "Float Override",
            Some(
                "Enable or disable float override, which makes it so every new window opens in floating mode (default: global)",
            ),
            ws_config.float_override,
            DEFAULT_WORKSPACE_CONFIG.float_override,
            |v| Message::ConfigChange(ConfigChange::FloatOverride(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.float_override.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::FloatOverride(v)),
            }),
        );
        let floating_layer_behaviour = opt_helpers::choose_with_disable_default(
            "Floating Layer Behaviour",
            Some("Determine what happens to a new window when the Floating workspace layer is active (default: global)"),
            vec![
                t("Selected: 'Tile' -> Tile new windows opened when floating layer is active (unless they match a float rule)").into(),
                t("Selected: 'Float' -> Float new windows opened when floating layer is active.")
                    .into(),
            ],
            [
                FloatingLayerBehaviour::Tile,
                FloatingLayerBehaviour::Float,
            ],
            ws_config.floating_layer_behaviour,
            |v| Message::ConfigChange(ConfigChange::FloatingLayerBehaviour(v)),
            DEFAULT_WORKSPACE_CONFIG.floating_layer_behaviour,
            Some(opt_helpers::DisableArgs {
                disable: ws_config.floating_layer_behaviour.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::FloatingLayerBehaviour(v)),
            }),
        );
        let layout_rules = opt_helpers::expandable(
            "Layout Rules",
            Some(
                "Layout rules (default: None)\n\n\
                Define rules to automatically change the layout on a specified \
                workspace when a threshold of window containers is met.\n\n\
                However, if you add workspace layout rules, you will not be able \
                to manually change the layout of a workspace until all layout \
                rules for that workspace have been cleared.",
            ),
            || layout_rules_children(&ws_config.layout_rules, self),
            ws_config.layout_rules.is_some(),
            Message::ConfigChange(ConfigChange::LayoutRules(None)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.layout_rules.is_none(),
                label: Some("None"),
                on_toggle: |v| {
                    Message::ConfigChange(ConfigChange::LayoutRules((!v).then_some(HashMap::new())))
                },
            }),
        );
        let window_container_behaviour = opt_helpers::choose_with_disable_default(
            "Window Container Behaviour",
            Some("Determine what happens when a new window is opened (default: global)"),
            vec![
                t("Selected: 'Create' -> Create a new container for each new window").into(),
                t("Selected: 'Append' -> Append new windows to the focused window container")
                    .into(),
            ],
            [
                WindowContainerBehaviour::Create,
                WindowContainerBehaviour::Append,
            ],
            ws_config.window_container_behaviour,
            |v| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(v)),
            DEFAULT_WORKSPACE_CONFIG.window_container_behaviour,
            Some(opt_helpers::DisableArgs {
                disable: ws_config.window_container_behaviour.is_none(),
                label: Some("Global"),
                on_toggle: |v| {
                    Message::ToggleOverrideGlobal(OverrideConfig::WindowContainerBehaviour(v))
                },
            }),
        );
        let window_container_behaviour_rules = opt_helpers::expandable(
            "Window Container Behaviour Rules",
            Some(
                "Window Container Behaviour rules (default: None)\n\n\
                Define rules to automatically change the window container behaviour \
                on a specified workspace when a threshold of window containers is met.\n\n\
                However, if you add workspace window container behaviour rules, you \
                will not be able to manually change the layout of a workspace until \
                all behaviour rules for that workspace have been cleared.",
            ),
            || behaviour_rules_children(&ws_config.window_container_behaviour_rules, self),
            ws_config.window_container_behaviour_rules.is_some(),
            Message::ConfigChange(ConfigChange::BehaviourRules(None)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.window_container_behaviour_rules.is_none(),
                label: Some("None"),
                on_toggle: |v| {
                    Message::ConfigChange(ConfigChange::BehaviourRules(
                        (!v).then_some(HashMap::new()),
                    ))
                },
            }),
        );
        let workspace_padding = opt_helpers::number_with_disable_default_option(
            "Workspace Padding",
            Some("Workspace padding (default: global)"),
            ws_config.workspace_padding,
            DEFAULT_WORKSPACE_CONFIG.workspace_padding,
            |v| Message::ConfigChange(ConfigChange::WorkspacePadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.workspace_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::WorkspacePadding(v)),
            }),
        );
        let tile = opt_helpers::toggle_with_disable_default(
            "Tile Workspace",
            Some("Enable or disable tiling for the workspace (default: true)"),
            ws_config.tile,
            DEFAULT_WORKSPACE_CONFIG.tile,
            |v| Message::ConfigChange(ConfigChange::Tile(v)),
            None,
        );
        let work_area_offset = opt_helpers::expandable(
            "Work Area Offset",
            Some("Workspace-specific work area offset (default: global)"),
            || {
                [
                    opt_helpers::number(
                        "left",
                        None,
                        ws_config.work_area_offset.map_or(0, |r| r.left),
                        move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetLeft(value)),
                    ),
                    opt_helpers::number(
                        "top",
                        None,
                        ws_config.work_area_offset.map_or(0, |r| r.top),
                        move |value| Message::ConfigChange(ConfigChange::WorkAreaOffsetTop(value)),
                    ),
                    opt_helpers::number(
                        "bottom",
                        None,
                        ws_config.work_area_offset.map_or(0, |r| r.bottom),
                        move |value| {
                            Message::ConfigChange(ConfigChange::WorkAreaOffsetBottom(value))
                        },
                    ),
                    opt_helpers::number(
                        "right",
                        None,
                        ws_config.work_area_offset.map_or(0, |r| r.right),
                        move |value| {
                            Message::ConfigChange(ConfigChange::WorkAreaOffsetRight(value))
                        },
                    ),
                ]
            },
            ws_config.work_area_offset.is_some(),
            Message::ConfigChange(ConfigChange::WorkAreaOffset(None)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.work_area_offset.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::WorkAreaOffset(v)),
            }),
        );
        let wp = opt_helpers::opt_button_disable_default(
            "Wallpaper",
            Some(
                "Specify a wallpaper which will be set when switching to this workspace. (default: None)\n\n\
                It doesn't go back to the previous wallpaper when moving to another workspace. \
                If you don't have other workspaces with a wallpaper, then moving to this workspace will change the \
                wallpaper and it will stay as that wallpaper forever. In order to have your wallpaper change on \
                different workspaces you need to setup 'Wallpaper' on all of them.\n\n\
                Changing wallpapers is really slow on Windows unfortunately, so you won't have the smoothest \
                experience when using this setting.",
            ),
            Message::SetScreen(Screen::WorkspaceWallpaper),
            ws_config.wallpaper.is_some(),
            Some(Message::ConfigChange(ConfigChange::Wallpaper(None))),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.wallpaper.is_none(),
                label: Some("None"),
                on_toggle: |v| {
                    Message::ConfigChange(ConfigChange::Wallpaper(
                        (!v).then(|| wallpaper::DEFAULT_WALLPAPER.clone()),
                    ))
                },
            }),
        );
        let initial_workspace_rules_button = opt_helpers::opt_button(
            "Initial Workspace Rules",
            Some(
                "Initial workspace application rules. The matched windows only move to this worksapace once, \
                after that you can freely move them anywhere.",
            ),
            Message::SetScreen(Screen::InitialWorkspaceRules),
        );
        let workspace_rules_button = opt_helpers::opt_button(
            "Workspace Rules",
            Some(
                "Permanent workspace application rules. The matched windows will always move to this workspace.",
            ),
            Message::SetScreen(Screen::WorkspaceRules),
        );
        column![
            name,
            layout,
            grid_option_rows,
            scrolling_option_columns,
            scrolling_option_center,
            layout_flip,
            apply_window_based_offset,
            container_padding,
            float_override,
            floating_layer_behaviour,
            layout_rules,
            tile,
            window_container_behaviour,
            window_container_behaviour_rules,
            workspace_padding,
            work_area_offset,
            wp,
            initial_workspace_rules_button,
            workspace_rules_button,
        ]
        .spacing(10)
        .into()
    }

    pub fn subscription(&self) -> Subscription<(usize, Message)> {
        match self.screen {
            Screen::Workspace | Screen::WorkspaceWallpaper => Subscription::none(),
            Screen::WorkspaceRules | Screen::InitialWorkspaceRules => self
                .rule
                .subscription()
                .with(self.index)
                .map(|(w_idx, m)| (w_idx, Message::Rule(m))),
        }
    }
}

fn get_rules_from_config<'a>(
    config: &'a WorkspaceConfig,
    screen: &'a Screen,
) -> Option<&'a Vec<MatchingRule>> {
    match screen {
        Screen::WorkspaceRules => config.workspace_rules.as_ref(),
        Screen::InitialWorkspaceRules => config.initial_workspace_rules.as_ref(),
        _ => None,
    }
}

fn get_rules_from_config_mut<'a>(
    config: &'a mut WorkspaceConfig,
    screen: &'a Screen,
) -> &'a mut Option<Vec<MatchingRule>> {
    match screen {
        Screen::WorkspaceRules => &mut config.workspace_rules,
        Screen::InitialWorkspaceRules => &mut config.initial_workspace_rules,
        _ => panic!("wrong screen for rules!"),
    }
}

fn layout_rule<'a>(
    limit: usize,
    layout: Layout,
    limit_message: impl Fn(usize) -> Message + Copy + 'static,
    layout_message: impl Fn(Layout) -> Message + 'a,
    is_add: bool,
) -> Element<'a, Message> {
    let number = opt_helpers::number_simple(limit, limit_message).width(50);
    let choose = container(
        pick_list(
            &LAYOUT_OPTIONS_WITHOUT_NONE[..],
            Some(layout),
            layout_message,
        )
        .font(ICONS)
        .text_shaping(text::Shaping::Advanced),
    )
    .max_width(200)
    .width(Fill);
    let final_button = if is_add {
        button(icons::plus().style(|t| text::Style {
            color: t.palette().primary.into(),
        }))
        .on_press(Message::AddNewLayoutRule)
        .style(button::text)
    } else {
        button(icons::delete().style(|t| text::Style {
            color: t.palette().danger.into(),
        }))
        .on_press(Message::RemoveLayoutRule(limit))
        .style(button::text)
    };
    row![
        text("If windows open >="),
        number,
        text("change layout to "),
        choose,
        final_button,
    ]
    .spacing(5)
    .align_y(Center)
    .into()
}

fn layout_rules_children<'a>(
    layout_rules: &Option<HashMap<usize, DefaultLayout>>,
    workspace: &Workspace,
) -> Vec<Element<'a, Message>> {
    let mut children = Vec::new();
    let new_rule = opt_helpers::opt_box(
        column![
            text("Add New Rule:"),
            layout_rule(
                workspace.new_layout_rule_limit,
                workspace.new_layout_rule_layout,
                Message::ChangeNewLayoutRuleLimit,
                Message::ChangeNewLayoutRuleLayout,
                true,
            ),
        ]
        .spacing(10),
    );
    children.push(new_rule.into());
    let mut rules = layout_rules.as_ref().map_or(Vec::new(), |lr| {
        lr.iter()
            .collect::<BTreeMap<&usize, &DefaultLayout>>()
            .into_iter()
            .map(|(limit, layout)| {
                let limit = *limit;
                let layout = *layout;
                layout_rule(
                    limit,
                    layout.into(),
                    move |new_limit| {
                        Message::ConfigChange(ConfigChange::LayoutRuleLimit((limit, new_limit)))
                    },
                    move |new_layout| {
                        Message::ConfigChange(ConfigChange::LayoutRuleLayout((limit, new_layout)))
                    },
                    false,
                )
            })
            .collect()
    });
    if rules.is_empty() {
        rules.push(text("Rules:").into());
        // The 30.8 height came from trial and error to make it so the space is the
        // same as the one from one rule. Why isn't it 30, I don't know?! Any other
        // value other 30.8 would result in the UI adjusting when adding first rule.
        rules.push(space().height(30.8).into());
    } else {
        rules.insert(0, text("Rules:").into());
    }
    children.push(ruler::horizontal(2.0).into());
    children.extend(rules);
    children
}

fn behaviour_rule<'a>(
    limit: usize,
    behaviour: WindowContainerBehaviour,
    limit_message: impl Fn(usize) -> Message + Copy + 'static,
    behaviour_message: impl Fn(WindowContainerBehaviour) -> Message + 'a,
    is_add: bool,
) -> Element<'a, Message> {
    let number = opt_helpers::number_simple(limit, limit_message).width(50);
    let choose = container(
        pick_list(
            [
                WindowContainerBehaviour::Create,
                WindowContainerBehaviour::Append,
            ],
            Some(behaviour),
            behaviour_message,
        )
        .font(ICONS)
        .text_shaping(text::Shaping::Advanced),
    )
    .max_width(200)
    .width(Fill);
    let final_button = if is_add {
        button(icons::plus().style(|t| text::Style {
            color: t.palette().primary.into(),
        }))
        .on_press(Message::AddNewBehaviourRule)
        .style(button::text)
    } else {
        button(icons::delete().style(|t| text::Style {
            color: t.palette().danger.into(),
        }))
        .on_press(Message::RemoveBehaviourRule(limit))
        .style(button::text)
    };
    row![
        text("If windows open >="),
        number,
        text("change behaviour to "),
        choose,
        final_button,
    ]
    .spacing(5)
    .align_y(Center)
    .into()
}

fn behaviour_rules_children<'a>(
    behaviour_rules: &Option<HashMap<usize, WindowContainerBehaviour>>,
    workspace: &Workspace,
) -> Vec<Element<'a, Message>> {
    let mut children = Vec::new();
    let new_rule = opt_helpers::opt_box(
        column![
            text("Add New Rule:"),
            behaviour_rule(
                workspace.new_behaviour_rule_limit,
                workspace.new_behaviour_rule_behaviour,
                Message::ChangeNewBehaviourRuleLimit,
                Message::ChangeNewBehaviourRuleBehaviour,
                true,
            ),
        ]
        .spacing(10),
    );
    children.push(new_rule.into());
    let mut rules = behaviour_rules.as_ref().map_or(Vec::new(), |br| {
        br.iter()
            .collect::<BTreeMap<&usize, &WindowContainerBehaviour>>()
            .into_iter()
            .map(|(limit, behaviour)| {
                let limit = *limit;
                let behaviour = *behaviour;
                behaviour_rule(
                    limit,
                    behaviour,
                    move |new_limit| {
                        Message::ConfigChange(ConfigChange::BehaviourRuleLimit((limit, new_limit)))
                    },
                    move |new_behaviour| {
                        Message::ConfigChange(ConfigChange::BehaviourRuleBehaviour((
                            limit,
                            new_behaviour,
                        )))
                    },
                    false,
                )
            })
            .collect()
    });
    if rules.is_empty() {
        rules.push(text("Rules:").into());
        // The 30.8 height came from trial and error to make it so the space is the
        // same as the one from one rule. Why isn't it 30, I don't know?! Any other
        // value other 30.8 would result in the UI adjusting when adding first rule.
        rules.push(space().height(30.8).into());
    } else {
        rules.insert(0, text("Rules:").into());
    }
    children.push(ruler::horizontal(2.0).into());
    children.extend(rules);
    children
}

fn layout_options_descriptions<'a>() -> Vec<Element<'a, Message>> {
    vec![
        row![t("Selected: '[None] (Floating)' layout -> This workspace will behave as a floating workspace, like normal Windows does!")].spacing(5).into(),
        row![t("Selected: "), td(icons::bsp()), t("'BSP' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::vstack()), t("'Vertical Stack' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::rmvstack()), t("'Right Main Vertical Stack' layout")].spacing(5).into(),
        row![
            t("Selected: "),
            td(icons::uwvstack()),
            t("'Ultra Wide Vertical Stack' layout ->"),
            t("recommended if using an ultrawide monitor")
        ]
        .spacing(5)
        .wrap()
        .into(),
        row![t("Selected: "), td(icons::hstack()), t("'Horizontal Stack' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::rows()), t("'Rows' layout -> recommended if using a vertical monitor")].spacing(5).into(),
        row![t("Selected: "), td(icons::columns()), t("'Columns' layout")].spacing(5).into(),
        column![
            row![
                t("Selected: "),
                td(icons::grid()),
                t("'Grid' layout"),
            ].spacing(5),
            t("If you like the grid layout in LeftWM this is almost exactly the same!\n\n\
            The 'Grid' layout does not support resizing windows.")
        ]
        .spacing(5)
        .into(),
        column![
            row![
                t("Selected: "),
                td(icons::scrolling()),
                t("'Scrolling' layout")
            ]
            .spacing(5),
            t("The Scrolling layout is inspired by the Niri scrolling window manager, presenting a workspace \
            as an infinite scrollable horizontal strip with a viewport which includes the focused window + N \
            other windows in columns. There is no support for splitting columns into multiple rows.\n\n\
            This layout can currently only be applied to single-monitor setups as the scrolling would result \
            in layout calculations which push the windows in the columns moving out of the viewport onto \
            adjacent monitors.")
        ]
        .spacing(5)
        .into(),
    ]
}
