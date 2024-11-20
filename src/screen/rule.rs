use crate::widget::{self, button_with_icon, icons, opt_helpers};

use iced::{
    padding,
    widget::{
        button, column, container, horizontal_rule, pick_list, row, text, text_input, Row, Space,
    },
    Center, Element, Fill, Right, Shrink, Task, Top,
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
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct Rule {
    pub show_new_rule: bool,
    pub new_rule: Vec<IdWithIdentifier>,
    pub rules_edit: Vec<bool>,
}

impl Rule {
    pub fn new(rules: &Option<Vec<MatchingRule>>) -> Self {
        Rule {
            show_new_rule: false,
            new_rule: Vec::new(),
            rules_edit: rules.as_ref().map_or(Vec::new(), |rules| {
                let count = rules.len();
                vec![false; count]
            }),
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
                        self.rules_edit.push(false);
                    } else {
                        let rule = MatchingRule::Simple(self.new_rule.remove(0));
                        *rules = Some(vec![rule]);
                        self.rules_edit = vec![false];
                    }
                } else if let Some(rules) = rules {
                    let rule = MatchingRule::Composite(self.new_rule.drain(..).collect());
                    rules.push(rule);
                    self.rules_edit.push(false);
                } else {
                    let rule = MatchingRule::Composite(self.new_rule.drain(..).collect());
                    *rules = Some(vec![rule]);
                    self.rules_edit = vec![false];
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
                if let (Some(_rule), Some(rule_edit)) = (
                    rules.as_mut().and_then(|rls| rls.get_mut(idx)),
                    self.rules_edit.get_mut(idx),
                ) {
                    *rule_edit = edit;
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
                if let Some(rules) = rules {
                    if rules.get(idx).is_some() {
                        rules.remove(idx);
                        self.rules_edit.remove(idx);
                    }
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, rules: Option<&'a Vec<MatchingRule>>) -> Element<'a, Message> {
        let add_new_rule_button =
            widget::button_with_icon(icons::plus_icon(), text("Add New Rule"))
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
            let add_rule_button = button_with_icon(icons::plus_icon(), "Add")
                .on_press(Message::AddNewRule)
                .width(75);
            opt_helpers::opt_box(
                row![
                    column!["Match any window where:", rls].spacing(10),
                    add_rule_button
                ]
                .spacing(10)
                .align_y(Top),
            )
            .into()
        } else {
            Space::new(Shrink, Shrink).into()
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
                                .push(
                                    rule_view(
                                        rule,
                                        self.rules_edit[idx],
                                        self.rules_edit[idx],
                                        move |v| Message::ChangeRuleKind(idx, 0, v),
                                        move |v| {
                                            Message::ChangeRuleMatchingStrategy(idx, 0, Some(v))
                                        },
                                        move |v| Message::ChangeRuleId(idx, 0, v),
                                        Message::ComposingAddToRule(idx),
                                        None,
                                    )
                                    .width(635),
                                )
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
                                        col.push(
                                            rule_view(
                                                r,
                                                if self.rules_edit[idx] {
                                                    i == rules.len() - 1
                                                } else {
                                                    i != rules.len() - 1
                                                },
                                                self.rules_edit[idx],
                                                move |v| Message::ChangeRuleKind(idx, i, v),
                                                move |v| {
                                                    Message::ChangeRuleMatchingStrategy(
                                                        idx,
                                                        i,
                                                        Some(v),
                                                    )
                                                },
                                                move |v| Message::ChangeRuleId(idx, i, v),
                                                Message::ComposingAddToRule(idx),
                                                if i != 0 {
                                                    Some(Message::ComposingRemoveFromRule(idx, i))
                                                } else {
                                                    None
                                                },
                                            )
                                            .width(635),
                                        )
                                    },
                                )
                                .into(),
                        ),
                    ),
                });
            rls.into()
        } else {
            Space::new(Shrink, Shrink).into()
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
                container(opt_helpers::opt_box(content).style(opt_helpers::opt_box_style_bottom))
                    .padding(padding::right(170))
                    .into(),
                column![row![]
                    .push_maybe(
                        self.rules_edit[idx].then_some(
                            button(icons::check_icon())
                                .on_press(Message::ToggleRuleEdit(idx, false))
                                .style(button::primary),
                        )
                    )
                    .push_maybe(
                        self.rules_edit[idx].then_some(
                            button(icons::delete_icon())
                                .on_press(Message::RemoveRule(idx))
                                .style(button::danger),
                        )
                    )
                    .spacing(10)
                    .align_y(Center)
                    .width(160)
                    .height(Fill)]
                .width(Fill)
                .align_x(Right)
                .into(),
            ]),
            column![row![]
                .push_maybe(
                    (!self.rules_edit[idx]).then_some(
                        button(icons::edit_icon())
                            .on_press(Message::ToggleRuleEdit(idx, true))
                            .style(button::secondary),
                    )
                )
                .spacing(10)
                .align_y(Center)
                .width(160)
                .height(Fill)]
            .width(Fill)
            .align_x(Right),
        )
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
            rule.matching_strategy
                .as_ref()
                .map(Into::<MatchingStrategy>::into),
            change_matching_strategy,
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
        text_input("", &rule.id).on_input(change_id).into()
    } else {
        container(text(&rule.id))
            .padding(5)
            .style(container::dark)
            .into()
    };
    let composing_add_button: Option<Element<_>> = show_and
        .then_some(if edit {
            button(
                row![icons::level_down_icon(), "And"]
                    .spacing(5)
                    .align_y(Center),
            )
            .style(button::secondary)
            .on_press(composing_add)
            .into()
        } else {
            container(
                row![icons::level_down_icon(), "And"]
                    .spacing(5)
                    .align_y(Center),
            )
            .padding(padding::all(5).right(10).left(10))
            .into()
        })
        .or((!show_and && edit).then_some(
            container(
                row![icons::level_down_icon(), "And"]
                    .spacing(5)
                    .align_y(Center),
            )
            .padding(padding::all(5).right(10).left(10))
            .into(),
        ));

    let delete_rule_line_button: Option<Element<_>> = edit
        .then_some(composing_remove.map(|m| {
            button(icons::delete_icon())
                .on_press(m)
                .style(button::danger)
                .into()
        }))
        .flatten()
        .or_else(|| Some(Space::new(32, Shrink).into()));

    row![kind, matching_strategy, id]
        .push_maybe(composing_add_button)
        .push_maybe(delete_rule_line_button)
        .spacing(10)
        .width(550)
        .align_y(Center)
}

fn default_rule() -> IdWithIdentifier {
    IdWithIdentifier {
        kind: ApplicationIdentifier::Exe,
        id: "".into(),
        matching_strategy: Some(MatchingStrategy::Equals.into()),
    }
}
