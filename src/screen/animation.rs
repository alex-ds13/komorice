use crate::{
    config::DEFAULT_CONFIG,
    widget::{number_input, opt_helpers},
};

use std::collections::HashMap;

use iced::{
    Center, Element, Task,
    widget::{column, pick_list, row, space},
};
use komorebi_client::{
    AnimationPrefix, AnimationStyle, AnimationsConfig, PerAnimationPrefixConfig,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref ALL_ANIMATIONS_STYLES: [AnimationStyle; 30] = [
        AnimationStyle::Linear,
        AnimationStyle::EaseInSine,
        AnimationStyle::EaseOutSine,
        AnimationStyle::EaseInOutSine,
        AnimationStyle::EaseInQuad,
        AnimationStyle::EaseOutQuad,
        AnimationStyle::EaseInOutQuad,
        AnimationStyle::EaseInCubic,
        AnimationStyle::EaseInOutCubic,
        AnimationStyle::EaseInQuart,
        AnimationStyle::EaseOutQuart,
        AnimationStyle::EaseInOutQuart,
        AnimationStyle::EaseInQuint,
        AnimationStyle::EaseOutQuint,
        AnimationStyle::EaseInOutQuint,
        AnimationStyle::EaseInExpo,
        AnimationStyle::EaseOutExpo,
        AnimationStyle::EaseInOutExpo,
        AnimationStyle::EaseInCirc,
        AnimationStyle::EaseOutCirc,
        AnimationStyle::EaseInOutCirc,
        AnimationStyle::EaseInBack,
        AnimationStyle::EaseOutBack,
        AnimationStyle::EaseInOutBack,
        AnimationStyle::EaseInElastic,
        AnimationStyle::EaseOutElastic,
        AnimationStyle::EaseInOutElastic,
        AnimationStyle::EaseInBounce,
        AnimationStyle::EaseOutBounce,
        AnimationStyle::EaseInOutBounce,
    ];
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleEnableConfigType(ConfigType),
    ToggleDurationConfigType(ConfigType),
    ToggleStyleConfigType(ConfigType),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    EnableGlobal(bool),
    EnablePerType(AnimationPrefix, bool),
    DurationGlobal(u64),
    DurationPerType(AnimationPrefix, u64),
    StyleGlobal(AnimationStyle),
    StylePerType(AnimationPrefix, AnimationStyle),
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
pub struct Animation;

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
                    config.duration = Some(PerAnimationPrefixConfig::Global(value));
                }
                ConfigChange::DurationPerType(prefix, value) => {
                    if let Some(PerAnimationPrefixConfig::Prefix(ap)) = &mut config.duration {
                        ap.insert(prefix, value);
                    } else {
                        config.duration =
                            Some(PerAnimationPrefixConfig::Prefix(HashMap::from([(
                                prefix, value,
                            )])));
                    }
                }
                ConfigChange::StyleGlobal(value) => {
                    config.style = Some(PerAnimationPrefixConfig::Global(value));
                }
                ConfigChange::StylePerType(prefix, value) => {
                    if let Some(PerAnimationPrefixConfig::Prefix(ap)) = &mut config.style {
                        ap.insert(prefix, value);
                    } else {
                        config.style = Some(PerAnimationPrefixConfig::Prefix(HashMap::from([(
                            prefix, value,
                        )])));
                    }
                }
                ConfigChange::Fps(value) => {
                    config.fps = value;
                }
            },
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
            Message::ToggleDurationConfigType(c_type) => {
                match c_type {
                    ConfigType::Global => {
                        if let Some(PerAnimationPrefixConfig::Prefix(ap)) = &config.duration {
                            // If all animation types duration was the same then set the `Global`
                            // with that value, otherwise set it to default value
                            let duration = ap.values().next().and_then(|duration| {
                                ap.values().all(|v| v == duration).then_some(*duration)
                            });
                            config.duration = duration.map(PerAnimationPrefixConfig::Global);
                        } else {
                            config.duration = DEFAULT_CONFIG
                                .animation
                                .as_ref()
                                .and_then(|a| a.duration.clone());
                        }
                    }
                    ConfigType::PerType => {
                        if let Some(PerAnimationPrefixConfig::Global(global)) = config.duration {
                            // Use the `Global` value on each animation type
                            config.duration =
                                Some(PerAnimationPrefixConfig::Prefix(HashMap::from([
                                    (AnimationPrefix::Movement, global),
                                    (AnimationPrefix::Transparency, global),
                                ])));
                        } else {
                            config.duration =
                                Some(PerAnimationPrefixConfig::Prefix(HashMap::new()));
                        }
                    }
                }
            }
            Message::ToggleStyleConfigType(c_type) => {
                match c_type {
                    ConfigType::Global => {
                        if let Some(PerAnimationPrefixConfig::Prefix(ap)) = &config.style {
                            // If all animation types style was the same then set the `Global`
                            // with that value, otherwise set it to default value
                            let style = ap.values().next().and_then(|style| {
                                ap.values().all(|v| v == style).then_some(*style)
                            });
                            config.style = style.map(PerAnimationPrefixConfig::Global);
                        } else {
                            config.style = DEFAULT_CONFIG
                                .animation
                                .as_ref()
                                .and_then(|a| a.style.clone());
                        }
                    }
                    ConfigType::PerType => {
                        if let Some(PerAnimationPrefixConfig::Global(global)) = config.style {
                            // Use the `Global` value on each animation type
                            config.style = Some(PerAnimationPrefixConfig::Prefix(HashMap::from([
                                (AnimationPrefix::Movement, global),
                                (AnimationPrefix::Transparency, global),
                            ])));
                        } else {
                            config.style = Some(PerAnimationPrefixConfig::Prefix(HashMap::new()));
                        }
                    }
                }
            }
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
        let duration_config_type = match &config.duration {
            Some(PerAnimationPrefixConfig::Prefix(_)) => ConfigType::PerType,
            Some(PerAnimationPrefixConfig::Global(_)) => ConfigType::Global,
            None => ConfigType::Global,
        };
        let style_config_type = match &config.style {
            Some(PerAnimationPrefixConfig::Prefix(_)) => ConfigType::PerType,
            Some(PerAnimationPrefixConfig::Global(_)) => ConfigType::Global,
            None => ConfigType::Global,
        };
        opt_helpers::section_view(
            "Animations:",
            [
                opt_helpers::expandable(
                    "Enable",
                    Some("Enable or disable all animations or per type of animation"),
                    move || {
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
                                    space::horizontal(),
                                ]
                                .push(matches!(&config.enabled, PerAnimationPrefixConfig::Global(_)).then(|| -> Element<Message> {
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
                        .push(matches!(&enable_config_type, ConfigType::PerType).then(|| {
                            opt_helpers::toggle(
                                "Enable Movement Animations",
                                None,
                                matches!(&config.enabled, PerAnimationPrefixConfig::Prefix(p) if p.get(&AnimationPrefix::Movement).unwrap_or(&false) == &true),
                                |v| Message::ConfigChange(ConfigChange::EnablePerType(AnimationPrefix::Movement, v)),
                            )
                        }))
                        .push(matches!(&enable_config_type, ConfigType::PerType).then(|| {
                            opt_helpers::toggle(
                                "Enable Transparency Animations",
                                None,
                                matches!(&config.enabled, PerAnimationPrefixConfig::Prefix(p) if p.get(&AnimationPrefix::Transparency).unwrap_or(&false) == &true),
                                |v| Message::ConfigChange(ConfigChange::EnablePerType(AnimationPrefix::Transparency, v)),
                            )
                        }))
                        .spacing(10)
                        .into()
                    ]
                    },
                    DEFAULT_CONFIG
                        .animation
                        .as_ref()
                        .map(|a| a.enabled != config.enabled)
                        .unwrap_or_default(),
                    Message::ConfigChange(ConfigChange::EnableGlobal(false)),
                    None,
                ),
                opt_helpers::expandable(
                    "Duration",
                    Some(
                        "Set the animation duration in ms for all animations or per type of animation (default: 250)",
                    ),
                    move || {
                        [column![opt_helpers::opt_box(
                            row![
                                column![
                                    pick_list(
                                        [ConfigType::Global, ConfigType::PerType],
                                        Some(duration_config_type.clone()),
                                        Message::ToggleDurationConfigType,
                                    ),
                                    opt_helpers::description_text(
                                        if matches!(duration_config_type, ConfigType::Global) {
                                            "Set Duration for all types of animations"
                                        } else {
                                            "Set Duration per type of animation"
                                        }
                                    ),
                                ]
                                .spacing(5),
                                space::horizontal(),
                            ]
                            .push(config.duration.as_ref().map(|d| -> Element<Message> {
                                if let PerAnimationPrefixConfig::Global(duration) = d {
                                    number_input("", *duration)
                                        .on_input(|v| {
                                            Message::ConfigChange(ConfigChange::DurationGlobal(v))
                                        })
                                        .into()
                                } else {
                                    space::horizontal().into()
                                }
                            }))
                            .align_y(Center)
                        )]
                        .push(matches!(&duration_config_type, ConfigType::PerType).then(
                            || -> Element<Message> {
                                if let Some(PerAnimationPrefixConfig::Prefix(hm)) = &config.duration
                                {
                                    if let Some(duration) = hm.get(&AnimationPrefix::Movement) {
                                        opt_helpers::number_with_disable_default(
                                            "Set Duration for Movement Animations",
                                            None,
                                            *duration,
                                            250,
                                            |v| {
                                                Message::ConfigChange(
                                                    ConfigChange::DurationPerType(
                                                        AnimationPrefix::Movement,
                                                        v,
                                                    ),
                                                )
                                            },
                                            None,
                                        )
                                    } else {
                                        space::horizontal().into()
                                    }
                                } else {
                                    space::horizontal().into()
                                }
                            },
                        ))
                        .push(matches!(&duration_config_type, ConfigType::PerType).then(
                            || -> Element<Message> {
                                if let Some(PerAnimationPrefixConfig::Prefix(hm)) = &config.duration
                                {
                                    if let Some(duration) = hm.get(&AnimationPrefix::Transparency) {
                                        opt_helpers::number_with_disable_default(
                                            "Set Duration for Transparency Animations",
                                            None,
                                            *duration,
                                            250,
                                            |v| {
                                                Message::ConfigChange(
                                                    ConfigChange::DurationPerType(
                                                        AnimationPrefix::Transparency,
                                                        v,
                                                    ),
                                                )
                                            },
                                            None,
                                        )
                                    } else {
                                        space::horizontal().into()
                                    }
                                } else {
                                    space::horizontal().into()
                                }
                            },
                        ))
                        .spacing(10)
                        .into()]
                    },
                    DEFAULT_CONFIG
                        .animation
                        .as_ref()
                        .map(|a| a.duration != config.duration)
                        .unwrap_or_default(),
                    Message::ConfigChange(ConfigChange::DurationGlobal(250)),
                    None,
                ),
                opt_helpers::expandable(
                    "Style",
                    Some(
                        "Set the animation style for all animations or per type of animation (default: Linear)",
                    ),
                    move || {
                        [column![opt_helpers::opt_box(
                            row![
                                column![
                                    pick_list(
                                        [ConfigType::Global, ConfigType::PerType],
                                        Some(style_config_type.clone()),
                                        Message::ToggleStyleConfigType,
                                    ),
                                    opt_helpers::description_text(
                                        if matches!(style_config_type, ConfigType::Global) {
                                            "Set Style for all types of animations"
                                        } else {
                                            "Set Style per type of animation"
                                        }
                                    ),
                                ]
                                .spacing(5),
                                space::horizontal(),
                            ]
                            .push(config.style.as_ref().map(|s| -> Element<Message> {
                                if let PerAnimationPrefixConfig::Global(style) = s {
                                    pick_list(*ALL_ANIMATIONS_STYLES, Some(style), |s| {
                                        Message::ConfigChange(ConfigChange::StyleGlobal(s))
                                    })
                                    .into()
                                } else {
                                    space::horizontal().into()
                                }
                            }))
                            .align_y(Center)
                        )]
                        .push(matches!(&style_config_type, ConfigType::PerType).then(
                            || -> Element<Message> {
                                if let Some(PerAnimationPrefixConfig::Prefix(hm)) = &config.style {
                                    if let Some(style) = hm.get(&AnimationPrefix::Movement) {
                                        opt_helpers::choose(
                                            "Set Style for Movement Animations",
                                            None,
                                            *ALL_ANIMATIONS_STYLES,
                                            Some(style),
                                            |s| {
                                                Message::ConfigChange(ConfigChange::StylePerType(
                                                    AnimationPrefix::Movement,
                                                    s,
                                                ))
                                            },
                                        )
                                    } else {
                                        space::horizontal().into()
                                    }
                                } else {
                                    space::horizontal().into()
                                }
                            },
                        ))
                        .push(matches!(&style_config_type, ConfigType::PerType).then(
                            || -> Element<Message> {
                                if let Some(PerAnimationPrefixConfig::Prefix(hm)) = &config.style {
                                    if let Some(style) = hm.get(&AnimationPrefix::Transparency) {
                                        opt_helpers::choose(
                                            "Set Style for Transparency Animations",
                                            None,
                                            *ALL_ANIMATIONS_STYLES,
                                            Some(style),
                                            |s| {
                                                Message::ConfigChange(ConfigChange::StylePerType(
                                                    AnimationPrefix::Transparency,
                                                    s,
                                                ))
                                            },
                                        )
                                    } else {
                                        space::horizontal().into()
                                    }
                                } else {
                                    space::horizontal().into()
                                }
                            },
                        ))
                        .spacing(10)
                        .into()]
                    },
                    DEFAULT_CONFIG
                        .animation
                        .as_ref()
                        .map(|a| a.style != config.style)
                        .unwrap_or_default(),
                    Message::ConfigChange(ConfigChange::StyleGlobal(AnimationStyle::Linear)),
                    None,
                ),
                opt_helpers::number_with_disable_default_option(
                    "FPS",
                    Some("Set the animation FPS for all animations"),
                    config.fps,
                    DEFAULT_CONFIG.animation.as_ref().and_then(|a| a.fps),
                    |v| Message::ConfigChange(ConfigChange::Fps(v)),
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
