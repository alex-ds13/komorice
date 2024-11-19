use super::rule::{self, Rule};

use crate::{
    widget::{self, opt_helpers},
    BOLD_FONT,
};

use iced::{
    padding,
    widget::{button, column, container, horizontal_rule, row, scrollable, text},
    Element, Fill, Task,
};
use komorebi::{config_generation::MatchingRule, StaticConfig};

#[derive(Clone, Debug)]
pub enum Message {
    SetScreen(Screen),
    SetMainRulesScreen,
    Rule(rule::Message),
    IgnoreRulesHover(bool),
    FloatingApplicationsHover(bool),
    ManageRulesHover(bool),
    TrayAndMultiWindowApplicationsHover(bool),
    ObjectNameChangeApplicationsHover(bool),
    SlowApplicationIdentifiersHover(bool),
    LayeredApplicationsHover(bool),
    BorderOverflowApplicationsHover(bool),
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
    pub ignore_rules_hovered: bool,
    pub floating_applications_hovered: bool,
    pub manage_rules_hovered: bool,
    pub tray_and_multi_window_applications_hovered: bool,
    pub object_name_change_applications_hovered: bool,
    pub slow_application_identifiers_hovered: bool,
    pub layered_applications_hovered: bool,
    pub border_overflow_applications_hovered: bool,
}

impl Rules {
    pub fn update(
        &mut self,
        config: &mut StaticConfig,
        message: Message,
    ) -> (Action, Task<Message>) {
        match message {
            Message::SetScreen(screen) => {
                let rule = Rule::new(&config.ignore_rules);
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
            Message::IgnoreRulesHover(hover) => {
                self.ignore_rules_hovered = hover;
            }
            Message::FloatingApplicationsHover(hover) => {
                self.floating_applications_hovered = hover;
            }
            Message::ManageRulesHover(hover) => {
                self.manage_rules_hovered = hover;
            }
            Message::TrayAndMultiWindowApplicationsHover(hover) => {
                self.tray_and_multi_window_applications_hovered = hover;
            }
            Message::ObjectNameChangeApplicationsHover(hover) => {
                self.object_name_change_applications_hovered = hover;
            }
            Message::SlowApplicationIdentifiersHover(hover) => {
                self.slow_application_identifiers_hovered = hover;
            }
            Message::LayeredApplicationsHover(hover) => {
                self.layered_applications_hovered = hover;
            }
            Message::BorderOverflowApplicationsHover(hover) => {
                self.border_overflow_applications_hovered = hover;
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: &'a StaticConfig) -> Element<'a, Message> {
        if let Some((rule, screen)) = &self.rule_screen {
            let title = row![
                nav_button(text!("{}", self.title()), Message::SetMainRulesScreen),
                text!(" > {}", screen).size(20).font(*BOLD_FONT)
            ];
            let rules = get_rules_from_config(config, screen);
            let content = rule
                .view(iced::widget::value(screen), rules.as_ref())
                .map(Message::Rule);
            column![
                title,
                horizontal_rule(2.0),
                scrollable(
                    container(content)
                        .width(Fill)
                        .padding(padding::top(10).bottom(10).right(20))
                )
            ]
            .spacing(10)
            .into()
        } else {
            let ignore_rules_button = opt_helpers::opt_button(
                iced::widget::value(Screen::IgnoreRules),
                Some(
                    "Individual window ignore rules. Windows ignored \
                    by komorebi will not be hidden and will show on all workspaces.",
                ),
                self.ignore_rules_hovered,
                Message::SetScreen(Screen::IgnoreRules),
                Message::IgnoreRulesHover,
            );
            let floating_applications_button = opt_helpers::opt_button(
                iced::widget::value(Screen::FloatingApplications),
                Some(
                    "Identify applications which should be managed \
                    as floating windows. Floating windows can be tiled later using the \
                    command `toggle-float`, they are held by the workspace and can be \
                    moved to specific workspaces using workspace rules.",
                ),
                self.floating_applications_hovered,
                Message::SetScreen(Screen::FloatingApplications),
                Message::FloatingApplicationsHover,
            );
            let manage_rules_button = opt_helpers::opt_button(
                iced::widget::value(Screen::ManageRules),
                Some(
                    "Individual window force-manage rules. You can use this \
                    to try to force manage some window that is not being managed by komorebi.",
                ),
                self.manage_rules_hovered,
                Message::SetScreen(Screen::ManageRules),
                Message::ManageRulesHover,
            );
            let tray_and_multi_window_applications_button = opt_helpers::opt_button(
                iced::widget::value(Screen::TrayAndMultiWindowApplications),
                Some(
                    "Identify tray and multi-window applications. You can try to \
                    use this for windows that close/minimize to tray or apps that open \
                    multiple windows if they are not behaving correctly.",
                ),
                self.tray_and_multi_window_applications_hovered,
                Message::SetScreen(Screen::TrayAndMultiWindowApplications),
                Message::TrayAndMultiWindowApplicationsHover,
            );
            let object_name_change_apps_button = opt_helpers::opt_button(
                iced::widget::value(Screen::ObjectNameChangeApplications),
                Some(
                    "Identify applications that send EVENT_OBJECT_NAME_CHANGE \
                    on launch (very rare).",
                ),
                self.object_name_change_applications_hovered,
                Message::SetScreen(Screen::ObjectNameChangeApplications),
                Message::ObjectNameChangeApplicationsHover,
            );
            let slow_application_identifiers_button = opt_helpers::opt_button(
                iced::widget::value(Screen::SlowApplicationIdentifiers),
                Some("Identify applications which are slow to send initial event notifications."),
                self.slow_application_identifiers_hovered,
                Message::SetScreen(Screen::SlowApplicationIdentifiers),
                Message::SlowApplicationIdentifiersHover,
            );
            let layered_applications_button = opt_helpers::opt_button(
                iced::widget::value(Screen::LayeredApplications),
                Some("Identify applications that have the WS_EXLAYERED extended window style."),
                self.layered_applications_hovered,
                Message::SetScreen(Screen::LayeredApplications),
                Message::LayeredApplicationsHover,
            );
            let border_overflow_applications_button = opt_helpers::opt_button(
                iced::widget::value(Screen::BorderOverflowApplications),
                Some("Identify border overflow applications."),
                self.border_overflow_applications_hovered,
                Message::SetScreen(Screen::BorderOverflowApplications),
                Message::BorderOverflowApplicationsHover,
            );
            widget::opt_helpers::section_view(
                self.title(),
                [
                    ignore_rules_button,
                    floating_applications_button,
                    manage_rules_button,
                    tray_and_multi_window_applications_button,
                    object_name_change_apps_button,
                    slow_application_identifiers_button,
                    layered_applications_button,
                    border_overflow_applications_button,
                ],
            )
        }
    }

    pub fn title(&self) -> &str {
        "Rules"
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
