use crate::{
    whkd::{HotkeyBinding, Shell, Whkdrc, DEFAULT_WHKDRC},
    widget::{self, opt_helpers},
};

use std::{collections::HashMap, sync::LazyLock};

use async_compat::Compat;
use iced::{
    widget::{column, markdown, pick_list, row, text},
    Element, Subscription, Task, Theme,
};

static MODIFIERS: [&str; 4] = ["CTRL", "SHIFT", "ALT", "WIN"];

static COMMANDS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "focus-window",
        "move-window",
        "cycle-focus-window",
        "cycle-move-window",
        "stack-window",
        "unstack-window",
        "cycle-stack",
        "cycle-stack-index",
        "focus-stack-window",
        "stack-all",
        "unstack-all",
        "resize-window-edge",
        "resize-window-axis",
        "move-container-to-last-workspace",
        "send-container-to-last-workspace",
        "move-container-to-monitor-number",
        "cycle-move-container-to-monitor",
        "move-container-to-workspace-number",
        "move-container-to-named-workspace",
        "cycle-move-container-to-workspace",
        "send-container-to-monitor-number",
        "cycle-send-container-to-monitor",
        "send-container-to-workspace-number",
        "cycle-send-container-to-workspace",
        "send-container-to-monitor-workspace-number",
        "move-container-to-monitor-workspace-number",
        "send-container-to-named-workspace",
        "cycle-move-workspace-to-monitor",
        "move-workspace-to-monitor-number",
        "swap-workspaces-to-monitor-number",
        "force-focus",
        "close",
        "minimize",
        "promote",
        "promote-focus",
        "promote-window",
        "eager-focus",
        "lock-monitor-workspace-container",
        "unlock-monitor-workspace-container",
        "toggle-lock",
        "toggle-float",
        "toggle-monocle",
        "toggle-maximize",
        "toggle-window-container-behaviour",
        "toggle-float-override",
        "window-hiding-behaviour",
        "toggle-cross-monitor-move-behaviour",
        "cross-monitor-move-behaviour",
        "unmanaged-window-operation-behaviour",
        "manage-focused-window",
        "unmanage-focused-window",
        "adjust-container-padding",
        "adjust-workspace-padding",
        "change-layout",
        "cycle-layout",
        "change-layout-custom",
        "flip-layout",
        "toggle-workspace-window-container-behaviour",
        "toggle-workspace-float-override",
        "monitor-index-preference",
        "display-index-preference",
        "ensure-workspaces",
        "ensure-named-workspaces",
        "new-workspace",
        "toggle-tiling",
        "stop",
        "stop-ignore-restore",
        "toggle-pause",
        "retile",
        "retile-with-resize-dimensions",
        "quick-save",
        "quick-load",
        "save",
        "load",
        "cycle-focus-monitor",
        "cycle-focus-workspace",
        "cycle-focus-empty-workspace",
        "focus-monitor-number",
        "focus-monitor-at-cursor",
        "focus-last-workspace",
        "close-workspace",
        "focus-workspace-number",
        "focus-workspace-numbers",
        "focus-monitor-workspace-number",
        "focus-named-workspace",
        "container-padding",
        "named-workspace-container-padding",
        "focused-workspace-container-padding",
        "workspace-padding",
        "named-workspace-padding",
        "focused-workspace-padding",
        "workspace-tiling",
        "named-workspace-tiling",
        "workspace-name",
        "workspace-layout",
        "named-workspace-layout",
        "workspace-layout-custom",
        "named-workspace-layout-custom",
        "workspace-layout-rule",
        "named-workspace-layout-rule",
        "workspace-layout-custom-rule",
        "named-workspace-layout-custom-rule",
        "clear-workspace-layout-rules",
        "clear-named-workspace-layout-rules",
        "toggle-workspace-layer",
        "reload-configuration",
        "replace-configuration",
        "reload-static-configuration",
        "watch-configuration",
        "complete-configuration",
        "alt-focus-hack",
        "theme",
        "animation",
        "animation-duration",
        "animation-fps",
        "animation-style",
        "border",
        "border-colour",
        "border-style",
        "border-width",
        "border-offset",
        "border-implementation",
        "transparency",
        "toggle-transparency",
        "transparency-alpha",
        "invisible-borders",
        "stackbar-mode",
        "stackbar-label",
        "stackbar-focused-text-colour",
        "stackbar-unfocused-text-colour",
        "stackbar-background-colour",
        "stackbar-height",
        "stackbar-tab-width",
        "stackbar-font-size",
        "stackbar-font-family",
        "work-area-offset",
        "monitor-work-area-offset",
        "toggle-window-based-work-area-offset",
        "resize-delta",
        "initial-workspace-rule",
        "initial-named-workspace-rule",
        "workspace-rule",
        "named-workspace-rule",
        "clear-workspace-rules",
        "clear-named-workspace-rules",
        "clear-all-workspace-rules",
        "enforce-workspace-rules",
        "ignore-rule",
        "manage-rule",
        "identify-object-name-change-application",
        "identify-tray-application",
        "identify-layered-application",
        "identify-border-overflow-application",
        "state",
        "global-state",
        "visible-windows",
        "monitor-information",
        "query",
        "focus-follows-mouse",
        "toggle-focus-follows-mouse",
        "mouse-follows-focus",
        "toggle-mouse-follows-focus",
        "remove-title-bar",
        "toggle-title-bars",
        "add-subscriber-socket",
        "add-subscriber-socket-with-options",
        "remove-subscriber-socket",
        "add-subscriber-pipe",
        "remove-subscriber-pipe",
        "application-specific-configuration-schema",
        "notification-schema",
        "socket-schema",
        "static-config-schema",
        "generate-static-config",
        "debug-window",
    ]
});

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Shell(Shell),
    PBMod(usize, String),
    PBKey(String),
    PauseBinding(Option<Vec<String>>),
    PauseHook(Option<String>),
    AppBindings(Vec<(Vec<String>, Vec<HotkeyBinding>)>),
    AddNewAppBinding,
    RemoveAppBinding(usize),
    ChangeAppBindingKeys(usize, Vec<String>),
    ChangeAppBindingProcessName(usize, String),
    ChangeAppBindingCommand(usize, String),
    Bindings(Vec<HotkeyBinding>),
    AddNewBinding,
    RemoveBinding(usize),
    ChangeBindingKeys(usize, Vec<String>),
    ChangeBindingCommand(usize, String),

    LoadedCommands(Vec<String>),
    FailedToLoadCommands(String),
    LoadedCommandDescription(String, String),
    FailedToLoadCommandsDescription(String),
    KeyPress(Option<String>, String),
    KeyRelease,
    Navigate(NavMessage),
    NavigateForward,
    NavigateBack,
    UrlClicked(markdown::Url),
    Nothing,
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct Whkd {
    pub loaded_commands: bool,
    commands: Vec<String>,
    pub loaded_commands_desc: bool,
    commands_desc: HashMap<String, Vec<markdown::Item>>,
    pressed_key: String,
    pressed_mod: String,
    pb_mods: Vec<String>,
    pause_hook_command: String,
    pause_hook_custom: String,
}

