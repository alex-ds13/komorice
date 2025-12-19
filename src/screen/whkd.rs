pub mod bindings;
pub mod helpers;

pub use bindings::Bindings;
pub use helpers::{get_vk_key_mods, modal_content};

use crate::{
    screen::View,
    whkd::{DEFAULT_WHKDRC, MODIFIERS, SEPARATOR, Shell, UNPADDED_SEPARATOR, WhkdBinary, Whkdrc},
    widget::{self, hover, icons, opt_helpers},
};

use std::collections::HashMap;

use iced::{
    Center, Element, Subscription, Task, Theme, keyboard, padding,
    widget::{
        bottom_center, button, column, combo_box, container, markdown, pick_list, row, stack, text,
        text_editor,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Shell(Shell),
    PBMod(usize, String),
    PBKey(String),
    PauseBinding(Option<Vec<String>>),
    PauseHook(Option<String>),
    PauseHookContentChange(text_editor::Action),
    // AppBindings(Vec<(Vec<String>, Vec<HotkeyBinding>)>),
    // AddNewAppBinding,
    // RemoveAppBinding(usize),
    // ChangeAppBindingKeys(usize, Vec<String>),
    // ChangeAppBindingProcessName(usize, String),
    // ChangeAppBindingCommand(usize, String),
    BindKey,
    CloseModal(bool),
    KeyPress(String, String),
    KeyRelease(String, String),
    Navigate(NavMessage),
    UrlClicked(markdown::Url),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    StopWhkd,
    StartWhkd,
}

#[derive(Debug, Default)]
pub struct Whkd {
    pressed_keys: Vec<String>,
    pressed_keys_temp: Vec<String>,
    pressed_mod: String,
    pause_hook_state: combo_box::State<String>,
    pause_hook_content: text_editor::Content,
    bind_key: bool,
}

impl Whkd {
    pub fn update(&mut self, message: Message, whkdrc: &mut Whkdrc) -> (Action, Task<Message>) {
        match message {
            Message::Shell(shell) => whkdrc.shell = shell,
            Message::BindKey => {
                self.bind_key = true;
                self.pressed_mod = String::new();
                self.pressed_keys = Vec::new();
                self.pressed_keys_temp = Vec::new();
                return (Action::StopWhkd, Task::none());
            }
            Message::CloseModal(save) => {
                self.bind_key = false;
                if save && (!self.pressed_mod.is_empty() || !self.pressed_keys.is_empty()) {
                    let modifiers = std::mem::take(&mut self.pressed_mod);
                    let mods = modifiers.split(&SEPARATOR).map(|s| s.to_string());
                    let keys = self.pressed_keys.drain(..);
                    let key_combination = mods.chain(keys).collect();
                    whkdrc.pause_binding = Some(key_combination);
                }
                return (Action::StartWhkd, Task::none());
            }
            Message::KeyPress(k, m) => {
                if self.pressed_keys_temp.is_empty() {
                    self.pressed_keys = vec![k.clone()];
                    self.pressed_keys_temp.push(k);
                } else if !self.pressed_keys_temp.contains(&k) {
                    self.pressed_keys_temp.push(k);
                    self.pressed_keys = self.pressed_keys_temp.clone();
                }
                self.pressed_mod = m;
            }
            Message::KeyRelease(key, _m) => {
                if let Some(pos) = self.pressed_keys_temp.iter().position(|k| k == &key) {
                    self.pressed_keys_temp.remove(pos);
                }
            }
            Message::PBMod(i, modifier) => {
                let sb = split_binding(&whkdrc.pause_binding);
                if modifier.is_empty() {
                    if i < sb.modifiers.len()
                        && let Some(pause_binding) = &mut whkdrc.pause_binding
                        && i < pause_binding.len()
                    {
                        pause_binding.remove(i);
                        if pause_binding.is_empty() {
                            whkdrc.pause_binding = None;
                        }
                    }
                } else if let Some(pause_binding) = &mut whkdrc.pause_binding {
                    if let Some(k) = pause_binding.iter_mut().filter(|m| is_mod(m)).nth(i) {
                        *k = modifier;
                    } else if i <= pause_binding.len() {
                        pause_binding.insert(i, modifier);
                    } else {
                        //TODO: show error to user in case `i` is higher than len(), this shouldn't
                        //happen though
                        println!(
                            "Failed to add mod {modifier} to pause_binding with index {i} since len is {}",
                            pause_binding.len()
                        );
                    }
                } else {
                    whkdrc.pause_binding = Some(vec![modifier]);
                }
            }
            Message::PBKey(keys) => {
                let sb = split_binding(&whkdrc.pause_binding);
                let mods_count = sb.modifiers.len();
                if let Some(pause_binding) = whkdrc.pause_binding.as_mut() {
                    let _ = pause_binding.split_off(mods_count);
                    if keys.is_empty() {
                        if pause_binding.is_empty() {
                            whkdrc.pause_binding = None;
                        }
                    } else {
                        let keys = keys.split(&UNPADDED_SEPARATOR).map(|s| s.to_string());
                        pause_binding.extend(keys);
                    }
                } else {
                    let keys = keys
                        .split(&UNPADDED_SEPARATOR)
                        .map(|s| s.to_string())
                        .collect();
                    whkdrc.pause_binding = Some(keys);
                };
            }
            Message::PauseBinding(binding) => whkdrc.pause_binding = binding,
            Message::PauseHook(pause_hook) => {
                if let Some(hook) = pause_hook.as_ref() {
                    self.pause_hook_content = text_editor::Content::with_text(hook);
                } else {
                    self.pause_hook_content = text_editor::Content::new();
                }
                whkdrc.pause_hook = pause_hook;
            }
            Message::PauseHookContentChange(action) => {
                self.pause_hook_content.perform(action);
                whkdrc.pause_hook = Some(self.pause_hook_content.text());
            }
            Message::Navigate(nav) => match nav {
                NavMessage::Forward => {}
                NavMessage::Back => {}
            },
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(
        &'a self,
        whkdrc: &'a Whkdrc,
        whkd_bin: &'a WhkdBinary,
        commands: &'a [String],
        commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
        theme: &'a Theme,
    ) -> View<'a, Message> {
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
        let bind_button = button(widget::icons::edit())
            .style(button::subtle)
            .on_press(Message::BindKey);
        let pause_binding = opt_helpers::opt_custom_el_disable_default(
            "Pause Binding",
            Some("Can be any hotkey combo to toggle all other hotkeys on/off. (default: None)"),
            row![bind_button, keys(&whkdrc.pause_binding)]
                .spacing(10)
                .align_y(Center),
            whkdrc.pause_binding.is_some(),
            Some(Message::PauseBinding(None)),
            None,
        );
        let pause_hook = hook_custom(
            &self.pause_hook_state,
            &self.pause_hook_content,
            &whkdrc.pause_hook,
            commands,
            commands_desc,
            theme,
        );

        let content = opt_helpers::section_view("Whkd:", [shell, pause_binding, pause_hook]);

        View::new(content).modal(
            self.bind_key.then_some(modal_content(
                &self.pressed_mod,
                &self.pressed_keys,
                whkd_bin,
                Message::CloseModal,
            )),
            Message::CloseModal(false),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = if self.bind_key {
            iced::event::listen_with(|event, status, _id| {
                if matches!(status, iced::event::Status::Captured) {
                    return None;
                }
                match event {
                    iced::Event::Keyboard(event) => match event {
                        keyboard::Event::KeyPressed {
                            key,
                            modified_key: _,
                            physical_key,
                            location,
                            modifiers,
                            text: _,
                        } => {
                            let (k, m) = get_vk_key_mods(key, physical_key, location, modifiers);
                            if !k.is_empty() {
                                Some(Message::KeyPress(k, m))
                            } else {
                                None
                            }
                        }
                        keyboard::Event::KeyReleased {
                            key,
                            modified_key: _,
                            physical_key,
                            location,
                            modifiers,
                        } => {
                            let (k, m) = get_vk_key_mods(key, physical_key, location, modifiers);
                            if !k.is_empty() {
                                Some(Message::KeyRelease(k, m))
                            } else {
                                None
                            }
                        }
                        keyboard::Event::ModifiersChanged(_modifiers) => None,
                    },
                    _ => None,
                }
            })
        } else {
            Subscription::none()
        };
        let navigation = navigation_sub().map(Message::Navigate);

        Subscription::batch([navigation, keys])
    }

    pub fn load_new_commands(&mut self, commands: &[String]) {
        self.pause_hook_state = combo_box::State::new(commands.to_vec());
    }

    /// Refreshes any internal state with a newly loaded config.
    pub fn refresh(&mut self, whkdrc: &Whkdrc) {
        if let Some(hook) = whkdrc.pause_hook.as_ref() {
            self.pause_hook_content = text_editor::Content::with_text(hook);
        } else {
            self.pause_hook_content = text_editor::Content::new();
        }
    }
}

fn is_mod(key: &str) -> bool {
    MODIFIERS.contains(&key.to_uppercase().as_str())
}

fn mod_choose<'a>(binding_mods: &[String], pos: usize) -> Option<Element<'a, Message>> {
    let pl = |k: String| -> Element<Message> {
        let mut options = vec![
            "".into(),
            "ctrl".into(),
            "shift".into(),
            "alt".into(),
            "win".into(),
        ];
        options.retain(|v| {
            !binding_mods
                .iter()
                .map(|m| m.to_lowercase())
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
    let joined_keys = sb.keys.join(UNPADDED_SEPARATOR);
    let hidden_str = if joined_keys.chars().count() <= 8 {
        format!("{:n^8}", "n".repeat(joined_keys.len()))
    } else {
        let rest = joined_keys.len() - 8;
        format!(
            "{:n^8}{}",
            "n".repeat(8),
            joined_keys.chars().take(rest).collect::<String>()
        )
    };
    let keys_hidden = container(text(hidden_str)).padding(5);
    let key = widget::input("", joined_keys, Message::PBKey, None);
    row![
        mod_choose(sb.modifiers, 3),
        mod_choose(sb.modifiers, 2),
        mod_choose(sb.modifiers, 1),
        mod_choose(sb.modifiers, 0),
        stack![keys_hidden, key],
    ]
    .align_y(Center)
    .spacing(5)
    .into()
}

fn hook_custom<'a>(
    state: &'a combo_box::State<String>,
    content: &'a text_editor::Content,
    pause_hook: &'a Option<String>,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let (hook_command, hook_custom) = split_pause_hook(pause_hook, commands);
    let is_dirty = pause_hook != &DEFAULT_WHKDRC.pause_hook;
    widget::expandable::expandable(move |hovered, expanded| {
        let label = if is_dirty {
            row![text("Pause Hook")]
                .push(widget::opt_helpers::reset_button(Some(Message::PauseHook(
                    DEFAULT_WHKDRC.pause_hook.clone(),
                ))))
                .spacing(5)
                .height(30)
                .align_y(Center)
        } else {
            row![text("Pause Hook")].height(30).align_y(Center)
        };

        let main = row![widget::opt_helpers::label_element_with_description(
            label,
            Some(
                "A command that should run when the keybind above is triggered. (default: None)\n\n\
                You can use this to pause komorebi along with whkd for example."
            )
        )]
        .push(widget::opt_helpers::disable_checkbox(None))
        .push({
            let custom = text_editor(content).on_action(Message::PauseHookContentChange);
            container(custom)
                .max_width(700)
                .align_right(iced::FillPortion(3))
        })
        .align_y(Center)
        .spacing(10);

        let top = bottom_center(
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
                text(" komorebic help ").size(10),
                if expanded {
                    icons::up_chevron().size(12)
                } else {
                    icons::down_chevron().size(12)
                },
            ]
            .align_y(iced::Center),
        )
        .padding(padding::bottom(-10.0));

        hover(main, top)
    })
    .bottom_elements(move || {
        let rest = hook_custom.to_string();
        let main_cmd = hook_command.to_string();
        let commands_box = combo_box(state, "", Some(&main_cmd), move |v| {
            let cmd = if rest.is_empty() {
                format!("komorebic {v}")
            } else {
                format!("komorebic {v} {rest}")
            };
            Message::PauseHook(Some(cmd))
        });
        let selector = container(
            column![
                row!["Komorebic commands:", container(commands_box)]
                    .spacing(5)
                    .align_y(Center),
            ]
            .max_width(700)
            .padding(padding::bottom(10))
            .spacing(10),
        )
        .align_left(iced::FillPortion(3))
        .into();
        if let Some(items) =
            commands_desc.get(hook_command.strip_prefix("komorebic ").unwrap_or_default())
        {
            vec![selector, markdown(items, theme).map(Message::UrlClicked)]
        } else {
            vec![selector]
        }
    })
    .dirty(is_dirty)
    .into()
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
