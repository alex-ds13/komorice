use crate::widget::{self, icons, opt_helpers};

use std::collections::HashMap;

use iced::{
    padding,
    widget::{button, column, container, pick_list, row, text, text_input, Space, Text},
    Center, Element, Fill, Right, Shrink, Task,
};
use komorebi::{
    config_generation::{IdWithIdentifier, MatchingRule},
    ApplicationIdentifier,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref APPLICATION_IDENTIFIER_OPTIONS: [ApplicationIdentifier; 4] = [
        ApplicationIdentifier::Exe,
        ApplicationIdentifier::Title,
        ApplicationIdentifier::Class,
        ApplicationIdentifier::Path,
    ];
    static ref MATCHING_STRATEGY_OPTIONS: [MatchingStrategy; 10] = [
        MatchingStrategy::Legacy,
        MatchingStrategy::Equals,
        MatchingStrategy::StartsWith,
        MatchingStrategy::EndsWith,
        MatchingStrategy::Contains,
        MatchingStrategy::Regex,
        MatchingStrategy::DoesNotEndWith,
        MatchingStrategy::DoesNotStartWith,
        MatchingStrategy::DoesNotEqual,
        MatchingStrategy::DoesNotContain,
    ];
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchingStrategy {
    Legacy,
    Equals,
    StartsWith,
    EndsWith,
    Contains,
    Regex,
    DoesNotEndWith,
    DoesNotStartWith,
    DoesNotEqual,
    DoesNotContain,
}

impl std::fmt::Display for MatchingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchingStrategy::Legacy => write!(f, "Legacy"),
            MatchingStrategy::Equals => write!(f, "Equals"),
            MatchingStrategy::StartsWith => write!(f, "StartsWith"),
            MatchingStrategy::EndsWith => write!(f, "EndsWith"),
            MatchingStrategy::Contains => write!(f, "Contains"),
            MatchingStrategy::Regex => write!(f, "Regex"),
            MatchingStrategy::DoesNotEndWith => write!(f, "DoesNotEndWith"),
            MatchingStrategy::DoesNotStartWith => write!(f, "DoesNotStartWith"),
            MatchingStrategy::DoesNotEqual => write!(f, "DoesNotEqual"),
            MatchingStrategy::DoesNotContain => write!(f, "DoesNotContain"),
        }
    }
}

impl From<MatchingStrategy> for komorebi::config_generation::MatchingStrategy {
    fn from(value: MatchingStrategy) -> Self {
        match value {
            MatchingStrategy::Legacy => komorebi::config_generation::MatchingStrategy::Legacy,
            MatchingStrategy::Equals => komorebi::config_generation::MatchingStrategy::Equals,
            MatchingStrategy::StartsWith => {
                komorebi::config_generation::MatchingStrategy::StartsWith
            }
            MatchingStrategy::EndsWith => komorebi::config_generation::MatchingStrategy::EndsWith,
            MatchingStrategy::Contains => komorebi::config_generation::MatchingStrategy::Contains,
            MatchingStrategy::Regex => komorebi::config_generation::MatchingStrategy::Regex,
            MatchingStrategy::DoesNotEndWith => {
                komorebi::config_generation::MatchingStrategy::DoesNotEndWith
            }
            MatchingStrategy::DoesNotStartWith => {
                komorebi::config_generation::MatchingStrategy::DoesNotStartWith
            }
            MatchingStrategy::DoesNotEqual => {
                komorebi::config_generation::MatchingStrategy::DoesNotEqual
            }
            MatchingStrategy::DoesNotContain => {
                komorebi::config_generation::MatchingStrategy::DoesNotContain
            }
        }
    }
}

