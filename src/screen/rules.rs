use super::rule::{self, Rule};

use crate::{BOLD_FONT, ITALIC_FONT, widget::opt_helpers};

use iced::{
    Center, Element, Fill, Subscription, Task, padding,
    widget::{Column, button, column, container, horizontal_rule, row, scrollable, text},
};
use komorebi_client::{MatchingRule, StaticConfig};

#[derive(Clone, Debug)]
pub enum Message {
    SetScreen(Screen),
    SetMainRulesScreen,
    Rule(rule::Message),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Debug)]
pub enum Screen {
    IgnoreRules,
    FloatingApplications,
    ManageRules,
    TrayAndMultiWindowApplications,
    ObjectNameChangeApplications,
    SlowApplicationIdentifiers,
    LayeredApplications,
    BorderOverflowApplications,
}

impl Screen {
    fn to_str(&self) -> &'static str {
        match self {
            Screen::IgnoreRules => "Ignore Rules",
            Screen::FloatingApplications => "Floating Applications Rules",
            Screen::ManageRules => "Manage Rules",
            Screen::TrayAndMultiWindowApplications => "Tray and Multi Window Applications",
            Screen::ObjectNameChangeApplications => "Object Name Change Applications",
            Screen::SlowApplicationIdentifiers => "Slow Application Identifiers",
            Screen::LayeredApplications => "Layered Applications",
            Screen::BorderOverflowApplications => "Border Overflow Applications",
        }
    }
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::IgnoreRules => write!(f, "Ignore Rules"),
            Screen::FloatingApplications => write!(f, "Floating Applications Rules"),
            Screen::ManageRules => write!(f, "Manage Rules"),
            Screen::TrayAndMultiWindowApplications => {
                write!(f, "Tray and Multi Window Applications")
            }
            Screen::ObjectNameChangeApplications => write!(f, "Object Name Change Applications"),
            Screen::SlowApplicationIdentifiers => write!(f, "Slow Application Identifiers"),
            Screen::LayeredApplications => write!(f, "Layered Applications"),
            Screen::BorderOverflowApplications => write!(f, "Border Overflow Applications"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Rules {
    pub rule_screen: Option<(Rule, Screen)>,
}

impl Rules {
    pub fn update(
        &mut self,
        message: Message,
        config: &mut StaticConfig,
    ) -> (Action, Task<Message>) {
        match message {
            Message::SetScreen(screen) => {
                let rule = Rule::new();
                self.rule_screen = Some((rule, screen));
            }
            Message::SetMainRulesScreen => {
                self.rule_screen = None;
            }
            Message::Rule(message) => {
                if let Some((rule, screen)) = &mut self.rule_screen {
                    let rules = get_rules_from_config_mut(config, screen);
                    let (action, task) = rule.update(rules, message);
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

    pub fn view<'a>(
        &'a self,
        config: &'a StaticConfig,
        show_advanced: bool,
    ) -> Element<'a, Message> {
        if let Some((rule, screen)) = &self.rule_screen {
            let title = row![
                nav_button(text!("{}", self.title()), Message::SetMainRulesScreen),
                text!(" > {}:", screen).size(20).font(*BOLD_FONT)
            ];
            let rules = get_rules_from_config(config, screen);
            let content = rule.view(rules.as_ref()).map(Message::Rule);
            column![
                title,
                horizontal_rule(2.0),
                container(content)
                    .width(Fill)
                    .padding(padding::top(10).bottom(10))
            ]
            .spacing(10)
            .into()
        } else {
            let ignore_rules_button = opt_helpers::opt_button(
                Screen::IgnoreRules.to_str(),
                Some(
                    "Individual window ignore rules. Windows ignored \
                    by komorebi will not be hidden and will show on all workspaces.",
                ),
                Message::SetScreen(Screen::IgnoreRules),
            );
            let floating_applications_button = opt_helpers::opt_button(
                Screen::FloatingApplications.to_str(),
                Some(
                    "Identify applications which should be managed \
                    as floating windows. Floating windows can be tiled later using the \
                    command `toggle-float`, they are held by the workspace and can be \
                    moved to specific workspaces using workspace rules.",
                ),
                Message::SetScreen(Screen::FloatingApplications),
            );
            let manage_rules_button = opt_helpers::opt_button(
                Screen::ManageRules.to_str(),
                Some(
                    "Individual window force-manage rules. You can use this \
                    to try to force manage some window that is not being managed by komorebi.",
                ),
                Message::SetScreen(Screen::ManageRules),
            );
            let tray_and_multi_window_applications_button = opt_helpers::opt_button(
                Screen::TrayAndMultiWindowApplications.to_str(),
                Some(
                    "Identify tray and multi-window applications. You can try to \
                    use this for windows that close/minimize to tray or apps that open \
                    multiple windows if they are not behaving correctly.",
                ),
                Message::SetScreen(Screen::TrayAndMultiWindowApplications),
            );
            let object_name_change_apps_button = opt_helpers::opt_button(
                Screen::ObjectNameChangeApplications.to_str(),
                Some(
                    "Identify applications that send EVENT_OBJECT_NAME_CHANGE \
                    on launch (very rare).",
                ),
                Message::SetScreen(Screen::ObjectNameChangeApplications),
            );
            let slow_application_identifiers_button = opt_helpers::opt_button(
                Screen::SlowApplicationIdentifiers.to_str(),
                Some("Identify applications which are slow to send initial event notifications."),
                Message::SetScreen(Screen::SlowApplicationIdentifiers),
            );
            let layered_applications_button = opt_helpers::opt_button(
                Screen::LayeredApplications.to_str(),
                Some("Identify applications that have the WS_EXLAYERED extended window style."),
                Message::SetScreen(Screen::LayeredApplications),
            );
            let border_overflow_applications_button = opt_helpers::opt_button(
                Screen::BorderOverflowApplications.to_str(),
                Some("Identify border overflow applications."),
                Message::SetScreen(Screen::BorderOverflowApplications),
            );
            let mut children = vec![
                ignore_rules_button,
                floating_applications_button,
                manage_rules_button,
            ];
            if show_advanced {
                children.extend([
                    row![
                        text("Advanced Rules:").size(18).font(*BOLD_FONT),
                        text("(You shouldn't need to mess with these ones...)")
                            .size(12)
                            .font(*ITALIC_FONT)
                    ]
                    .padding(padding::top(20))
                    .spacing(5)
                    .align_y(Center)
                    .into(),
                    horizontal_rule(2.0).into(),
                    tray_and_multi_window_applications_button,
                    object_name_change_apps_button,
                    slow_application_identifiers_button,
                    layered_applications_button,
                    border_overflow_applications_button,
                ]);
            }
            column![
                text!("{}:", self.title()).size(20).font(*BOLD_FONT),
                horizontal_rule(2.0),
                scrollable(
                    Column::with_children(children)
                        .spacing(10)
                        .width(Fill)
                        .padding(padding::top(10).bottom(10).right(20))
                )
            ]
            .spacing(10)
            .into()
        }
    }

    pub fn title(&self) -> &str {
        "Rules"
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match &self.rule_screen {
            Some((rule, _)) => rule.subscription().map(Message::Rule),
            None => Subscription::none(),
        }
    }
}

fn get_rules_from_config<'a>(
    config: &'a StaticConfig,
    screen: &'a Screen,
) -> &'a Option<Vec<MatchingRule>> {
    match screen {
        Screen::IgnoreRules => &config.ignore_rules,
        Screen::FloatingApplications => &config.floating_applications,
        Screen::ManageRules => &config.manage_rules,
        Screen::TrayAndMultiWindowApplications => &config.tray_and_multi_window_applications,
        Screen::ObjectNameChangeApplications => &config.object_name_change_applications,
        Screen::SlowApplicationIdentifiers => &config.slow_application_identifiers,
        Screen::LayeredApplications => &config.layered_applications,
        Screen::BorderOverflowApplications => &config.border_overflow_applications,
    }
}

fn get_rules_from_config_mut<'a>(
    config: &'a mut StaticConfig,
    screen: &'a Screen,
) -> &'a mut Option<Vec<MatchingRule>> {
    match screen {
        Screen::IgnoreRules => &mut config.ignore_rules,
        Screen::FloatingApplications => &mut config.floating_applications,
        Screen::ManageRules => &mut config.manage_rules,
        Screen::TrayAndMultiWindowApplications => &mut config.tray_and_multi_window_applications,
        Screen::ObjectNameChangeApplications => &mut config.object_name_change_applications,
        Screen::SlowApplicationIdentifiers => &mut config.slow_application_identifiers,
        Screen::LayeredApplications => &mut config.layered_applications,
        Screen::BorderOverflowApplications => &mut config.border_overflow_applications,
    }
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
