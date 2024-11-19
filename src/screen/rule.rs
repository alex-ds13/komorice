use crate::widget::{self, icons, opt_helpers};

use std::collections::HashMap;

use iced::{
    padding,
    widget::{
        button, column, horizontal_space, pick_list, row, text, text_input, Column, Space, Text,
    },
    Center, Element, Shrink, Task,
};
use komorebi::{
    config_generation::{IdWithIdentifier, IdWithIdentifierAndComment, MatchingRule},
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
    pub fn update(
        &mut self,
        rules: &mut Option<Vec<MatchingRule>>,
        message: Message,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ToggleRulesExpand => todo!(),
            Message::RulesHover(_) => todo!(),
            Message::ChangeNewRuleKind(_, _) => todo!(),
            Message::ChangeNewRuleId(_, _) => todo!(),
            Message::ChangeNewRuleMatchingStrategy(_, _) => todo!(),
            Message::ToggleShowNewRule => {
                self.new_rule = vec![IdWithIdentifier {
                    kind: ApplicationIdentifier::Exe,
                    id: "".into(),
                    matching_strategy: Some(MatchingStrategy::Equals.into()),
                }];
                self.show_new_rule = !self.show_new_rule;
            }
            Message::AddNewRule => {
                if self.new_rule.len() == 1 {
                    if let Some(rules) = rules {
                        let rule = MatchingRule::Simple(self.new_rule.remove(0));
                        rules.push(rule);
                    } else {
                        let rule = MatchingRule::Simple(self.new_rule.remove(0));
                        *rules = Some(vec![rule]);
                    }
                } else if let Some(rules) = rules {
                    let rule = MatchingRule::Composite(self.new_rule.drain(..).collect());
                    rules.push(rule);
                } else {
                    let rule = MatchingRule::Composite(self.new_rule.drain(..).collect());
                    *rules = Some(vec![rule]);
                }
                self.new_rule = vec![IdWithIdentifier {
                    kind: ApplicationIdentifier::Exe,
                    id: "".into(),
                    matching_strategy: Some(MatchingStrategy::Equals.into()),
                }];
            }
            Message::ComposingAddToNewRule => todo!(),
            Message::ToggleRuleHover(_, _) => todo!(),
            Message::ToggleRuleEdit(_, _) => todo!(),
            Message::ChangeRuleKind(_, _, _) => todo!(),
            Message::ChangeRuleId(_, _, _) => todo!(),
            Message::ChangeRuleMatchingStrategy(_, _, _) => todo!(),
            Message::ComposingAddToRule(_) => todo!(),
            Message::RemoveRule(_) => todo!(),
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
                    .style(button::secondary)
                    .into();

            let new_rule: Element<_> = if self.show_new_rule {
                let rls = self
                    .new_rule
                    .iter()
                    .enumerate()
                    .fold(column![].spacing(10), |col, (idx, rule)| {
                        col.push(rule_view(idx, rule))
                    });
                opt_helpers::opt_box(column!["Match any window where:", rls].spacing(10)).into()
            } else {
                Space::new(Shrink, Shrink).into()
            };

            let rls = rules
                .iter()
                .enumerate()
                .fold(column![].spacing(10), |col, (idx, rule)| match rule {
                    MatchingRule::Simple(rule) => col.push(rule_view(idx, rule)),
                    MatchingRule::Composite(rules) => {
                        col.extend(rules.iter().map(|r| rule_view(idx, r)))
                    }
                })
                .into();

            widget::opt_helpers::section_view(title, [add_new_rule_button, new_rule, rls])
        } else {
            let add_new_rule_button =
                widget::button_with_icon(icons::plus_icon(), text("Add New Rule"))
                    .on_press(Message::ToggleShowNewRule)
                    .style(button::secondary)
                    .into();

            let new_rule: Element<_> = if self.show_new_rule {
                let rls = self
                    .new_rule
                    .iter()
                    .enumerate()
                    .fold(column![].spacing(10), |col, (idx, rule)| {
                        col.push(rule_view(idx, rule))
                    });
                opt_helpers::opt_box(column!["Match any window where:", rls].spacing(10)).into()
            } else {
                Space::new(Shrink, Shrink).into()
            };

            widget::opt_helpers::section_view(title, [add_new_rule_button, new_rule])
        }
    }
}

fn rule_view(idx: usize, rule: &IdWithIdentifier) -> Element<Message> {
    let kind = pick_list(
        &APPLICATION_IDENTIFIER_OPTIONS[..],
        Some(rule.kind),
        move |v| Message::ChangeNewRuleKind(idx, v),
    );
    let matching_strategy = pick_list(
        &MATCHING_STRATEGY_OPTIONS[..],
        rule.matching_strategy
            .as_ref()
            .map(Into::<MatchingStrategy>::into),
        move |v| Message::ChangeNewRuleMatchingStrategy(idx, Some(v)),
    );
    let id = text_input("", &rule.id).on_input(move |v| Message::ChangeNewRuleId(idx, v));
    row![
        kind,
        matching_strategy,
        id,
        horizontal_space(),
        button(icons::plus_icon().style(|t| text::Style {
            color: t.palette().primary.into()
        }))
        .style(button::text)
        .on_press(Message::AddNewRule),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}
