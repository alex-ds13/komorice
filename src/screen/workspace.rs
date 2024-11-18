use crate::utils::DisplayOptionCustom as DisplayOption;
use crate::widget::{icons, opt_helpers};

use std::collections::{BTreeMap, HashMap};

use iced::widget::{button, column, container, horizontal_rule, pick_list, row, text, Space};
use iced::{Center, Element, Fill, Shrink};
use komorebi::{WindowContainerBehaviour, WorkspaceConfig};
use komorebi_client::DefaultLayout;
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_LAYOUT_OPTIONS: [DisplayOption<DefaultLayout>; 9] = [
        DisplayOption(None, "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::BSP), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::VerticalStack), "[None] (Floating)"),
        DisplayOption(
            Some(DefaultLayout::RightMainVerticalStack),
            "[None] (Floating)"
        ),
        DisplayOption(
            Some(DefaultLayout::UltrawideVerticalStack),
            "[None] (Floating)"
        ),
        DisplayOption(Some(DefaultLayout::HorizontalStack), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Rows), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Columns), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Grid), "[None] (Floating)"),
    ];
    static ref DEFAULT_LAYOUT_OPTIONS_WITHOUT_NONE: [DefaultLayout; 8] = [
        DefaultLayout::BSP,
        DefaultLayout::VerticalStack,
        DefaultLayout::RightMainVerticalStack,
        DefaultLayout::UltrawideVerticalStack,
        DefaultLayout::HorizontalStack,
        DefaultLayout::Rows,
        DefaultLayout::Columns,
        DefaultLayout::Grid,
    ];
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleOverrideGlobal(OverrideConfig),
    ToggleLayoutRulesExpand,
    LayoutRulesHover(bool),
    ChangeNewLayoutRuleLimit(i32),
    ChangeNewLayoutRuleLayout(DefaultLayout),
    AddNewLayoutRule,
    RemoveLayoutRule(usize),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    ApplyWindowBasedWorkAreaOffset(Option<bool>),
    ContainerPadding(Option<i32>),
    FloatOverride(Option<bool>),
    Layout(Option<DefaultLayout>),
    LayoutRules(Option<HashMap<usize, DefaultLayout>>),
    LayoutRuleLimit((usize, i32)),
    LayoutRuleLayout((usize, DefaultLayout)),
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

pub struct Workspace {
    pub is_hovered: bool,
    pub layout_rules_expanded: bool,
    pub layout_rules_hovered: bool,
    pub new_layout_rule_limit: usize,
    pub new_layout_rule_layout: DefaultLayout,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            is_hovered: Default::default(),
            layout_rules_expanded: Default::default(),
            layout_rules_hovered: Default::default(),
            new_layout_rule_limit: Default::default(),
            new_layout_rule_layout: DefaultLayout::BSP,
        }
    }
}

pub trait WorkspaceScreen {
    fn update(&mut self, workspace: &mut Workspace, message: Message) -> Action;

    fn view(&self, workspace: &Workspace) -> Element<Message>;
}

