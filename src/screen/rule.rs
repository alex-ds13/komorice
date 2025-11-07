use crate::widget::{self, button_with_icon, icons, opt_helpers};

use std::collections::HashSet;

use iced::{
    Center, Element, Fill, Right, Shrink, Subscription, Task, Top, padding,
    widget::{Row, space, button, column, container, pick_list, row, text, text_input},
};
use komorebi_client::{ApplicationIdentifier, IdWithIdentifier, MatchingRule, MatchingStrategy};
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

#[derive(Clone, Debug)]
pub enum Message {
    ChangeNewRuleKind(usize, ApplicationIdentifier),
    ChangeNewRuleId(usize, String),
    ChangeNewRuleMatchingStrategy(usize, Option<MatchingStrategy>),

    ToggleShowNewRule,

    AddNewRule,
    ComposingAddToNewRule,
    ComposingRemoveFromNewRule(usize),

    ToggleRuleEdit(usize, bool),

    ChangeRuleKind(usize, usize, ApplicationIdentifier),
    ChangeRuleId(usize, usize, String),
    ChangeRuleMatchingStrategy(usize, usize, Option<MatchingStrategy>),
    ComposingAddToRule(usize),
    ComposingRemoveFromRule(usize, usize),

    RemoveRule(usize),

    CopyRule(usize),
    CopyNewRule,
    PasteRule,
    ClipboardHasRule(bool),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Debug, Default)]
pub struct Rule {
    pub show_new_rule: bool,
    pub new_rule: Vec<IdWithIdentifier>,
    pub rules_editing: HashSet<usize>,
    pub clipboard_has_rule: bool,
}

impl Rule {
    pub fn new() -> Self {
        Rule {
            show_new_rule: false,
            new_rule: Vec::new(),
            rules_editing: HashSet::new(),
            clipboard_has_rule: false,
        }
    }