impl From<&komorebi::config_generation::MatchingStrategy> for MatchingStrategy {
    fn from(value: &komorebi::config_generation::MatchingStrategy) -> Self {
        match value {
            komorebi::config_generation::MatchingStrategy::Legacy => MatchingStrategy::Legacy,
            komorebi::config_generation::MatchingStrategy::Equals => MatchingStrategy::Equals,
            komorebi::config_generation::MatchingStrategy::StartsWith => {
                MatchingStrategy::StartsWith
            }
            komorebi::config_generation::MatchingStrategy::EndsWith => MatchingStrategy::EndsWith,
            komorebi::config_generation::MatchingStrategy::Contains => MatchingStrategy::Contains,
            komorebi::config_generation::MatchingStrategy::Regex => MatchingStrategy::Regex,
            komorebi::config_generation::MatchingStrategy::DoesNotEndWith => {
                MatchingStrategy::DoesNotEndWith
            }
            komorebi::config_generation::MatchingStrategy::DoesNotStartWith => {
                MatchingStrategy::DoesNotStartWith
            }
            komorebi::config_generation::MatchingStrategy::DoesNotEqual => {
                MatchingStrategy::DoesNotEqual
            }
            komorebi::config_generation::MatchingStrategy::DoesNotContain => {
                MatchingStrategy::DoesNotContain
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    ToggleRulesExpand,
    RulesHover(bool),

    ChangeNewRuleKind(usize, ApplicationIdentifier),
    ChangeNewRuleId(usize, String),
    ChangeNewRuleMatchingStrategy(usize, Option<MatchingStrategy>),

    ToggleShowNewRule,

    AddNewRule,
    ComposingAddToNewRule,

    ToggleRuleHover(usize, bool),
    ToggleRuleEdit(usize, bool),
    SaveRuleEdit(usize),

    ChangeRuleKind(usize, usize, ApplicationIdentifier),
    ChangeRuleId(usize, usize, String),
    ChangeRuleMatchingStrategy(usize, usize, Option<MatchingStrategy>),
    ComposingAddToRule(usize),

    RemoveRule(usize),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct Rule {
    pub show_new_rule: bool,
    pub new_rule: Vec<IdWithIdentifier>,
    pub rules_settings: HashMap<usize, RuleSettings>,
}

#[derive(Debug, Default)]
pub struct RuleSettings {
    pub is_hovered: bool,
    pub edit: bool,
}

impl Rule {
    pub fn new(rules: &Option<Vec<MatchingRule>>) -> Self {
        Rule {
            show_new_rule: false,
            new_rule: Vec::new(),
            rules_settings: rules.as_ref().map_or(HashMap::new(), |rules| {
                rules
                    .iter()
                    .enumerate()
                    .map(|(idx, _rule)| (idx, RuleSettings::default()))
                    .collect()
            }),
        }
    }

    pub fn update(
        &mut self,
        rules: &mut Option<Vec<MatchingRule>>,
        message: Message,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ToggleRulesExpand => todo!(),
            Message::RulesHover(_) => todo!(),
            Message::ChangeNewRuleKind(idx, kind) => {
                if let Some(rule) = self.new_rule.get_mut(idx) {
                    rule.kind = kind;
                }
                //TODO: inform user if idx didn't exist?!
            }
            Message::ChangeNewRuleId(idx, id) => {
                if let Some(rule) = self.new_rule.get_mut(idx) {
                    rule.id = id;
                }
                //TODO: inform user if idx didn't exist?!
            }
            Message::ChangeNewRuleMatchingStrategy(idx, matching_strategy) => {
                if let Some(rule) = self.new_rule.get_mut(idx) {
                    rule.matching_strategy = matching_strategy.map(Into::into);
                }
                //TODO: inform user if idx didn't exist?!
            }
            Message::ToggleShowNewRule => {
                self.new_rule = vec![default_rule()];
                self.show_new_rule = !self.show_new_rule;
            }
            Message::AddNewRule => {
                if self.new_rule.len() == 1 {
                    if let Some(rules) = rules {
                        let rule = MatchingRule::Simple(self.new_rule.remove(0));
                        rules.push(rule);
                        let idx = rules.len() - 1;
                        self.rules_settings.entry(idx).or_default();
                    } else {
                        let rule = MatchingRule::Simple(self.new_rule.remove(0));
                        *rules = Some(vec![rule]);
                        self.rules_settings.entry(0).or_default();
                    }
                } else if let Some(rules) = rules {
                    let rule = MatchingRule::Composite(self.new_rule.drain(..).collect());
                    rules.push(rule);
                    let idx = rules.len() - 1;
                    self.rules_settings.entry(idx).or_default();
                } else {
                    let rule = MatchingRule::Composite(self.new_rule.drain(..).collect());
                    *rules = Some(vec![rule]);
                    self.rules_settings.entry(0).or_default();
                }
                self.new_rule = vec![default_rule()];
            }
            Message::ComposingAddToNewRule => {
                self.new_rule.push(default_rule());
            }
            Message::ToggleRuleHover(_, _) => todo!(),
            Message::ToggleRuleEdit(idx, edit) => {
                if let (Some(_rule), Some(rule_settings)) = (
                    rules.as_mut().and_then(|rls| rls.get_mut(idx)),
                    self.rules_settings.get_mut(&idx),
                ) {
                    rule_settings.edit = edit;
                }
            }
            Message::SaveRuleEdit(idx) => {
                todo!()
            }
            Message::ChangeRuleKind(idx, sub_idx, kind) => {
                if let Some(rule) = rules.as_mut().and_then(|rls| rls.get_mut(idx)) {
                    match rule {
                        MatchingRule::Simple(rule) => {
                            rule.kind = kind;
                        }
                        MatchingRule::Composite(rules) => {
                            if let Some(rule) = rules.get_mut(sub_idx) {
                                rule.kind = kind;
                            }
                        }
                    }
                }
            }
            Message::ChangeRuleId(idx, sub_idx, id) => {
                if let Some(rule) = rules.as_mut().and_then(|rls| rls.get_mut(idx)) {
                    match rule {
                        MatchingRule::Simple(rule) => {
                            rule.id = id;
                        }
                        MatchingRule::Composite(rules) => {
                            if let Some(rule) = rules.get_mut(sub_idx) {
                                rule.id = id;
                            }
                        }
                    }
                }
            }
            Message::ChangeRuleMatchingStrategy(idx, sub_idx, matching_strategy) => {
                if let Some(rule) = rules.as_mut().and_then(|rls| rls.get_mut(idx)) {
                    match rule {
                        MatchingRule::Simple(rule) => {
                            rule.matching_strategy = matching_strategy.map(Into::into);
                        }
                        MatchingRule::Composite(rules) => {
                            if let Some(rule) = rules.get_mut(sub_idx) {
                                rule.matching_strategy = matching_strategy.map(Into::into);
                            }
                        }
                    }
                }
            }
            Message::ComposingAddToRule(_) => todo!(),
            Message::RemoveRule(idx) => {
                if let Some(rules) = rules {
                    if rules.get(idx).is_some() {
                        rules.remove(idx);
                        self.rules_settings.remove(&idx);
                    }
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        title: impl Into<Text<'a>>,
        rules: Option<&'a Vec<MatchingRule>>,
    ) -> Element<'a, Message> {
        if let Some(rules) = rules {
            let add_new_rule_button =
                widget::button_with_icon(icons::plus_icon(), text("Add New Rule"))
                    .on_press(Message::ToggleShowNewRule)
                    .style(button::secondary);

            let new_rule: Element<_> = if self.show_new_rule {
                let rls = self.new_rule.iter().enumerate().fold(
                    column![].spacing(10),
                    |col, (idx, rule)| {
                        col.push(rule_view(idx, rule, idx == self.new_rule.len() - 1, true))
                    },
                );
                // let add_rule_button = button(icons::plus_icon().style(|t| text::Style {
                //     color: t.palette().primary.into(),
                // }))
                // .style(button::text)
                // .on_press(Message::AddNewRule);
                let add_rule_button = button(icons::plus_icon()).on_press(Message::AddNewRule);
                opt_helpers::opt_box(
                    row![
                        column!["Match any window where:", rls].spacing(10),
                        add_rule_button
                    ]
                    .spacing(10)
                    .align_y(Center),
                )
                .into()
            } else {
                Space::new(Shrink, Shrink).into()
            };

            let rls = rules
                .iter()
                .enumerate()
                .fold(column![].spacing(10), |col, (idx, rule)| match rule {
                    MatchingRule::Simple(rule) => col.push(iced::widget::hover(
                        iced::widget::stack([
                            container(
                                opt_helpers::opt_box(
                                    column![
                                        "Match any window where:",
                                        rule_view(
                                            idx,
                                            rule,
                                            false,
                                            self.rules_settings
                                                .get(&idx)
                                                .map(|rs| rs.edit)
                                                .unwrap_or_default()
                                        )
                                    ]
                                    .spacing(10),
                                )
                                .style(opt_helpers::opt_box_style_bottom),
                            )
                            .padding(padding::right(170))
                            .into(),
                            column![row![]
                                .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                    rs.edit.then_some(
                                        button(icons::cross_icon())
                                            .on_press(Message::ToggleRuleEdit(idx, false))
                                            .style(button::secondary),
                                    )
                                }))
                                .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                    rs.edit.then_some(
                                        button(icons::check_icon())
                                            .on_press(Message::SaveRuleEdit(idx))
                                            .style(button::primary),
                                    )
                                }))
                                .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                    rs.edit.then_some(
                                        button(icons::delete_icon())
                                            .on_press(Message::RemoveRule(idx))
                                            .style(button::danger),
                                    )
                                }))
                                .spacing(10)
                                .align_y(Center)
                                .width(160)
                                .height(Fill)]
                            .width(Fill)
                            .align_x(Right)
                            .into(),
                        ]),
                        column![row![]
                            .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                (!rs.edit).then_some(
                                    button(icons::edit_icon())
                                        .on_press(Message::ToggleRuleEdit(idx, true))
                                        .style(button::secondary),
                                )
                            }))
                            .spacing(10)
                            .align_y(Center)
                            .width(160)
                            .height(Fill)]
                        .width(Fill)
                        .align_x(Right),
                    )),
                    MatchingRule::Composite(rules) => col.push(iced::widget::hover(
                        iced::widget::stack([
                            container(
                                opt_helpers::opt_box(rules.iter().enumerate().fold(
                                    column!["Match any window where:"].spacing(10),
                                    |col, (i, r)| {
                                        col.push(rule_view(
                                            idx,
                                            r,
                                            i != rules.len() - 1,
                                            self.rules_settings
                                                .get(&idx)
                                                .map(|rs| rs.edit)
                                                .unwrap_or_default(),
                                        ))
                                    },
                                ))
                                .style(opt_helpers::opt_box_style_bottom),
                            )
                            .padding(padding::right(170))
                            .into(),
                            column![row![]
                                .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                    rs.edit.then_some(
                                        button(icons::cross_icon())
                                            .on_press(Message::ToggleRuleEdit(idx, false))
                                            .style(button::secondary),
                                    )
                                }))
                                .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                    rs.edit.then_some(
                                        button(icons::check_icon())
                                            .on_press(Message::SaveRuleEdit(idx))
                                            .style(button::primary),
                                    )
                                }))
                                .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                    rs.edit.then_some(
                                        button(icons::delete_icon())
                                            .on_press(Message::RemoveRule(idx))
                                            .style(button::danger),
                                    )
                                }))
                                .spacing(10)
                                .align_y(Center)
                                .width(160)
                                .height(Fill)]
                            .width(Fill)
                            .align_x(Right)
                            .into(),
                        ]),
                        column![row![]
                            .push_maybe(self.rules_settings.get(&idx).and_then(|rs| {
                                (!rs.edit).then_some(
                                    button(icons::edit_icon())
                                        .on_press(Message::ToggleRuleEdit(idx, true))
                                        .style(button::secondary),
                                )
                            }))
                            .spacing(10)
                            .align_y(Center)
                            .width(160)
                            .height(Fill)]
                        .width(Fill)
                        .align_x(Right),
                    )),
                });

            column![add_new_rule_button, new_rule, rls]
                .spacing(10)
                .into()
        } else {
            let add_new_rule_button =
                widget::button_with_icon(icons::plus_icon(), text("Add New Rule"))
                    .on_press(Message::ToggleShowNewRule)
                    .style(button::secondary);

            let new_rule: Element<_> = if self.show_new_rule {
                let rls = self.new_rule.iter().enumerate().fold(
                    column![].spacing(10),
                    |col, (idx, rule)| {
                        col.push(rule_view(idx, rule, idx == self.new_rule.len() - 1, true))
                    },
                );
                // let add_rule_button = button(icons::plus_icon().style(|t| text::Style {
                //     color: t.palette().primary.into(),
                // }))
                // .style(button::text)
                // .on_press(Message::AddNewRule);
                let add_rule_button = button(icons::plus_icon()).on_press(Message::AddNewRule);
                opt_helpers::opt_box(
                    row![
                        column!["Match any window where:", rls].spacing(10),
                        add_rule_button
                    ]
                    .spacing(10)
                    .align_y(Center),
                )
                .into()
            } else {
                Space::new(Shrink, Shrink).into()
            };

            column![add_new_rule_button, new_rule].spacing(10).into()
        }
    }
}

