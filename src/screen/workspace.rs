use super::rule;

use crate::komo_interop::layout::{Layout, LAYOUT_OPTIONS, LAYOUT_OPTIONS_WITHOUT_NONE};
use crate::utils::DisplayOptionCustom as DisplayOption;
use crate::widget::icons::ICONS;
use crate::widget::opt_helpers::description_text as t;
use crate::widget::opt_helpers::to_description_text as td;
use crate::widget::{icons, opt_helpers};

use std::collections::{BTreeMap, HashMap};

use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, Space};
use iced::{Center, Element, Fill, Shrink, Subscription, Task};
use komorebi::config_generation::MatchingRule;
use komorebi::{WindowContainerBehaviour, WorkspaceConfig};
use komorebi_client::DefaultLayout;

#[derive(Clone, Debug)]
pub enum Message {
    SetScreen(Screen),
    ConfigChange(ConfigChange),
    ToggleOverrideGlobal(OverrideConfig),
    ToggleLayoutRulesExpand,
    LayoutRulesHover(bool),
    ChangeNewLayoutRuleLimit(i32),
    ChangeNewLayoutRuleLayout(Layout),
    AddNewLayoutRule,
    RemoveLayoutRule(usize),
    WorkspaceRulesHover(bool),
    InitialWorkspaceRulesHover(bool),
    Rule(rule::Message),
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
    LayoutRules(Option<HashMap<usize, DefaultLayout>>),
    LayoutRuleLimit((usize, i32)),
    LayoutRuleLayout((usize, Layout)),
    Name(String),
    WindowContainerBehaviour(Option<komorebi::WindowContainerBehaviour>),
    WorkspacePadding(Option<i32>),
}

#[derive(Clone, Debug)]
pub enum OverrideConfig {
    ContainerPadding(bool),
    FloatOverride(bool),
    WindowContainerBehaviour(bool),
    WorkspacePadding(bool),
}

#[derive(Clone, Debug, Default)]
pub enum Screen {
    #[default]
    Workspace,
    WorkspaceRules,
    InitialWorkspaceRules,
}

#[derive(Clone, Debug, Default)]
pub struct Workspace {
    pub index: usize,
    pub screen: Screen,
    pub rule: rule::Rule,
    pub is_hovered: bool,
    pub layout_rules_expanded: bool,
    pub layout_rules_hovered: bool,
    pub new_layout_rule_limit: usize,
    pub new_layout_rule_layout: Layout,
    pub workspace_rules_hovered: bool,
    pub initial_workspace_rules_hovered: bool,
}

pub trait WorkspaceScreen {
    fn update(&mut self, workspace: &mut Workspace, message: Message) -> (Action, Task<Message>);

    fn view<'a>(&'a self, workspace: &'a Workspace) -> Element<'a, Message>;
}

