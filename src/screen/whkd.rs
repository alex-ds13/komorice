pub mod bindings;

pub use bindings::Bindings;

use crate::{
    whkd::{DEFAULT_WHKDRC, HotkeyBinding, Shell, Whkdrc},
    widget::{self, opt_helpers},
};

use std::collections::HashMap;

use iced::{
    Element, Subscription, Task, Theme,
    widget::{column, markdown, pick_list, row, text},
};

static MODIFIERS: [&str; 4] = ["CTRL", "SHIFT", "ALT", "WIN"];

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
    pressed_key: String,
    pressed_mod: String,
    pub bindings: Bindings,
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
                let sb = split_binding(&whkdrc.pause_binding);
                if mod1.is_empty() {
                    if i < sb.modifiers.len()
                        && let Some(pause_binding) = &mut whkdrc.pause_binding
                        && i < pause_binding.len()
                    {
                        pause_binding.remove(i);
                    }
                } else if let Some(pause_binding) = &mut whkdrc.pause_binding {
                    if let Some(k) = pause_binding.iter_mut().filter(|m| is_mod(m)).nth(i) {
                        *k = mod1.to_lowercase();
                    } else if i <= pause_binding.len() {
                        pause_binding.insert(i, mod1.to_lowercase());
                    } else {
                        //TODO: show error to user in case `i` is higher than len(), this shouldn't
                        //happen though
                        println!(
                            "Failed to add mod {mod1} to pause_binding with index {i} since len is {}",
                            pause_binding.len()
                        );
                    }
                } else {
                    whkdrc.pause_binding = Some(vec![mod1.to_lowercase()]);
                }
            }
            Message::PBKey(key) => {
                let keys_count = split_binding(&whkdrc.pause_binding).key.len();
                if key.is_empty() {
                    if keys_count == 1 {
                        whkdrc.pause_binding.as_mut().and_then(|pb| pb.pop());
                    } else if keys_count >= 2 {
                        //TODO: show error to user
                        println!(
                            "Failed to remove key {key} from pause_binding since key count is {}, should be <=1",
                            keys_count
                        );
                    }
                } else if let Some(pause_binding) = whkdrc.pause_binding.as_mut() {
                    if keys_count == 1 {
                        if let Some(k) = pause_binding.last_mut() {
                            *k = key;
                        } else {
                            pause_binding.push(key);
                        }
                    } else if keys_count == 0 {
                        pause_binding.push(key);
                    } else {
                        //TODO: show error to user
                        println!(
                            "Failed to add key {key} to pause_binding since key count is {}, should be <=1",
                            keys_count
                        );
                    }
                } else {
                    whkdrc.pause_binding = Some(vec![key]);
                }
            }
            Message::PauseBinding(binding) => whkdrc.pause_binding = binding,
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
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        whkdrc: &'a Whkdrc,
        commands: &'a [String],
        commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
        theme: &'a Theme,
    ) -> Element<'a, Message> {
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
            Some("Can be any hotkey combo to toggle all other hotkeys on/off. (default: None)"),
            keys(&whkdrc.pause_binding),
            whkdrc.pause_binding.is_some(),
            Some(Message::PauseBinding(None)),
            None,
        );
        let pause_hook = hook_custom(&whkdrc.pause_hook, commands, commands_desc, theme);

        let mut key_pressed = row![text("PRESSED: "), text!("{}", self.pressed_mod),];

        key_pressed = key_pressed.push(
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
}

fn is_mod(key: &str) -> bool {
    MODIFIERS.contains(&key.to_uppercase().as_str())
}

fn mod_choose<'a>(binding_mods: &[String], pos: usize) -> Option<Element<'a, Message>> {
    let pl = |k: String| -> Element<Message> {
        let mut options = vec![
            "".into(),
            "CTRL".into(),
            "SHIFT".into(),
            "ALT".into(),
            "WIN".into(),
        ];
        options.retain(|v| {
            !binding_mods
                .iter()
                .map(|m| m.to_uppercase())
                .any(|m| &m == v)
        });
        pick_list(options, Some(k), move |v| Message::PBMod(pos, v)).into()
    };
    if let Some(k) = binding_mods.get(pos) {
        Some(pl((*k).clone()))
    } else {
        // Only show an empty picklist for the first position in case there are no modifiers being
        // used or show an empty picklist for the next position (modifiers.len()), otherwise do not
        // show any picklist at all.
        ((pos == 0 && binding_mods.is_empty()) || pos == binding_mods.len())
            .then_some(pl(String::new()))
    }
}

fn keys(binding: &Option<Vec<String>>) -> Element<'_, Message> {
    let sb = split_binding(binding);
    let key = widget::input(
        "",
        sb.key.iter().next_back().map_or("", |s| s.as_str()),
        Message::PBKey,
        None,
    )
    .width(75);
    column![
        row![]
            .push(mod_choose(sb.modifiers, 3))
            .push(mod_choose(sb.modifiers, 2))
            .push(mod_choose(sb.modifiers, 1))
            .push(mod_choose(sb.modifiers, 0))
            .push(key)
            .spacing(5),
        binding.as_ref().map_or(row!["PB: [None]"], |pb| pb
            .iter()
            .fold(row!["PB:"].spacing(5), |r, m| { r.push(text(m)) })),
    ]
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
        Some("A command that should run on pause keybind trigger.  (default: None)"),
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
            if let Some(items) =
                commands_desc.get(hook_command.strip_prefix("komorebic ").unwrap_or_default())
            {
                vec![markdown(items, theme).map(Message::UrlClicked)]
            } else {
                vec![]
            }
        },
        is_dirty,
        true,
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

struct SplitBinding<'a> {
    modifiers: &'a [String],
    key: &'a [String],
}
impl<'a> From<(&'a [String], &'a [String])> for SplitBinding<'a> {
    fn from(value: (&'a [String], &'a [String])) -> Self {
        Self {
            modifiers: value.0,
            key: value.1,
        }
    }
}
impl<'a> From<(&'a [String; 0], &'a [String; 0])> for SplitBinding<'a> {
    fn from(value: (&'a [String; 0], &'a [String; 0])) -> Self {
        Self {
            modifiers: value.0,
            key: value.1,
        }
    }
}

fn split_binding(binding: &Option<Vec<String>>) -> SplitBinding<'_> {
    if let Some(split_at) = binding.as_ref().and_then(|pb| {
        pb.iter()
            .enumerate()
            .find_map(|(i, k)| (!is_mod(k)).then_some(i))
    }) {
        binding.as_ref().unwrap().split_at(split_at).into()
    } else {
        binding
            .as_ref()
            .map_or((&[], &[]).into(), |pb| pb.split_at(pb.len()).into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NavMessage {
    Forward,
    Back,
}

pub fn navigation_sub() -> Subscription<NavMessage> {
    use iced::{Event, event, mouse};

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