impl WorkspaceScreen for WorkspaceConfig {
    fn update(&mut self, workspace: &mut Workspace, message: Message) -> Action {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::ApplyWindowBasedWorkAreaOffset(value) => {
                    self.apply_window_based_work_area_offset = value
                }
                ConfigChange::ContainerPadding(value) => self.container_padding = value,
                ConfigChange::FloatOverride(value) => self.float_override = value,
                ConfigChange::Layout(value) => self.layout = value,
                ConfigChange::LayoutRules(value) => {
                    self.layout_rules = value;
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
                        *rule_layout = new_layout;
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
                        e.insert(workspace.new_layout_rule_layout);
                        workspace.new_layout_rule_limit = 0;
                        workspace.new_layout_rule_layout = DefaultLayout::BSP;
                    }
                } else {
                    let rules = HashMap::from([(
                        workspace.new_layout_rule_limit,
                        workspace.new_layout_rule_layout,
                    )]);
                    self.layout_rules = Some(rules);
                    workspace.new_layout_rule_limit = 0;
                    workspace.new_layout_rule_layout = DefaultLayout::BSP;
                }
            }
            Message::RemoveLayoutRule(limit) => {
                if let Some(layout_rules) = &mut self.layout_rules {
                    layout_rules.remove(&limit);
                }
            }
        }
        Action::None
    }

    fn view(&self, workspace: &Workspace) -> Element<Message> {
        let name = opt_helpers::input(
            "Name",
            Some("Name of the workspace. Should be unique."),
            "",
            &self.name,
            |v| Message::ConfigChange(ConfigChange::Name(v)),
            None,
        );
        let layout = opt_helpers::choose_with_disable_default(
            "Layout",
            Some("Layout (default: BSP)"),
            &DEFAULT_LAYOUT_OPTIONS[..],
            Some(DisplayOption(self.layout, "[None] (Floating)")),
            |s| Message::ConfigChange(ConfigChange::Layout(s.and_then(|s| s.0))),
            Some(DisplayOption(Some(DefaultLayout::BSP), "[None] (Floating)")),
            None,
        );
        let apply_window_based_offset = opt_helpers::toggle_with_disable_default(
            "Apply Window Based Work Area Offset",
            Some("Apply this monitor's window-based work area offset (default: true)"),
            self.apply_window_based_work_area_offset.or(Some(true)),
            Some(true),
            |v| Message::ConfigChange(ConfigChange::ApplyWindowBasedWorkAreaOffset(v)),
            None,
        );
        let container_padding = opt_helpers::number_with_disable_default_option(
            "Container Padding",
            Some("Container padding (default: global)"),
            self.container_padding,
            None,
            |v| Message::ConfigChange(ConfigChange::ContainerPadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.container_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::ContainerPadding(v)),
            }),
        );
        let float_override = opt_helpers::toggle_with_disable_default(
            "Float Override",
            Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: global)"),
            self.float_override,
            None,
            |v| Message::ConfigChange(ConfigChange::FloatOverride(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.float_override.is_none(),
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
            layout_rules_children(&self.layout_rules, workspace),
            workspace.layout_rules_expanded,
            workspace.layout_rules_hovered,
            Message::ToggleLayoutRulesExpand,
            Message::LayoutRulesHover,
            self.layout_rules.is_some(),
            Message::ConfigChange(ConfigChange::LayoutRules(None)),
            Some(opt_helpers::DisableArgs {
                disable: self.layout_rules.is_none(),
                label: Some("None"),
                on_toggle: |v| {
                    Message::ConfigChange(ConfigChange::LayoutRules((!v).then_some(HashMap::new())))
                },
            }),
        );
        let window_container_behaviour = opt_helpers::choose_with_disable_default(
            "Window Container Behaviour",
            Some("Determine what happens when a new window is opened (default: global)"),
            [
                WindowContainerBehaviour::Create,
                WindowContainerBehaviour::Append,
            ],
            self.window_container_behaviour,
            |v| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(v)),
            None,
            Some(opt_helpers::DisableArgs {
                disable: self.window_container_behaviour.is_none(),
                label: Some("Global"),
                on_toggle: |v| {
                    Message::ToggleOverrideGlobal(OverrideConfig::WindowContainerBehaviour(v))
                },
            }),
        );
        let workspace_padding = opt_helpers::number_with_disable_default_option(
            "Workspace Padding",
            Some("Workspace padding (default: global)"),
            self.workspace_padding,
            None,
            |v| Message::ConfigChange(ConfigChange::WorkspacePadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.workspace_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::WorkspacePadding(v)),
            }),
        );
        column![
            name,
            layout,
            apply_window_based_offset,
            container_padding,
            float_override,
            layout_rules,
            window_container_behaviour,
            workspace_padding
        ]
        .spacing(10)
        .into()
    }
}

fn layout_rule<'a>(
    limit: usize,
    layout: DefaultLayout,
    limit_message: impl Fn(i32) -> Message + Copy + 'static,
    layout_message: impl Fn(DefaultLayout) -> Message + 'a,
    is_add: bool,
) -> Element<'a, Message> {
    let number = opt_helpers::number_simple(limit as i32, limit_message).content_width(50);
    let choose = container(pick_list(
        &DEFAULT_LAYOUT_OPTIONS_WITHOUT_NONE[..],
        Some(layout),
        layout_message,
    ))
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
                    layout,
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