impl WorkspaceScreen for WorkspaceConfig {
    fn update(&mut self, workspace: &mut Workspace, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::SetScreen(screen) => {
                if matches!(
                    screen,
                    Screen::WorkspaceRules | Screen::InitialWorkspaceRules
                ) {
                    let rules = get_rules_from_config_mut(self, &screen);
                    workspace.rule = rule::Rule::new(rules);
                    workspace.screen = screen.clone();
                    let task = iced::widget::scrollable::scroll_to(
                        iced::widget::scrollable::Id::new("monitors_scrollable"),
                        iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                    );
                    return (Action::ScreenChange(screen), task);
                }
            }
            Message::ConfigChange(change) => match change {
                ConfigChange::ApplyWindowBasedWorkAreaOffset(value) => {
                    self.apply_window_based_work_area_offset = value
                }
                ConfigChange::ContainerPadding(value) => self.container_padding = value,
                ConfigChange::FloatOverride(value) => self.float_override = value,
                ConfigChange::Layout(value) => self.layout = value.map(Into::into),
                ConfigChange::LayoutRules(value) => {
                    self.layout_rules = value.map(Into::into);
                }
                ConfigChange::LayoutRuleLimit((previous_limit, new_limit)) => {
                    if let Ok(new_limit) = new_limit.try_into() {
                        if let Some(layout_rules) = &mut self.layout_rules {
                            if !layout_rules.contains_key(&new_limit) {
                                if let Some(layout) = layout_rules.remove(&previous_limit) {
                                    layout_rules.insert(new_limit, layout);
                                }
                            }
                        }
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
                ConfigChange::WorkspacePadding(value) => self.workspace_padding = value,
            },
            Message::ToggleOverrideGlobal(to_override) => match to_override {
                OverrideConfig::ContainerPadding(disable) => {
                    if disable {
                        self.container_padding = None;
                    } else {
                        self.container_padding = Some(10);
                    }
                }
                OverrideConfig::FloatOverride(disable) => {
                    if disable {
                        self.float_override = None;
                    } else {
                        self.float_override = Some(false);
                    }
                }
                OverrideConfig::WindowContainerBehaviour(disable) => {
                    if disable {
                        self.window_container_behaviour = None;
                    } else {
                        self.window_container_behaviour = Some(WindowContainerBehaviour::Create);
                    }
                }
                OverrideConfig::WorkspacePadding(disable) => {
                    if disable {
                        self.workspace_padding = None;
                    } else {
                        self.workspace_padding = Some(10);
                    }
                }
            },
            Message::ToggleLayoutRulesExpand => {
                workspace.layout_rules_expanded = !workspace.layout_rules_expanded;
            }
            Message::LayoutRulesHover(hover) => {
                workspace.layout_rules_hovered = hover;
            }
            Message::ChangeNewLayoutRuleLimit(limit) => {
                if let Ok(limit) = limit.try_into() {
                    workspace.new_layout_rule_limit = limit;
                }
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
            Message::WorkspaceRulesHover(hover) => {
                workspace.workspace_rules_hovered = hover;
            }
            Message::InitialWorkspaceRulesHover(hover) => {
                workspace.initial_workspace_rules_hovered = hover;
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
        }
        (Action::None, Task::none())
    }

    fn view<'a>(&'a self, workspace: &'a Workspace) -> Element<'a, Message> {
        match workspace.screen {
            Screen::Workspace => workspace.workspace_view(self),
            Screen::WorkspaceRules | Screen::InitialWorkspaceRules => workspace
                .rule
                .view(get_rules_from_config(self, &workspace.screen))
                .map(Message::Rule),
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
            Some(DisplayOption(
                ws_config.layout.map(Into::into),
                "[None] (Floating)",
            )),
            |s| Message::ConfigChange(ConfigChange::Layout(s.and_then(|s| s.0))),
            Some(DisplayOption(Some(Layout::BSP), "[None] (Floating)")),
            None,
        );
        let apply_window_based_offset = opt_helpers::toggle_with_disable_default(
            "Apply Window Based Work Area Offset",
            Some("Apply this monitor's window-based work area offset (default: true)"),
            ws_config.apply_window_based_work_area_offset.or(Some(true)),
            Some(true),
            |v| Message::ConfigChange(ConfigChange::ApplyWindowBasedWorkAreaOffset(v)),
            None,
        );
        let container_padding = opt_helpers::number_with_disable_default_option(
            "Container Padding",
            Some("Container padding (default: global)"),
            ws_config.container_padding,
            None,
            |v| Message::ConfigChange(ConfigChange::ContainerPadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.container_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::ContainerPadding(v)),
            }),
        );
        let float_override = opt_helpers::toggle_with_disable_default(
            "Float Override",
            Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: global)"),
            ws_config.float_override,
            None,
            |v| Message::ConfigChange(ConfigChange::FloatOverride(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.float_override.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::FloatOverride(v)),
            })
        );
        let layout_rules = opt_helpers::expandable_with_disable_default(
            "Layout Rules",
            Some(
                "Layout rules (default: None)\n\n\
                Define rules to automatically change the layout on a specified \
                workspace when a threshold of window containers is met.\n\n\
                However, if you add workspace layout rules, you will not be able \
                to manually change the layout of a workspace until all layout \
                rules for that workspace have been cleared.",
            ),
            layout_rules_children(&ws_config.layout_rules, self),
            self.layout_rules_expanded,
            self.layout_rules_hovered,
            Message::ToggleLayoutRulesExpand,
            Message::LayoutRulesHover,
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
            None,
            Some(opt_helpers::DisableArgs {
                disable: ws_config.window_container_behaviour.is_none(),
                label: Some("Global"),
                on_toggle: |v| {
                    Message::ToggleOverrideGlobal(OverrideConfig::WindowContainerBehaviour(v))
                },
            }),
        );
        let workspace_padding = opt_helpers::number_with_disable_default_option(
            "Workspace Padding",
            Some("Workspace padding (default: global)"),
            ws_config.workspace_padding,
            None,
            |v| Message::ConfigChange(ConfigChange::WorkspacePadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: ws_config.workspace_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::WorkspacePadding(v)),
            }),
        );
        let initial_workspace_rules_button = opt_helpers::opt_button(
            "Initial Workspace Rules",
            Some(
                "Initial workspace application rules. The matched windows only move to this worksapace once, \
                after that you can freely move them anywhere.",
            ),
            self.initial_workspace_rules_hovered,
            Message::SetScreen(Screen::InitialWorkspaceRules),
            Message::InitialWorkspaceRulesHover,
        );
        let workspace_rules_button = opt_helpers::opt_button(
            "Workspace Rules",
            Some(
                "Permanent workspace application rules. The matched windows will always move to this workspace.",
            ),
            self.workspace_rules_hovered,
            Message::SetScreen(Screen::WorkspaceRules),
            Message::WorkspaceRulesHover,
        );
        column![
            name,
            layout,
            apply_window_based_offset,
            container_padding,
            float_override,
            layout_rules,
            window_container_behaviour,
            workspace_padding,
            initial_workspace_rules_button,
            workspace_rules_button,
        ]
        .spacing(10)
        .into()
    }

    pub fn subscription(&self) -> Subscription<(usize, Message)> {
        match self.screen {
            Screen::Workspace => Subscription::none(),
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
    limit_message: impl Fn(i32) -> Message + Copy + 'static,
    layout_message: impl Fn(Layout) -> Message + 'a,
    is_add: bool,
) -> Element<'a, Message> {
    let number = opt_helpers::number_simple(limit as i32, limit_message).content_width(50);
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
        let add_button = button(icons::plus_icon().style(|t| text::Style {
            color: t.palette().primary.into(),
        }))
        .on_press(Message::AddNewLayoutRule)
        .style(button::text);
        add_button
    } else {
        let remove_button = button(icons::delete_icon().style(|t| text::Style {
            color: t.palette().danger.into(),
        }))
        .on_press(Message::RemoveLayoutRule(limit))
        .style(button::text);
        remove_button
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
        rules.push(Space::new(Shrink, 30.8).into());
    } else {
        rules.insert(0, text("Rules:").into());
    }
    children.push(horizontal_rule(2.0).into());
    children.extend(rules);
    children
}

fn layout_options_descriptions<'a>() -> Vec<Element<'a, Message>> {
    vec![
        row![t("Selected: '[None] (Floating)' layout -> This workspace will behave as a floating workspace, like normal Windows does!")].spacing(5).into(),
        row![t("Selected: "), td(icons::bsp_icon()), t("'BSP' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::vstack_icon()), t("Vertical Stack' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::rmvstack_icon()), t("Right Main Vertical Stack' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::uwvstack_icon()), t("Ultra Wide Vertical Stack' layout -> recommended if using and ultrawide monitor")].spacing(5).into(),
        row![t("Selected: "), td(icons::hstack()), t("Horizontal Stack' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::rows_icon()), t("Rows' layout -> recommended if using a vertical monitor")].spacing(5).into(),
        row![t("Selected: "), td(icons::columns_icon()), t("Columns' layout")].spacing(5).into(),
        row![t("Selected: "), td(icons::grid_icon()), t("Grid' layout -> If you like the grid layout in LeftWM this is almost exactly the same!\n\nThe 'Grid' layout does not suppot resizing windows.")].spacing(5).into(),
    ]
}
