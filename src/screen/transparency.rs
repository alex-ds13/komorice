use super::rule::{self, Rule};

use crate::config::DEFAULT_CONFIG;
use crate::{BOLD_FONT, widget::opt_helpers};

use iced::{
    Element, Fill, Subscription, Task, padding,
    widget::{button, column, container, horizontal_rule, row, text},
};
use komorebi_client::{MatchingRule, StaticConfig};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    SetScreen(Screen),
    Rule(rule::Message),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    Transparency(Option<bool>),
    TransparencyAlpha(Option<u8>),
}

#[derive(Clone, Debug, Default)]
pub enum Screen {
    #[default]
    Transparency,
    TransparencyIgnoreRules,
}
impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::Transparency => write!(f, "Transparency"),
            Screen::TransparencyIgnoreRules => write!(f, "Transparency Ignore Rules"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct Transparency {
    pub screen: Screen,
    pub rule: Rule,
}

impl Transparency {
    pub fn update(
        &mut self,
        message: Message,
        config: &mut StaticConfig,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::Transparency(value) => {
                    config.transparency = value;
                }
                ConfigChange::TransparencyAlpha(value) => {
                    config.transparency_alpha = value;
                }
            },
            Message::SetScreen(screen) => {
                if matches!(screen, Screen::TransparencyIgnoreRules) {
                    self.rule = Rule::new();
                }
                self.screen = screen;
            }
            Message::Rule(message) => {
                let rules = get_rules_from_config_mut(config);
                let (action, task) = self.rule.update(rules, message);
                let action_task = match action {
                    rule::Action::None => Task::none(),
                };
                return (
                    Action::None,
                    Task::batch([task.map(Message::Rule), action_task]),
                );
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: &'a StaticConfig) -> Element<'a, Message> {
        match self.screen {
            Screen::Transparency => self.view_transparency(config),
            Screen::TransparencyIgnoreRules => {
                let title = row![
                    nav_button(
                        text("Transparency"),
                        Message::SetScreen(Screen::Transparency)
                    ),
                    text!(" > {}:", self.screen).size(20).font(*BOLD_FONT)
                ];
                let rules = get_rules_from_config(config);
                let content = self.rule.view(rules.as_ref()).map(Message::Rule);
                column![
                    title,
                    horizontal_rule(2.0),
                    container(content)
                        .width(Fill)
                        .padding(padding::top(10).bottom(10))
                ]
                .spacing(10)
                .into()
            }
        }
    }

    pub fn view_transparency<'a>(&'a self, config: &'a StaticConfig) -> Element<'a, Message> {
        opt_helpers::section_view(
            "Transparency:",
            [
                opt_helpers::toggle_with_disable_default(
                    "Transparency",
                    Some("Add transparency to unfocused windows (default: false)"),
                    config.transparency.or(DEFAULT_CONFIG.transparency),
                    DEFAULT_CONFIG.transparency,
                    |value| Message::ConfigChange(ConfigChange::Transparency(value)),
                    None,
                ),
                opt_helpers::number_with_disable_default_option(
                    "Transparency Alpha",
                    Some(
                        "Alpha value for unfocused window transparency [[0-255]] (default: 200)\n\n\
                        Value must be greater or equal to 0.0",
                    ),
                    config
                        .transparency_alpha
                        .or(DEFAULT_CONFIG.transparency_alpha),
                    DEFAULT_CONFIG.transparency_alpha,
                    |value| Message::ConfigChange(ConfigChange::TransparencyAlpha(value)),
                    None,
                ),
                opt_helpers::opt_button(
                    "Transparency Ignore Rules",
                    Some(
                        "Individual window transparency ignore rules. Windows \
                        matched by these rules won't get transparency applied to them.",
                    ),
                    Message::SetScreen(Screen::TransparencyIgnoreRules),
                ),
            ],
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.screen {
            Screen::Transparency => Subscription::none(),
            Screen::TransparencyIgnoreRules => self.rule.subscription().map(Message::Rule),
        }
    }
}

fn get_rules_from_config(config: &StaticConfig) -> &Option<Vec<MatchingRule>> {
    &config.transparency_ignore_rules
}

fn get_rules_from_config_mut(config: &mut StaticConfig) -> &mut Option<Vec<MatchingRule>> {
    &mut config.transparency_ignore_rules
}

fn nav_button<'a>(
    content: impl Into<iced::widget::Text<'a>>,
    on_press: Message,
) -> iced::widget::Button<'a, Message> {
    button(content.into().size(20).font(*BOLD_FONT))
        .on_press(on_press)
        .padding(0)
        .style(button::text)
}
