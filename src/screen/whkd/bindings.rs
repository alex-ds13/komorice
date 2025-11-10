use crate::{
    BOLD_FONT,
    whkd::{HotkeyBinding, Whkdrc},
    widget::{self, button_with_icon, hover, icons, opt_helpers},
};

use std::collections::{HashMap, HashSet};

use iced::{
    Element, Subscription, Task, Theme, padding,
    widget::{
        bottom_center, button, column, combo_box, container, markdown, pick_list, right, row, rule,
        scrollable, space, text,
    },
};

static MODIFIERS: [&str; 4] = ["CTRL", "SHIFT", "ALT", "WIN"];

const SEPARATOR: &str = " + ";

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Bindings(Vec<HotkeyBinding>),
    ChangeNewBindingMod(usize, String),
    ChangeNewBindingKey(String),
    ChangeNewBindingCommand(String),
    ToggleShowNewBinding,
    AddNewBinding,
    RemoveBinding(usize),
    ChangeBindingKeys(usize, Vec<String>),
    ChangeBindingMod(usize, (usize, String)),
    ChangeBindingKey(usize, String),
    ChangeBindingCommand(usize, String),
    EditBinding(usize),
    FinishEditBinding(usize),

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
    new_binding_state: combo_box::State<String>,
    show_new_binding: bool,
    editing: HashSet<usize>,
    editing_states: HashMap<usize, combo_box::State<String>>,
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
            new_binding_state: Default::default(),
            show_new_binding: false,
            editing: Default::default(),
            editing_states: Default::default(),
        }
    }
}

