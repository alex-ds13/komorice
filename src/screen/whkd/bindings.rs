use crate::{
    whkd::{HotkeyBinding, Whkdrc},
    widget::{self, opt_helpers},
};

use std::collections::HashMap;

use iced::{
    widget::{column, container, markdown, pick_list, row, text},
    Element, Subscription, Task, Theme,
};

static MODIFIERS: [&str; 4] = ["CTRL", "SHIFT", "ALT", "WIN"];

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Bindings(Vec<HotkeyBinding>),
    ChangeNewBindingMod(usize, String),
    ChangeNewBindingKey(String),
    ChangeNewBindingCommand(String),
    AddNewBinding,
    RemoveBinding(usize),
    ChangeBindingKeys(usize, Vec<String>),
    ChangeBindingMod(usize, (usize, String)),
    ChangeBindingKey(usize, String),
    ChangeBindingCommand(usize, String),

    KeyPress(Option<String>, String),
    KeyRelease,
    UrlClicked(markdown::Url),
    Nothing,
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug)]
pub struct Bindings {
    pressed_key: String,
    pressed_mod: String,
    new_binding: HotkeyBinding,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            pressed_key: Default::default(),
            pressed_mod: Default::default(),
            new_binding: HotkeyBinding {
                keys: Vec::new(),
                command: String::new(),
                process_name: None,
            },
        }
    }
}