    pub fn update(
        &mut self,
        rules: &mut Option<Vec<MatchingRule>>,
        message: Message,
    ) -> (Action, Task<Message>) {
        match message {
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
                    rule.matching_strategy = matching_strategy;
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
                self.new_rule = vec![default_rule()];
            }
            Message::ComposingAddToNewRule => {
                self.new_rule.push(default_rule());
            }
            Message::ComposingRemoveFromNewRule(idx) => {
                self.new_rule.remove(idx);
            }
            Message::ToggleRuleEdit(idx, edit) => {
                if edit {
                    self.rules_editing.insert(idx);
                } else {
                    self.rules_editing.remove(&idx);
                }
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
                            rule.matching_strategy = matching_strategy;
                        }
                        MatchingRule::Composite(rules) => {
                            if let Some(rule) = rules.get_mut(sub_idx) {
                                rule.matching_strategy = matching_strategy;
                            }
                        }
                    }
                }
            }
            Message::ComposingAddToRule(idx) => {
                if let Some(rules) = rules {
                    let rule = rules.remove(idx);
                    let changed_rule = match rule {
                        MatchingRule::Simple(r) => MatchingRule::Composite(vec![r, default_rule()]),
                        MatchingRule::Composite(mut rls) => {
                            rls.push(default_rule());
                            MatchingRule::Composite(rls)
                        }
                    };
                    rules.insert(idx, changed_rule);
                }
            }
            Message::ComposingRemoveFromRule(idx, sub_idx) => {
                if let Some(rules) = rules {
                    let rule = rules.remove(idx);
                    let changed_rule = match rule {
                        MatchingRule::Simple(_) => rule,
                        MatchingRule::Composite(mut rls) => {
                            rls.remove(sub_idx);
                            if rls.len() == 1 {
                                MatchingRule::Simple(rls.remove(0))
                            } else {
                                MatchingRule::Composite(rls)
                            }
                        }
                    };
                    rules.insert(idx, changed_rule);
                }
            }
            Message::RemoveRule(idx) => {
                if let Some(rules) = rules
                    && rules.get(idx).is_some()
                {
                    rules.remove(idx);
                }
                self.rules_editing.remove(&idx);
            }
            Message::CopyRule(idx) => {
                if let Some(rule) = rules.as_mut().and_then(|rls| rls.get_mut(idx))
                    && let Ok(rule_str) = serde_json::to_string_pretty(&rule)
                {
                    let _ = clipboard_win::set_clipboard_string(&rule_str);
                }
            }
            Message::CopyNewRule => {
                let rule = if self.new_rule.len() == 1 {
                    MatchingRule::Simple(self.new_rule[0].clone())
                } else {
                    MatchingRule::Composite(self.new_rule.clone())
                };

                if let Ok(rule_str) = serde_json::to_string_pretty(&rule) {
                    let _ = clipboard_win::set_clipboard_string(&rule_str);
                }
            }
            Message::PasteRule => {
                if let Ok(content) = clipboard_win::get_clipboard_string()
                    && let Ok(rule) = serde_json::from_str::<MatchingRule>(&content)
                {
                    match rule {
                        MatchingRule::Simple(rule) => self.new_rule = vec![rule],
                        MatchingRule::Composite(rls) => self.new_rule = rls,
                    }
                }
            }
            Message::ClipboardHasRule(has_rule) => {
                self.clipboard_has_rule = has_rule;
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, rules: Option<&'a Vec<MatchingRule>>) -> Element<'a, Message> {
        let add_new_rule_button = widget::button_with_icon(icons::plus(), text("Add New Rule"))
            .on_press(Message::ToggleShowNewRule)
            .style(button::secondary);

        let new_rule: Element<_> = if self.show_new_rule {
            let rls =
                self.new_rule
                    .iter()
                    .enumerate()
                    .fold(column![].spacing(10), |col, (idx, rule)| {
                        col.push(rule_view(
                            rule,
                            idx == self.new_rule.len() - 1,
                            true,
                            move |v| Message::ChangeNewRuleKind(idx, v),
                            move |v| Message::ChangeNewRuleMatchingStrategy(idx, Some(v)),
                            move |v| Message::ChangeNewRuleId(idx, v),
                            Message::ComposingAddToNewRule,
                            if idx != 0 {
                                Some(Message::ComposingRemoveFromNewRule(idx))
                            } else {
                                None
                            },
                        ))
                    });
            let add_rule_button = button_with_icon(icons::plus(), "Add")
                .on_press(Message::AddNewRule)
                .width(77);
            let copy_button = button(icons::copy())
                .on_press_maybe((!self.new_rule[0].id.is_empty()).then_some(Message::CopyNewRule))
                .style(button::secondary);
            let paste_button = button(icons::paste())
                .on_press_maybe(self.clipboard_has_rule.then_some(Message::PasteRule))
                .style(button::secondary);
            opt_helpers::opt_box(
                row![
                    column!["Match any window where:", rls].spacing(10),
                    column![add_rule_button, row![copy_button, paste_button].spacing(5)]
                        .align_x(Right)
                        .spacing(10),
                ]
                .spacing(10)
                .align_y(Top),
            )
            .max_width(685)
            .into()
        } else {
            space().into()
        };

        let rls: Element<_> = if let Some(rules) = rules {
            let rls = rules
                .iter()
                .enumerate()
                .fold(column![].spacing(10), |col, (idx, rule)| match rule {
                    MatchingRule::Simple(rule) => col.push(
                        self.matching_rule_view(
                            idx,
                            column!["Match any window where:"]
                                .push(rule_view(
                                    rule,
                                    self.rules_editing.contains(&idx),
                                    self.rules_editing.contains(&idx),
                                    move |v| Message::ChangeRuleKind(idx, 0, v),
                                    move |v| Message::ChangeRuleMatchingStrategy(idx, 0, Some(v)),
                                    move |v| Message::ChangeRuleId(idx, 0, v),
                                    Message::ComposingAddToRule(idx),
                                    None,
                                ))
                                .spacing(10)
                                .into(),
                        ),
                    ),
                    MatchingRule::Composite(rules) => col.push(
                        self.matching_rule_view(
                            idx,
                            rules
                                .iter()
                                .enumerate()
                                .fold(
                                    column!["Match any window where:"].spacing(10),
                                    |col, (i, r)| {
                                        col.push(rule_view(
                                            r,
                                            if self.rules_editing.contains(&idx) {
                                                i == rules.len() - 1
                                            } else {
                                                i != rules.len() - 1
                                            },
                                            self.rules_editing.contains(&idx),
                                            move |v| Message::ChangeRuleKind(idx, i, v),
                                            move |v| {
                                                Message::ChangeRuleMatchingStrategy(idx, i, Some(v))
                                            },
                                            move |v| Message::ChangeRuleId(idx, i, v),
                                            Message::ComposingAddToRule(idx),
                                            if i != 0 {
                                                Some(Message::ComposingRemoveFromRule(idx, i))
                                            } else {
                                                None
                                            },
                                        ))
                                    },
                                )
                                .into(),
                        ),
                    ),
                });
            rls.into()
        } else {
            space().into()
        };

        column![
            add_new_rule_button,
            new_rule,
            opt_helpers::section_view("Rules:", [rls])
        ]
        .spacing(10)
        .into()
    }

    fn matching_rule_view<'a>(
        &'a self,
        idx: usize,
        content: Element<'a, Message>,
    ) -> Element<'a, Message> {
        iced::widget::hover(
            iced::widget::stack([
                container(
                    opt_helpers::opt_box(content)
                        .max_width(685)
                        .style(opt_helpers::opt_box_style_bottom),
                )
                .width(Shrink)
                .padding(padding::right(90))
                .into(),
                column![
                    row![]
                        .push(
                            self.rules_editing.contains(&idx).then_some(
                                button(icons::check())
                                    .on_press(Message::ToggleRuleEdit(idx, false))
                                    .style(button::primary),
                            )
                        )
                        .push(
                            self.rules_editing.contains(&idx).then_some(
                                button(icons::delete())
                                    .on_press(Message::RemoveRule(idx))
                                    .style(button::danger),
                            )
                        )
                        .spacing(10)
                        .align_y(Center)
                        .width(80)
                        .height(Fill)
                ]
                .width(Fill)
                .align_x(Right)
                .into(),
            ]),
            column![
                row![]
                    .push(
                        (!self.rules_editing.contains(&idx)).then_some(
                            button(icons::edit())
                                .on_press(Message::ToggleRuleEdit(idx, true))
                                .style(button::secondary),
                        )
                    )
                    .push(
                        (!self.rules_editing.contains(&idx)).then_some(
                            button(icons::copy())
                                .on_press(Message::CopyRule(idx))
                                .style(button::secondary),
                        )
                    )
                    .spacing(10)
                    .align_y(Center)
                    .width(80)
                    .height(Fill)
            ]
            .width(Fill)
            .align_x(Right),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.show_new_rule {
            iced::time::every(std::time::Duration::from_millis(250))
                .map(|_| {
                    if let Ok(content) = clipboard_win::get_clipboard_string() {
                        return serde_json::from_str::<MatchingRule>(&content).is_ok();
                    }
                    false
                })
                .map(Message::ClipboardHasRule)
        } else {
            Subscription::none()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn rule_view<'a>(
    rule: &'a IdWithIdentifier,
    show_and: bool,
    edit: bool,
    change_kind: impl Fn(ApplicationIdentifier) -> Message + 'a,
    change_matching_strategy: impl Fn(MatchingStrategy) -> Message + 'a,
    change_id: impl Fn(String) -> Message + 'a,
    composing_add: Message,
    composing_remove: Option<Message>,
) -> Row<'a, Message> {
    let kind: Element<_> = if edit {
        pick_list(
            &APPLICATION_IDENTIFIER_OPTIONS[..],
            Some(rule.kind),
            change_kind,
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
            rule.matching_strategy.as_ref(),
            change_matching_strategy,
        )
        .width(Fill)
        .into()
    } else {
        row![
            container(iced::widget::value(
                rule.matching_strategy
                    .as_ref()
                    .unwrap_or(&MatchingStrategy::Legacy),
            ))
            .width(Fill)
            .padding(5)
            .style(container::dark),
            "→"
        ]
        .spacing(10)
        .align_y(Center)
        .into()
    };
    let id: Element<_> = if edit {
        text_input("", &rule.id)
            .on_input(change_id)
            .width(Fill)
            .into()
    } else {
        container(text(&rule.id))
            .padding(5)
            .width(Fill)
            .style(container::dark)
            .into()
    };
    let composing_add_button: Option<Element<_>> = show_and
        .then_some(if edit {
            button(row![icons::level_down(), "And"].spacing(5).align_y(Center))
                .style(button::secondary)
                .on_press(composing_add)
                .into()
        } else {
            container(row![icons::level_down(), "And"].spacing(5).align_y(Center))
                .padding(padding::all(5).right(10).left(10))
                .into()
        })
        .or((!show_and && edit).then_some(
            container(row![icons::level_down(), "And"].spacing(5).align_y(Center))
                .padding(padding::all(5).right(10).left(10))
                .into(),
        ));

    let delete_rule_line_button: Option<Element<_>> = edit
        .then_some(
            composing_remove
                .map(|m| {
                    button(icons::delete())
                        .on_press(m)
                        .style(button::danger)
                        .into()
                })
                .or_else(|| Some(space().width(33).into())),
        )
        .flatten();

    row![kind, matching_strategy, id]
        .push(composing_add_button)
        .push(delete_rule_line_button)
        .spacing(10)
        .align_y(Center)
}

fn default_rule() -> IdWithIdentifier {
    IdWithIdentifier {
        kind: ApplicationIdentifier::Exe,
        id: "".into(),
        matching_strategy: Some(MatchingStrategy::Equals),
    }
}