impl Bindings {
    pub fn update(
        &mut self,
        message: Message,
        whkdrc: &mut Whkdrc,
        commands: &[String],
    ) -> (Action, Task<Message>) {
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
                    let sb = split_binding(binding);
                    if mod1.is_empty() {
                        if pos < sb.modifiers.len() {
                            binding.keys.remove(pos);
                        }
                    } else if let Some(k) = binding.keys.iter_mut().filter(|m| is_mod(m)).nth(pos) {
                        *k = mod1.to_lowercase();
                    } else if pos <= binding.keys.len() {
                        binding.keys.insert(pos, mod1.to_lowercase());
                    } else {
                        //TODO: show error to user in case `i` is higher than len(), this shouldn't
                        //happen though
                        println!(
                            "Failed to add mod {mod1} to binding with index {pos} since len is {}",
                            binding.keys.len()
                        );
                    }
                }
            }
            Message::ChangeBindingKey(idx, key) => {
                if let Some(binding) = whkdrc.bindings.get_mut(idx) {
                    let sb = split_binding(binding);
                    let keys_count = sb.keys.len();
                    let mods_count = sb.modifiers.len();
                    if key.is_empty() {
                        if keys_count == 1 {
                            binding.keys.pop();
                        } else if keys_count >= 2 {
                            //TODO: show error to user
                            println!(
                                "Failed to remove key {key} from binding since key count is {}, should be <=1",
                                keys_count
                            );
                        }
                    } else {
                        println!("Adding/Updating a key");
                        if keys_count == 1 {
                            let kk = key.split(SEPARATOR).map(String::from).collect::<Vec<_>>();
                            if kk.len() > 1 {
                                binding.keys.truncate(mods_count);
                                kk.into_iter().for_each(|k| {
                                    println!("Adding key {k}...");
                                    binding.keys.push(k);
                                });
                            } else if let Some(k) = binding.keys.last_mut() {
                                *k = key;
                            } else {
                                binding.keys.push(key);
                            }
                        } else if keys_count == 0 {
                            binding.keys.push(key);
                        } else {
                            //TODO: show error to user
                            // println!("Failed to add key {key} to binding since key count is {}, should be <=1", keys_count);
                            binding.keys.truncate(binding.keys.len() - keys_count);
                            key.split(SEPARATOR).map(String::from).for_each(|k| {
                                println!("Adding key {k}...");
                                binding.keys.push(k);
                            });
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
            Message::AddNewBinding => {
                let default_binding = HotkeyBinding {
                    keys: Vec::new(),
                    command: String::new(),
                    process_name: None,
                };
                let new_binding = std::mem::replace(&mut self.new_binding, default_binding);
                whkdrc.bindings.push(new_binding);
            }
            Message::RemoveBinding(idx) => {
                if whkdrc.bindings.len() > idx {
                    whkdrc.bindings.remove(idx);
                }
            }
            Message::ChangeBindingKeys(_, _) => todo!(),
            Message::Nothing => {}
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
            Message::ChangeNewBindingMod(pos, mod1) => {
                let sb = split_binding(&self.new_binding);
                if mod1.is_empty() {
                    if pos < sb.modifiers.len() {
                        self.new_binding.keys.remove(pos);
                    }
                } else if let Some(k) = self
                    .new_binding
                    .keys
                    .iter_mut()
                    .filter(|m| is_mod(m))
                    .nth(pos)
                {
                    *k = mod1.to_lowercase();
                } else if pos <= self.new_binding.keys.len() {
                    self.new_binding.keys.insert(pos, mod1.to_lowercase());
                } else {
                    //TODO: show error to user in case `i` is higher than len(), this shouldn't
                    //happen though
                    println!(
                        "Failed to add mod {mod1} to binding with index {pos} since len is {}",
                        self.new_binding.keys.len()
                    );
                }
            }
            Message::ChangeNewBindingKey(key) => {
                let sb = split_binding(&self.new_binding);
                let keys_count = sb.keys.len();
                let mods_count = sb.modifiers.len();
                if key.is_empty() {
                    if keys_count == 1 {
                        self.new_binding.keys.pop();
                    } else if keys_count >= 2 {
                        //TODO: show error to user
                        println!(
                            "Failed to remove key {key} from new binding since key count is {}, should be <=1",
                            keys_count
                        );
                    }
                } else {
                    println!("Adding/Updating a key");
                    if keys_count == 1 {
                        let kk = key.split(SEPARATOR).map(String::from).collect::<Vec<_>>();
                        if kk.len() > 1 {
                            self.new_binding.keys.truncate(mods_count);
                            kk.into_iter().for_each(|k| {
                                println!("Adding key {k}...");
                                self.new_binding.keys.push(k);
                            });
                        } else if let Some(k) = self.new_binding.keys.last_mut() {
                            *k = key;
                        } else {
                            self.new_binding.keys.push(key);
                        }
                    } else if keys_count == 0 {
                        self.new_binding.keys.push(key);
                    } else {
                        //TODO: show error to user
                        // println!("Failed to add key {key} to new binding since key count is {}, should be <=1", keys_count);
                        self.new_binding
                            .keys
                            .truncate(self.new_binding.keys.len() - keys_count);
                        key.split(SEPARATOR).map(String::from).for_each(|k| {
                            println!("Adding key {k}...");
                            self.new_binding.keys.push(k);
                        });
                    }
                }
            }
            Message::ChangeNewBindingCommand(command) => self.new_binding.command = command,
            Message::ToggleShowNewBinding => self.show_new_binding = !self.show_new_binding,
            Message::EditBinding(idx) => {
                self.editing.insert(idx);
                self.editing_states
                    .insert(idx, combo_box::State::new(commands.to_vec()));
            }
            Message::FinishEditBinding(idx) => {
                self.editing.remove(&idx);
                self.editing_states.remove(&idx);
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
        let add_new_binding_button =
            widget::button_with_icon(icons::plus(), text("Add New Binding"))
                .on_press(Message::ToggleShowNewBinding)
                .style(button::secondary)
                .into();

        let new_binding = if self.show_new_binding {
            let keybind = opt_helpers::opt_custom_el_disable_default(
                "Keybind",
                Some("A key combination to trigger the following command."),
                keys(
                    &self.new_binding,
                    Message::ChangeNewBindingKey,
                    Message::ChangeNewBindingMod,
                ),
                false,
                None,
                None,
            );
            let command = command_edit(
                Some(&self.new_binding_state),
                &self.new_binding.command,
                commands,
                commands_desc,
                theme,
                Message::ChangeNewBindingCommand,
            );

            let add_binding_button = button_with_icon(icons::plus(), "Add")
                .on_press(Message::AddNewBinding)
                .width(77);

            column![
                keybind,
                command,
                column![add_binding_button] //, row![copy_button, paste_button].spacing(5)]
                    .align_x(iced::Right)
                    .spacing(10),
            ]
            .spacing(10)
            .into()
        } else {
            space().into()
        };

        let col = column![];
        let bindings = whkdrc
            .bindings
            .iter()
            .enumerate()
            .fold(col, |col, (idx, binding)| {
                if self.editing.contains(&idx) {
                    let keybind = opt_helpers::opt_custom_el_disable_default(
                        "Keybind",
                        Some("A key combination to trigger the following command."),
                        keys(
                            binding,
                            move |k| Message::ChangeBindingKey(idx, k),
                            move |pos, m| Message::ChangeBindingMod(idx, (pos, m)),
                        ),
                        false,
                        None,
                        None,
                    );
                    let command = command_edit(
                        self.editing_states.get(&idx),
                        &binding.command,
                        commands,
                        commands_desc,
                        theme,
                        move |c| Message::ChangeBindingCommand(idx, c),
                    );

                    let mut key_pressed = row![text("PRESSED: "), text!("{}", self.pressed_mod),];

                    key_pressed = key_pressed.push(
                        (!self.pressed_mod.is_empty() && !self.pressed_key.is_empty())
                            .then_some(text(SEPARATOR)),
                    );
                    key_pressed = key_pressed.push(text!("{}", self.pressed_key));

                    col.push(
                        container(
                            container(
                                column![
                                    keybind,
                                    command,
                                    key_pressed,
                                    row![
                                        space::horizontal(),
                                        button(icons::check())
                                            .on_press(Message::FinishEditBinding(idx)),
                                        button(icons::copy())
                                            .on_press(Message::Nothing)
                                            .style(button::secondary),
                                    ]
                                    .spacing(10),
                                ]
                                .spacing(10),
                            )
                            .padding(10)
                            .style(|t| container::Style {
                                background: None,
                                ..container::bordered_box(t)
                            }),
                        )
                        .padding(padding::top(5).bottom(5)),
                    )
                } else {
                    let keys_count = binding.keys.len();
                    let keybind = row![].push(
                        container(binding.keys.iter().enumerate().fold(
                            row![].spacing(5),
                            |r, (i, m)| {
                                if i == keys_count - 1 {
                                    r.push(text(m))
                                } else {
                                    r.push(text(m)).push(SEPARATOR)
                                }
                            },
                        ))
                        .padding(padding::all(2).left(4).right(4))
                        .style(container::dark),
                    );
                    let command = scrollable(text(&binding.command).wrapping(text::Wrapping::None))
                        .direction(scrollable::Direction::Horizontal(
                            scrollable::Scrollbar::new().width(5.0).scroller_width(5.0),
                        ))
                        .spacing(2.0);

                    let b = hover(
                        row![keybind, text(" -> "), command]
                            .height(30)
                            .align_y(iced::Center)
                            .padding(padding::right(60).top(2.5).bottom(2.5)),
                        right(
                            button(icons::edit())
                                .on_press(Message::EditBinding(idx))
                                .style(button::secondary),
                        )
                        .padding(padding::right(5))
                        .align_y(iced::Center),
                    );

                    col.push(b)
                }
            })
            .into();

        column![
            opt_helpers::section_view("Bindings:", [add_new_binding_button, new_binding]),
            opt_helpers::section_view("Bindings:", [bindings]),
        ]
        .spacing(10)
        .into()
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
                        s.push_str(SEPARATOR);
                    }
                    s.push_str(n);
                    s
                });
            Some(Message::KeyPress(k, m))
        });
        let release = keyboard::on_key_release(|_, _| Some(Message::KeyRelease));

        Subscription::batch([press, release])
    }

    pub fn load_new_commands(&mut self, commands: &[String]) {
        self.new_binding_state = combo_box::State::new(commands.to_vec());
    }
}