impl Bindings {
    pub fn update(&mut self, message: Message, whkdrc: &mut Whkdrc) -> (Action, Task<Message>) {
        match message {
            Message::KeyPress(Some(k), m) => {
                self.pressed_key = k;
                self.pressed_mod = m;
            }
            Message::KeyPress(None, _m) => {}
            Message::KeyRelease => {
                // self.pressed_key = String::new();
                // self.pressed_mod = String::new();
            }
            Message::ChangeBindingMod(idx, (pos, mod1)) => {
                if let Some(binding) = whkdrc.bindings.get_mut(idx) {
                    let sb = split_binding(&binding);
                    if mod1.is_empty() {
                        if pos < sb.modifiers.len() {
                            if pos < binding.keys.len() {
                                binding.keys.remove(pos);
                            }
                        }
                    } else {
                        if let Some(k) = binding.keys.iter_mut().filter(|m| is_mod(m)).nth(pos) {
                            *k = mod1.clone();
                        } else if pos <= binding.keys.len() {
                            binding.keys.insert(pos, mod1.clone());
                        } else {
                            //TODO: show error to user in case `i` is higher than len(), this shouldn't
                            //happen though
                            println!("Failed to add mod {mod1} to binding with index {pos} since len is {}", binding.keys.len());
                        }
                    }
                }
            }
            Message::ChangeBindingKey(idx, key) => {
                if let Some(binding) = whkdrc.bindings.get_mut(idx) {
                    let keys_count = split_binding(&binding).key.len();
                    if key.is_empty() {
                        if keys_count == 1 {
                            binding.keys.pop();
                        } else if keys_count >= 2 {
                            //TODO: show error to user
                            println!("Failed to remove key {key} from binding since key count is {}, should be <=1", keys_count);
                        }
                    } else {
                        if keys_count == 1 {
                            if let Some(k) = binding.keys.last_mut() {
                                *k = key;
                            } else {
                                binding.keys.push(key);
                            }
                        } else if keys_count == 0 {
                            binding.keys.push(key);
                        } else {
                            //TODO: show error to user
                            println!("Failed to add key {key} to binding since key count is {}, should be <=1", keys_count);
                        }
                    }
                }
            }
            Message::ChangeBindingCommand(idx, command) => {
                if let Some(binding) = whkdrc.bindings.get_mut(idx) {
                    binding.command = command;
                }
            }
            Message::Bindings(_) => todo!(),
            Message::AddNewBinding => todo!(),
            Message::RemoveBinding(_) => todo!(),
            Message::ChangeBindingKeys(_, _) => todo!(),
            Message::Nothing => {}
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
            Message::ChangeNewBindingMod(_, _) => todo!(),
            Message::ChangeNewBindingKey(_) => todo!(),
            Message::ChangeNewBindingCommand(_) => todo!(),
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
        let col = column![].spacing(10);
        let bindings = whkdrc
            .bindings
            .iter()
            .enumerate()
            .fold(col, |col, (idx, binding)| {
                // let keybind = opt_helpers::opt_custom_el_disable_default(
                //     "Binding",
                //     Some("todo!"),
                //     keys(idx, binding),
                //     false,
                //     None,
                //     None,
                // );
                // let command = command_edit(idx, &binding.command, commands, commands_desc, theme);

                // let mut key_pressed = row![text("PRESSED: "), text!("{}", self.pressed_mod),];

                // key_pressed = key_pressed.push_maybe(
                //     (!self.pressed_mod.is_empty() && !self.pressed_key.is_empty())
                //         .then_some(text(" + ")),
                // );
                // key_pressed = key_pressed.push(text!("{}", self.pressed_key));
                let keys_count = binding.keys.len();
                let keybind = row![].push(
                    container(binding.keys.iter().enumerate().fold(
                        row![].spacing(5),
                        |r, (i, m)| {
                            if i == keys_count - 1 {
                                r.push(text(m))
                            } else {
                                r.push(text(m)).push(" + ")
                            }
                        },
                    ))
                    .padding(2)
                    .style(container::dark),
                );
                let command =
                    container(text(&binding.command).wrapping(text::Wrapping::WordOrGlyph))
                        .max_width(300);

                let b = iced::widget::hover(
                    row![keybind, text(" -> "), command]
                        .height(30)
                        .align_y(iced::Center)
                        .padding(iced::padding::right(60)),
                    iced::widget::right(
                        iced::widget::button(crate::icons::edit())
                            .on_press(Message::Nothing)
                            .style(iced::widget::button::secondary),
                    )
                    .padding(iced::padding::right(5))
                    .align_y(iced::Center),
                );

                col.push(b)
            })
            .into();

        opt_helpers::section_view("Bindings:", [bindings])
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

        Subscription::batch([press, release])
    }
}

fn is_mod(key: &str) -> bool {
    MODIFIERS.contains(&key.to_uppercase().as_str())
}

fn mod_choose<'a>(idx: usize, binding_mods: &[String], pos: usize) -> Option<Element<'a, Message>> {
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
        pick_list(options, Some(k), move |v| {
            Message::ChangeBindingMod(idx, (pos, v))
        })
        .into()
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

fn keys(idx: usize, binding: &HotkeyBinding) -> Element<'_, Message> {
    let sb = split_binding(binding);
    let key = widget::input(
        "",
        sb.key.iter().next_back().map_or("", |s| s.as_str()),
        move |v| Message::ChangeBindingKey(idx, v),
        None,
    )
    .width(75);
    column![
        row![]
            .push_maybe(mod_choose(idx, sb.modifiers, 3))
            .push_maybe(mod_choose(idx, sb.modifiers, 2))
            .push_maybe(mod_choose(idx, sb.modifiers, 1))
            .push_maybe(mod_choose(idx, sb.modifiers, 0))
            .push(key)
            .spacing(5),
        binding
            .keys
            .iter()
            .fold(row!["PB:"].spacing(5), |r, m| { r.push(text(m)) }),
    ]
    .into()
}

fn command_edit<'a>(
    idx: usize,
    command: &'a String,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let (main_cmd, rest) = split_command(command, commands);
    let c = Some(command.clone());
    opt_helpers::expandable_custom(
        "Pause Hook",
        Some("A command that should run on pause keybind trigger.  (default: None)"),
        move |_, _| {
            let pick = pick_list(commands, Some(main_cmd.to_string()), move |v| {
                let cmd = if rest.is_empty() {
                    format!("komorebic {v}")
                } else {
                    format!("komorebic {v} {rest}")
                };
                Message::ChangeBindingCommand(idx, cmd)
            });
            let c = c.as_ref().map_or("", String::as_str);
            let custom =
                widget::input("", c, |v| Message::ChangeBindingCommand(idx, v), None).width(550);
            column![
                row!["Komorebic commands:", pick].spacing(5),
                "Command:",
                // custom,
                text(command.clone()),
            ]
            .width(iced::Shrink)
            .spacing(10)
        },
        move || {
            if let Some(items) =
                commands_desc.get(main_cmd.strip_prefix("komorebic ").unwrap_or_default())
            {
                vec![markdown(items, theme).map(Message::UrlClicked)]
            } else {
                vec![]
            }
        },
        false,
        true,
        Message::ChangeBindingCommand(idx, String::new()),
        None,
    )
}

fn split_command<'a>(command: &'a String, commands: &'a [String]) -> (&'a str, &'a str) {
    let mut final_command = "";
    let final_custom;
    if let Some((cmd, custom)) = commands.iter().find_map(|c| {
        let potential_cmd = format!("komorebic {}", c);
        command
            .starts_with(&potential_cmd)
            .then(|| command.split_at(potential_cmd.len()))
    }) {
        final_command = cmd;
        final_custom = custom.trim_start();
    } else {
        final_custom = command;
    }
    (final_command, final_custom)
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

fn split_binding(binding: &HotkeyBinding) -> SplitBinding<'_> {
    if let Some(split_at) = binding
        .keys
        .iter()
        .enumerate()
        .find_map(|(i, k)| (!is_mod(k)).then_some(i))
    {
        binding.keys.split_at(split_at).into()
    } else {
        binding.keys.split_at(binding.keys.len()).into()
    }
}