impl Whkd {
    pub fn update(&mut self, message: Message, whkdrc: &mut Whkdrc) -> (Action, Task<Message>) {
        match message {
            Message::Shell(shell) => whkdrc.shell = shell,
            Message::KeyPress(Some(k), m) => {
                self.pressed_key = k;
                self.pressed_mod = m;
            }
            Message::KeyPress(None, _m) => {}
            Message::KeyRelease => {
                // self.pressed_key = String::new();
                // self.pressed_mod = String::new();
            }
            Message::PBMod(i, mod1) => {
                if mod1.is_empty() {
                    if let Some(pause_binding) = &mut whkdrc.pause_binding {
                        if i < pause_binding.len() {
                            pause_binding.remove(i);
                        }
                    }
                    if let Some(i) = self.pb_mods.iter().position(|m| m == &mod1) {
                        self.pb_mods.remove(i);
                    }
                } else if let Some(pause_binding) = &mut whkdrc.pause_binding {
                    if let Some(k) = pause_binding
                        .iter_mut()
                        .filter(|m| MODIFIERS.contains(&m.as_str()))
                        .nth(i)
                    {
                        *k = mod1.clone();
                        self.pb_mods.push(mod1);
                    } else if i <= pause_binding.len() {
                        pause_binding.insert(i, mod1.clone());
                        self.pb_mods.push(mod1);
                    } else {
                        //TODO: show error to user in case `i` is higher than len(), this shouldn't
                        //happen though
                        println!("Failed to add mod {mod1} to pause_binding with index {i} since len is {}", pause_binding.len());
                    }
                } else {
                    whkdrc.pause_binding = Some(vec![mod1.clone()]);
                    self.pb_mods.push(mod1);
                }
            }
            Message::PBKey(key) => {
                if key.is_empty() {
                    if let Some(pause_binding) = &mut whkdrc.pause_binding {
                        let keys = pause_binding
                            .iter_mut()
                            .filter(|k| !self.pb_mods.contains(k));
                        let count = keys.count();
                        if count == 1 {
                            pause_binding.pop();
                        } else if count >= 2 {
                            //TODO: show error to user
                            println!("Failed to remove key {key} from pause_binding since key count is {count}, should be <=1");
                        }
                    }
                } else if let Some(pause_binding) = &mut whkdrc.pause_binding {
                    let count = pause_binding
                        .iter()
                        .filter(|k| !self.pb_mods.contains(k))
                        .count();
                    let mut keys = pause_binding
                        .iter_mut()
                        .filter(|k| !self.pb_mods.contains(k));
                    if count <= 1 {
                        if let Some(k) = keys.next_back() {
                            *k = key;
                        } else {
                            pause_binding.push(key);
                        }
                    } else {
                        //TODO: show error to user
                        println!("Failed to add key {key} to pause_binding since key count is {count}, should be <=1");
                    }
                } else {
                    whkdrc.pause_binding = Some(vec![key]);
                }
            }
            Message::PauseBinding(_) => todo!(),
            Message::PauseHook(pause_hook) => {
                // if let Some(hook) = &pause_hook {
                //     if let Some(command) = COMMANDS.iter().find(|c| hook.starts_with(*c)) {
                //         self.pause_hook_command = command.clone();
                //         self.pause_hook_custom =
                //             hook.split_at(command.len()).1.trim_start().to_string();
                //     } else {
                //         self.pause_hook_command = String::new();
                //         self.pause_hook_custom = hook.clone();
                //     }
                // } else {
                //     self.pause_hook_command = String::new();
                //     self.pause_hook_custom = String::new();
                // }
                whkdrc.pause_hook = pause_hook;
            }
            Message::AppBindings(_) => todo!(),
            Message::AddNewAppBinding => todo!(),
            Message::RemoveAppBinding(_) => todo!(),
            Message::ChangeAppBindingKeys(_, _) => todo!(),
            Message::ChangeAppBindingProcessName(_, _) => todo!(),
            Message::ChangeAppBindingCommand(_, _) => todo!(),
            Message::Bindings(_) => todo!(),
            Message::AddNewBinding => todo!(),
            Message::RemoveBinding(_) => todo!(),
            Message::ChangeBindingKeys(_, _) => todo!(),
            Message::ChangeBindingCommand(_, _) => todo!(),
            Message::Navigate(nav) => match nav {
                NavMessage::Forward => {}
                NavMessage::Back => {}
            },
            Message::NavigateForward => {}
            Message::NavigateBack => {}
            Message::Nothing => {}
            Message::LoadedCommands(commands) => {
                // println!("{commands:?}");
                self.commands = commands;
                self.loaded_commands = true;
                return (Action::None, self.load_commands_description());
            }
            Message::FailedToLoadCommands(error) => {
                println!("WHKD -> Failed to load commands: {error}");
            }
            Message::LoadedCommandDescription(command, description) => {
                println!("received description for command: {command}");
                let md = markdown::parse(&description).collect();
                self.commands_desc.insert(command, md);
            }
            Message::FailedToLoadCommandsDescription(error) => {
                println!("WHKD -> Failed to load commands: {error}");
            }
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, whkdrc: &'a Whkdrc, theme: &'a Theme) -> Element<'a, Message> {
        let shell = opt_helpers::choose_with_disable_default(
            "Shell",
            Some("The Shell you want whkd to use. (default: pwsh)"),
            Vec::new(),
            [Shell::Pwsh, Shell::Powershell, Shell::Cmd],
            Some(whkdrc.shell),
            |v| Message::Shell(v.unwrap_or(DEFAULT_WHKDRC.shell)),
            Some(DEFAULT_WHKDRC.shell),
            None,
        );
        let pause_binding = opt_helpers::opt_custom_el_disable_default(
            "Pause Binding",
            Some("todo!"),
            keys(whkdrc, self.pb_mods.as_slice()),
            false,
            None,
            None,
        );
        let pause_hook = hook_custom(
            &whkdrc.pause_hook,
            &self.commands,
            &self.commands_desc,
            theme,
        );

        let mut key_pressed = row![text("PRESSED: "), text!("{}", self.pressed_mod),];

        key_pressed = key_pressed.push_maybe(
            (!self.pressed_mod.is_empty() && !self.pressed_key.is_empty()).then_some(text(" + ")),
        );
        key_pressed = key_pressed.push(text!("{}", self.pressed_key));

        opt_helpers::section_view(
            "Whkd:",
            [shell, pause_binding, pause_hook, key_pressed.into()],
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard::{
            self,
            key::{Key, Named},
        };

        let press = keyboard::on_key_press(|key, modifiers| {
            let k = match key {
                Key::Named(named) => match named {
                    Named::Alt
                    | Named::AltGraph
                    | Named::Shift
                    | Named::Control
                    | Named::Meta
                    | Named::Hyper
                    | Named::Super => None,
                    _ => Some(format!("{:?}", named)),
                },
                Key::Character(c) => Some(format!("{}", c)),
                Key::Unidentified => None,
            };
            let m = modifiers
                .iter_names()
                .fold(String::new(), |mut s, (n, _m)| {
                    if !s.is_empty() {
                        s.push_str(" + ");
                    }
                    s.push_str(n);
                    s
                });
            Some(Message::KeyPress(k, m))
        });
        let release = keyboard::on_key_release(|_, _| Some(Message::KeyRelease));
        let navigation = navigation_sub().map(Message::Navigate);

        Subscription::batch([press, release, navigation])
    }

    pub fn load_commands(&self) -> Task<Message> {
        Task::future(Compat::new(async {
            static APP_USER_AGENT: &str =
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

            println!("Running GET request: {}", APP_USER_AGENT);

            let client = reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()?;
            client
                .get("https://api.github.com/repos/lgug2z/komorebi/contents/docs/cli")
                .send()
                .await
        }))
        .then(|res| match res {
            Ok(response) => Task::perform(
                Compat::new(async {
                    #[derive(serde::Deserialize)]
                    struct Command {
                        name: String,
                    }
                    response.json::<Vec<Command>>().await
                }),
                |res| match res {
                    Ok(commands) => Message::LoadedCommands(
                        commands
                            .into_iter()
                            .flat_map(|c| c.name.strip_suffix(".md").map(|v| v.to_string()))
                            .collect(),
                    ),
                    Err(error) => Message::FailedToLoadCommands(error.to_string()),
                },
            ),
            Err(error) => Task::done(Message::FailedToLoadCommands(error.to_string())),
        })
    }

    pub fn load_commands_description(&self) -> Task<Message> {
        Task::batch(self.commands.iter().map(|command| {
            let command_c = command.clone();
            let command_c1 = command.clone();
            Task::future(Compat::new(async move {
                static APP_USER_AGENT: &str =
                    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

                println!(
                    "Running GET request for command {} description: {}",
                    &command_c1, APP_USER_AGENT
                );

                let client = reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                client
                    .get(format!(
                        "https://raw.githubusercontent.com/lgug2z/komorebi/master/docs/cli/{}.md",
                        &command_c1,
                    ))
                    .send()
                    .await
            }))
            .then(move |res| {
                let command_c = command_c.clone();
                match res {
                    Ok(response) => {
                        Task::perform(Compat::new(async { response.text().await }), move |res| {
                            match res {
                                Ok(description) => {
                                    Message::LoadedCommandDescription(command_c, description)
                                }
                                Err(error) => {
                                    Message::FailedToLoadCommandsDescription(error.to_string())
                                }
                            }
                        })
                    }
                    Err(error) => {
                        Task::done(Message::FailedToLoadCommandsDescription(error.to_string()))
                    }
                }
            })
        }))
    }
}

fn mod_choose(whkdrc: &Whkdrc, pos: usize) -> Option<Element<Message>> {
    let pl = |k: String| -> Element<Message> {
        let mut options = vec![
            "".into(),
            "CTRL".into(),
            "SHIFT".into(),
            "ALT".into(),
            "WIN".into(),
        ];
        options.retain(|v| {
            !whkdrc
                .pause_binding
                .as_ref()
                .is_some_and(|ks| ks.contains(v))
        });
        pick_list(options, Some(k), move |v| Message::PBMod(pos, v)).into()
    };
    let is_mod = |k| MODIFIERS.contains(&k);
    if whkdrc.pause_binding.is_none() && pos == 0 {
        Some(pl(String::new()))
    } else if whkdrc.pause_binding.is_none() {
        None
    } else {
        whkdrc.pause_binding.as_ref().and_then(|pb| {
            if pb.is_empty() && pos == 0 {
                Some(pl(String::new()))
            } else {
                let filtered = pb.iter().filter(|k| is_mod(k.as_str())).collect::<Vec<_>>();
                if let Some(k) = filtered.get(pos) {
                    Some(pl((*k).clone()))
                } else {
                    (pos == filtered.len()).then_some(pl(String::new()))
                }
            }
        })
    }
}

fn keys<'a>(whkdrc: &'a Whkdrc, pb_mods: &'a [String]) -> Element<'a, Message> {
    let key = widget::input(
        "",
        whkdrc.pause_binding.as_ref().map_or("", |pb| {
            pb.iter()
                .filter(|k| !pb_mods.contains(k))
                .next_back()
                .map_or("", |s| s.as_str())
        }),
        Message::PBKey,
        None,
    )
    .width(75);
    row![]
        .push_maybe(mod_choose(whkdrc, 3))
        .push_maybe(mod_choose(whkdrc, 2))
        .push_maybe(mod_choose(whkdrc, 1))
        .push_maybe(mod_choose(whkdrc, 0))
        .push(key)
        .spacing(5)
        .into()
}

fn hook_custom<'a>(
    pause_hook: &'a Option<String>,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let (hook_command, hook_custom) = split_pause_hook(pause_hook, commands);
    let is_dirty = pause_hook != &DEFAULT_WHKDRC.pause_hook;
    opt_helpers::expandable_custom(
        "Pause Hook",
        Some("A command that should run on pause keybind trigger"),
        move |_, _| {
            let pick = pick_list(commands, Some(hook_command.to_string()), move |v| {
                let hook = Some(if hook_custom.is_empty() {
                    format!("komorebic {v}")
                } else {
                    format!("komorebic {v} {hook_custom}")
                });
                Message::PauseHook(hook)
            });
            let ph = pause_hook.as_ref().map_or("", String::as_str);
            let custom = widget::input(
                "",
                ph,
                // move |v| {
                //     let hook = Some(if hook_command.is_empty() {
                //         v
                //     } else {
                //         format!("{} {}", hook_command, v)
                //     });
                //     Message::PauseHook(hook)
                // },
                |v| Message::PauseHook(Some(v)),
                None,
            )
            .width(550);
            column![
                row!["Komorebic commands:", pick].spacing(5),
                "Command:",
                custom,
                text(ph)
            ]
            .width(iced::Shrink)
            .spacing(10)
        },
        move || {
            [if let Some(items) =
                commands_desc.get(hook_command.strip_prefix("komorebic ").unwrap_or_default())
            {
                markdown(items, theme)
            } else {
                iced::widget::horizontal_space().into()
            }
            .map(Message::UrlClicked)]
        },
        is_dirty,
        commands_desc
            .get(hook_command.strip_prefix("komorebic ").unwrap_or_default())
            .is_some(),
        Message::PauseHook(DEFAULT_WHKDRC.pause_hook.clone()),
        None,
    )
}

fn split_pause_hook<'a>(
    pause_hook: &'a Option<String>,
    commands: &'a [String],
) -> (&'a str, &'a str) {
    let mut pause_hook_command = "";
    let mut pause_hook_custom = "";
    if let Some(hook) = pause_hook {
        if let Some((command, custom)) = commands.iter().find_map(|c| {
            let potential_cmd = format!("komorebic {}", c);
            hook.starts_with(&potential_cmd)
                .then(|| hook.split_at(potential_cmd.len()))
        }) {
            pause_hook_command = command;
            pause_hook_custom = custom.trim_start();
        } else {
            pause_hook_custom = hook;
        }
    }
    (pause_hook_command, pause_hook_custom)
}

#[derive(Debug, Clone, PartialEq)]
pub enum NavMessage {
    Forward,
    Back,
}

pub fn navigation_sub() -> Subscription<NavMessage> {
    use iced::{event, mouse, Event};

    event::listen_with(|e, s, _| {
        if matches!(s, event::Status::Ignored) {
            match e {
                Event::Mouse(event) => match event {
                    mouse::Event::ButtonPressed(mouse::Button::Forward) => {
                        Some(NavMessage::Forward)
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Back) => Some(NavMessage::Back),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    })
}