fn is_mod(key: &str) -> bool {
    MODIFIERS.contains(&key.to_uppercase().as_str())
}

fn mod_choose<'a>(
    binding_mods: &[String],
    pos: usize,
    on_mod_change: impl Fn(usize, String) -> Message + Clone + 'a,
) -> Option<Element<'a, Message>> {
    let on_mod_change_clone = on_mod_change.clone();
    let pl = move |k: String| -> Element<Message> {
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
        pick_list(options, Some(k), move |v| on_mod_change_clone(pos, v)).into()
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

fn keys<'a>(
    binding: &'a HotkeyBinding,
    on_key_change: impl Fn(String) -> Message + 'a,
    on_mod_change: impl Fn(usize, String) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let sb = split_binding(binding);
    let joined_key = sb.keys.join(SEPARATOR);
    let key = widget::input("", joined_key, on_key_change, None).width(75);
    column![
        row![]
            .push(mod_choose(sb.modifiers, 3, on_mod_change.clone()))
            .push(mod_choose(sb.modifiers, 2, on_mod_change.clone()))
            .push(mod_choose(sb.modifiers, 1, on_mod_change.clone()))
            .push(mod_choose(sb.modifiers, 0, on_mod_change))
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
    state: Option<&'a combo_box::State<String>>,
    command: &'a String,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
    on_command_change: impl Fn(String) -> Message + Clone + 'static,
) -> Element<'a, Message> {
    let (main_cmd, rest) = split_command(command, commands);
    widget::expandable::expandable(move |hovered, expanded| {
        let on_command_change_clone = on_command_change.clone();
        let label = if false {
            row![text("Command")]
                .push(widget::opt_helpers::reset_button(Some(
                    on_command_change_clone(String::new()),
                )))
                .spacing(5)
                .height(30)
                .align_y(iced::Center)
        } else {
            row![text("Command")].height(30).align_y(iced::Center)
        };

        let main = row![widget::opt_helpers::label_element_with_description(
            label,
            Some("A command that should run when the keybind above is triggered.")
        )]
        .push(widget::opt_helpers::disable_checkbox(None))
        .push({
            let commands_box = state.map(|state| {
                let rest = rest.to_string();
                let main_cmd = main_cmd.to_string();
                let on_command_change_clone = on_command_change.clone();
                combo_box(state, "", Some(&main_cmd), move |v| {
                    let cmd = if rest.is_empty() {
                        format!("komorebic {v}")
                    } else {
                        format!("komorebic {v} {rest}")
                    };
                    on_command_change_clone(cmd)
                })
            });
            let on_command_change_c = on_command_change.clone();
            let custom = widget::input("", command, on_command_change_c, None);
            container(
                column![
                    row!["Komorebic commands:", commands_box].spacing(5),
                    "Command:",
                    custom,
                    text(command),
                ]
                .max_width(700)
                .padding(padding::bottom(10))
                .spacing(10),
            )
            .align_right(iced::FillPortion(3))
        })
        .spacing(10);

        let top = commands_desc
            .get(main_cmd.strip_prefix("komorebic ").unwrap_or_default())
            .is_some()
            .then(|| {
                bottom_center(
                    row![
                        container(icons::info().size(12)).style(move |t| {
                            if hovered {
                                container::Style {
                                    text_color: Some(*crate::widget::BLUE),
                                    ..container::transparent(t)
                                }
                            } else {
                                container::transparent(t)
                            }
                        }),
                        text(" Command Documentation ").size(10),
                        if expanded {
                            icons::up_chevron().size(12)
                        } else {
                            icons::down_chevron().size(12)
                        },
                    ]
                    .align_y(iced::Center),
                )
                .padding(padding::bottom(-10.0))
            });

        hover(main, top)
    })
    .bottom_elements(move || {
        if let Some(items) =
            commands_desc.get(main_cmd.strip_prefix("komorebic ").unwrap_or_default())
        {
            vec![markdown(items, theme).map(Message::UrlClicked)]
        } else {
            vec![]
        }
    })
    .into()
}

fn split_command<'a>(command: &'a str, commands: &'a [String]) -> (&'a str, &'a str) {
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
    keys: &'a [String],
}
impl<'a> From<(&'a [String], &'a [String])> for SplitBinding<'a> {
    fn from(value: (&'a [String], &'a [String])) -> Self {
        Self {
            modifiers: value.0,
            keys: value.1,
        }
    }
}
impl<'a> From<(&'a [String; 0], &'a [String; 0])> for SplitBinding<'a> {
    fn from(value: (&'a [String; 0], &'a [String; 0])) -> Self {
        Self {
            modifiers: value.0,
            keys: value.1,
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
