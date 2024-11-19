use crate::widget::{self, icons, opt_helpers};

use std::collections::HashMap;

use iced::{
    padding,
    widget::{column, pick_list, row, text, Column, Space},
    Center, Element, Shrink, Task,
};
use komorebi::{
    config_generation::{
        IdWithIdentifier, IdWithIdentifierAndComment, MatchingRule, MatchingStrategy,
    },
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
    pub new_rule: Vec<MatchingRule>,
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
        rules: &mut Option<&mut Vec<MatchingRule>>,
        message: Message,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ToggleRulesExpand => todo!(),
            Message::RulesHover(_) => todo!(),
            Message::ChangeNewRuleKind(_, _) => todo!(),
            Message::ChangeNewRuleId(_, _) => todo!(),
            Message::ChangeNewRuleMatchingStrategy(_, _) => todo!(),
            Message::ToggleShowNewRule => {
                self.new_rule = vec![MatchingRule::Simple(IdWithIdentifier {
                    kind: ApplicationIdentifier::Exe,
                    id: "".into(),
                    matching_strategy: None,
                })];
                self.show_new_rule = !self.show_new_rule;
            },
            Message::AddNewRule => {
                self.new_rule = vec![MatchingRule::Simple(IdWithIdentifier {
                    kind: ApplicationIdentifier::Exe,
                    id: "".into(),
                    matching_strategy: None,
                })];
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

    pub fn view(&self, rules: &Option<&Vec<MatchingRule>>) -> Element<Message> {
        if let Some(rules) = rules {
            let add_new_rule_button =
                widget::button_with_icon(icons::plus_icon(), text("Add New Rule"))
                    .on_press(Message::ToggleShowNewRule);

            let new_rule: Element<_> = if self.show_new_rule {
                let rls = self.new_rule.iter().enumerate().fold(
                    column![].spacing(10),
                    |col, (idx, rule)| match rule {
                        MatchingRule::Simple(rule) => {
                            let pl = pick_list(
                                &APPLICATION_IDENTIFIER_OPTIONS[..],
                                Some(rule.kind),
                                move |v| Message::ChangeNewRuleKind(idx, v),
                            );
                            col.push(row![pl].spacing(10).align_y(Center))
                        }
                        MatchingRule::Composite(rules) => {
                            col.push(rules.iter().fold(column![].spacing(10), move |col, rule| {
                                col.push({
                                    let pl = pick_list(
                                        &APPLICATION_IDENTIFIER_OPTIONS[..],
                                        Some(rule.kind),
                                        move |v| Message::ChangeNewRuleKind(idx, v),
                                    );
                                    row![pl].spacing(10).align_y(Center)
                                })
                            }))
                        }
                    },
                );
                opt_helpers::opt_box(column!["Match any window where:", rls].spacing(10)).into()
            } else {
                Space::new(Shrink, Shrink).into()
            };

            column![add_new_rule_button, new_rule]
                .padding(padding::top(10).bottom(10))
                .spacing(10)
                .into()
        } else {
            let add_new_rule_button =
                widget::button_with_icon(icons::plus_icon(), text("Add New Rule"))
                    .on_press(Message::ToggleShowNewRule);

            let new_rule: Element<_> = if self.show_new_rule {
                let rls = self.new_rule.iter().enumerate().fold(
                    column![].spacing(10),
                    |col, (idx, rule)| match rule {
                        MatchingRule::Simple(rule) => {
                            let pl = pick_list(
                                &APPLICATION_IDENTIFIER_OPTIONS[..],
                                Some(rule.kind),
                                move |v| Message::ChangeNewRuleKind(idx, v),
                            );
                            col.push(row![pl].spacing(10).align_y(Center))
                        }
                        MatchingRule::Composite(rules) => {
                            col.push(rules.iter().fold(column![].spacing(10), move |col, rule| {
                                col.push({
                                    let pl = pick_list(
                                        &APPLICATION_IDENTIFIER_OPTIONS[..],
                                        Some(rule.kind),
                                        move |v| Message::ChangeNewRuleKind(idx, v),
                                    );
                                    row![pl].spacing(10).align_y(Center)
                                })
                            }))
                        }
                    },
                );
                opt_helpers::opt_box(column!["Match any window with:", rls].spacing(10)).into()
            } else {
                Space::new(Shrink, Shrink).into()
            };

            column![add_new_rule_button, new_rule]
                .padding(padding::top(10).bottom(10))
                .spacing(10)
                .into()
        }
    }
}