fn rule_view(idx: usize, rule: &IdWithIdentifier, show_and: bool, edit: bool) -> Element<Message> {
    let kind: Element<_> = if edit {
        pick_list(
            &APPLICATION_IDENTIFIER_OPTIONS[..],
            Some(rule.kind),
            move |v| Message::ChangeNewRuleKind(idx, v),
        )
        .into()
    } else {
        row![
            container(iced::widget::value(rule.kind))
                .padding(5)
                .style(container::dark),
            "→"
        ]
        .spacing(10)
        .align_y(Center)
        .into()
    };
    let matching_strategy: Element<_> = if edit {
        pick_list(
            &MATCHING_STRATEGY_OPTIONS[..],
            rule.matching_strategy
                .as_ref()
                .map(Into::<MatchingStrategy>::into),
            move |v| Message::ChangeNewRuleMatchingStrategy(idx, Some(v)),
        )
        .into()
    } else {
        row![
            container(iced::widget::value(
                rule.matching_strategy
                    .as_ref()
                    .map_or(MatchingStrategy::Legacy, Into::into),
            ))
            .padding(5)
            .style(container::dark),
            "→"
        ]
        .spacing(10)
        .align_y(Center)
        .into()
    };
    let id: Element<_> = if edit {
        container(
            text_input("", &rule.id)
                .on_input(move |v| Message::ChangeNewRuleId(idx, v))
                .width(150),
        )
        .max_width(150)
        .into()
    } else {
        container(text(&rule.id))
            .padding(5)
            .style(container::dark)
            .into()
    };
    let composing_add_button: Option<Element<_>> = show_and.then(|| {
        if edit {
            button(
                row![icons::level_down_icon(), "And"]
                    .spacing(5)
                    .align_y(Center),
            )
            .style(button::secondary)
            .on_press(Message::ComposingAddToNewRule)
            .into()
        } else {
            container(
                row![icons::level_down_icon(), "And"]
                    .spacing(5)
                    .align_y(Center),
            )
            .padding(5)
            .into()
        }
    });
    row![kind, matching_strategy, id]
        .push_maybe(composing_add_button)
        .spacing(10)
        .align_y(Center)
        .into()
}

fn default_rule() -> IdWithIdentifier {
    IdWithIdentifier {
        kind: ApplicationIdentifier::Exe,
        id: "".into(),
        matching_strategy: Some(MatchingStrategy::Equals.into()),
    }
}
