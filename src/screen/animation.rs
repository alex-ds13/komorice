use std::collections::HashMap;

use crate::config::DEFAULT_CONFIG;
use crate::widget::opt_helpers;

use iced::{
    widget::{column, horizontal_space, pick_list, row},
    Center, Element, Task,
};
use komorebi_client::{
    AnimationPrefix, AnimationStyle, AnimationsConfig, PerAnimationPrefixConfig,
};

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleEnableExpand,
    ToggleEnableHover(bool),
    ToggleEnableConfigType(ConfigType),
    ToggleDurationExpand,
    ToggleDurationHover(bool),
    ToggleDurationConfigType(ConfigType),
    ToggleStyleExpand,
    ToggleStyleHover(bool),
    ToggleStyleConfigType(ConfigType),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    EnableGlobal(bool),
    EnablePerType(AnimationPrefix, bool),
    DurationGlobal(Option<u64>),
    DurationPerType(AnimationPrefix, Option<u64>),
    StyleGlobal(Option<AnimationStyle>),
    StylePerType(AnimationPrefix, Option<AnimationStyle>),
    Fps(Option<u64>),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum ConfigType {
    #[default]
    Global,
    PerType,
}
impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigType::Global => write!(f, "Global"),
            ConfigType::PerType => write!(f, "Per Type"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Animation {
    pub enable_hovered: bool,
    pub enable_expanded: bool,
    pub duration_hovered: bool,
    pub duration_expanded: bool,
    pub duration_config_type: ConfigType,
    pub style_hovered: bool,
    pub style_expanded: bool,
    pub style_config_type: ConfigType,
}

impl Animation {
    pub fn update(
        &mut self,
        message: Message,
        config: &mut AnimationsConfig,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::EnableGlobal(value) => {
                    config.enabled = PerAnimationPrefixConfig::Global(value);
                }
                ConfigChange::EnablePerType(prefix, value) => {
                    if let PerAnimationPrefixConfig::Prefix(ap) = &mut config.enabled {
                        ap.insert(prefix, value);
                    } else {
                        config.enabled =
                            PerAnimationPrefixConfig::Prefix(HashMap::from([(prefix, value)]));
                    }
                }
                ConfigChange::DurationGlobal(value) => {
                    config.duration = value.map(PerAnimationPrefixConfig::Global);
                }
                ConfigChange::DurationPerType(prefix, value) => {
                    if let Some(PerAnimationPrefixConfig::Prefix(ap)) = &mut config.duration {
                        if let Some(value) = value {
                            ap.insert(prefix, value);
                        } else {
                            ap.remove(&prefix);
                        }
                    } else {
                        config.duration = value.map(|v| {
                            PerAnimationPrefixConfig::Prefix(HashMap::from([(prefix, v)]))
                        });
                    }
                }
                ConfigChange::StyleGlobal(value) => {
                    config.style = value.map(PerAnimationPrefixConfig::Global);
                }
                ConfigChange::StylePerType(prefix, value) => {
                    if let Some(PerAnimationPrefixConfig::Prefix(ap)) = &mut config.style {
                        if let Some(value) = value {
                            ap.insert(prefix, value);
                        } else {
                            ap.remove(&prefix);
                        }
                    } else {
                        config.style = value.map(|v| {
                            PerAnimationPrefixConfig::Prefix(HashMap::from([(prefix, v)]))
                        });
                    }
                }
                ConfigChange::Fps(value) => config.fps = value,
            },
            Message::ToggleEnableExpand => self.enable_expanded = !self.enable_expanded,
            Message::ToggleEnableHover(hover) => self.enable_hovered = hover,
            Message::ToggleEnableConfigType(c_type) => {
                match c_type {
                    ConfigType::Global => {
                        if let PerAnimationPrefixConfig::Prefix(ap) = &config.enabled {
                            // If all animation types were enabled then set the `Global` as `true`
                            // otherwise set it to `false`
                            let global = ap.values().all(|v| *v);
                            config.enabled = PerAnimationPrefixConfig::Global(global);
                        }
                    }
                    ConfigType::PerType => {
                        if let PerAnimationPrefixConfig::Global(global) = config.enabled {
                            // Use the `Global` value on each animation type
                            config.enabled = PerAnimationPrefixConfig::Prefix(HashMap::from([
                                (AnimationPrefix::Movement, global),
                                (AnimationPrefix::Transparency, global),
                            ]));
                        }
                    }
                }
            }
            Message::ToggleDurationExpand => self.duration_expanded = !self.duration_expanded,
            Message::ToggleDurationHover(hover) => self.duration_hovered = hover,
            Message::ToggleDurationConfigType(c_type) => self.duration_config_type = c_type,
            Message::ToggleStyleExpand => self.style_expanded = !self.style_expanded,
            Message::ToggleStyleHover(hover) => self.style_hovered = hover,
            Message::ToggleStyleConfigType(c_type) => self.style_config_type = c_type,
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: Option<&'a AnimationsConfig>) -> Element<'a, Message> {
        let config = if let Some(config) = config {
            config
        } else {
            default_animations_config_ref()
        };
        let enable_config_type = match &config.enabled {
            PerAnimationPrefixConfig::Prefix(_) => ConfigType::PerType,
            PerAnimationPrefixConfig::Global(_) => ConfigType::Global,
        };
        opt_helpers::section_view(
            "Animations:",
            [
                opt_helpers::expandable_with_disable_default(
                    "Enable",
                    Some("Enable/Disable all animations or per type of animation"),
                    [
                        column![
                            opt_helpers::opt_box(
                                row![
                                    column![
                                        pick_list(
                                            [ConfigType::Global, ConfigType::PerType],
                                            Some(enable_config_type.clone()),
                                            Message::ToggleEnableConfigType,
                                        ),
                                        opt_helpers::description_text(if matches!(enable_config_type, ConfigType::Global) {
                                            "Enable/Disable all types of animations"
                                        } else {
                                            "Enable/Disable animations per type of animation"
                                        }),
                                    ].spacing(5),
                                    horizontal_space(),
                                ]
                                .push_maybe(matches!(&config.enabled, PerAnimationPrefixConfig::Global(_)).then(|| -> Element<Message> {
                                    iced::widget::toggler(matches!(&config.enabled, PerAnimationPrefixConfig::Global(v) if v == &true))
                                        .on_toggle(|v| Message::ConfigChange(ConfigChange::EnableGlobal(v)))
                                        .label(match config.enabled {
                                            PerAnimationPrefixConfig::Global(v) => if v { "On" } else { "Off" },
                                            _ => "Off",
                                        })
                                        .into()
                                }))
                                .align_y(Center)
                            )
                        ]
                        .push_maybe(matches!(&enable_config_type, ConfigType::PerType).then(|| {
                            opt_helpers::toggle(
                                "Enable Movement Animations",
                                None,
                                matches!(&config.enabled, PerAnimationPrefixConfig::Prefix(p) if p.get(&AnimationPrefix::Movement).unwrap_or(&false) == &true),
                                |v| Message::ConfigChange(ConfigChange::EnablePerType(AnimationPrefix::Movement, v)),
                            )
                        }))
                        .push_maybe(matches!(&enable_config_type, ConfigType::PerType).then(|| {
                            opt_helpers::toggle(
                                "Enable Transparency Animations",
                                None,
                                matches!(&config.enabled, PerAnimationPrefixConfig::Prefix(p) if p.get(&AnimationPrefix::Transparency).unwrap_or(&false) == &true),
                                |v| Message::ConfigChange(ConfigChange::EnablePerType(AnimationPrefix::Transparency, v)),
                            )
                        }))
                        .spacing(10)
                        .into()
                    ],
                    self.enable_expanded,
                    self.enable_hovered,
                    Message::ToggleEnableExpand,
                    Message::ToggleEnableHover,
                    DEFAULT_CONFIG.animation.as_ref().map(|a| a.enabled != config.enabled).unwrap_or_default(),
                    Message::ConfigChange(ConfigChange::EnableGlobal(false)),
                    None,
                ),
            ],
        )
    }
}

pub fn default_animations_config() -> AnimationsConfig {
    AnimationsConfig {
        enabled: PerAnimationPrefixConfig::Global(false),
        duration: None,
        style: None,
        fps: None,
    }
}

pub fn default_animations_config_ref() -> &'static AnimationsConfig {
    &AnimationsConfig {
        enabled: PerAnimationPrefixConfig::Global(false),
        duration: None,
        style: None,
        fps: None,
    }
}
