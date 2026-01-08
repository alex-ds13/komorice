use super::{MODIFIERS, SEPARATOR, UNPADDED_SEPARATOR, WhkdBinary, get_vk_key_mods, modal_content};

use crate::{
    screen::View,
    whkd::{HotkeyBinding, Whkdrc},
    widget::{self, button_with_icon, hover, icons, opt_helpers, unfocus},
};

use std::collections::{HashMap, HashSet};

use iced::{
    Center, Element, Subscription, Task, Theme, padding,
    widget::{
        bottom_center, button, column, combo_box, container, markdown, operation, pick_list, right,
        row, scrollable, space, stack, text, text_editor,
    },
};

const SCROLLABLE_ID: &str = "BINDINGS_SCROLLABLE";

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    ChangeNewBindingMod(usize, String),
    ChangeNewBindingKey(String),
    ChangeNewBindingCommand(String),
    ChangeNewBindingContent(text_editor::Action),
    ToggleShowNewBinding,
    AddNewBinding,
    RemoveBinding(usize),
    ChangeBindingMod(usize, (usize, String)),
    ChangeBindingKey(usize, String),
    ChangeBindingCommand(usize, String),
    ChangeBindingContent(usize, text_editor::Action),
    EditBinding(usize),
    FinishEditBinding(usize),

    KeyPress(String, String),
    KeyRelease(String, String),
    OpenNewBindingKeysModal,
    OpenBindingKeysModal(usize),
    CloseModal(bool),
    UrlClicked(markdown::Url),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    StopWhkd,
    StartWhkd,
}

#[derive(Debug)]
enum Modal {
    NewBinding,
    Binding(usize),
}

#[derive(Debug)]
pub struct Bindings {
    pressed_keys: Vec<String>,
    pressed_keys_temp: Vec<String>,
    pressed_mod: String,
    modal_opened: Option<Modal>,
    new_binding: HotkeyBinding,
    new_binding_state: combo_box::State<String>,
    new_binding_content: text_editor::Content,
    show_new_binding: bool,
    editing: HashSet<usize>,
    editing_states: HashMap<usize, combo_box::State<String>>,
    editing_contents: HashMap<usize, text_editor::Content>,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            pressed_keys: Default::default(),
            pressed_keys_temp: Default::default(),
            pressed_mod: Default::default(),
            modal_opened: Default::default(),
            new_binding: HotkeyBinding {
                keys: Vec::new(),
                command: String::new(),
                process_name: None,
            },
            new_binding_state: Default::default(),
            new_binding_content: text_editor::Content::new(),
            show_new_binding: false,
            editing: Default::default(),
            editing_states: Default::default(),
            editing_contents: Default::default(),
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
            Message::ChangeBindingMod(idx, (pos, modifier)) => {
                if let Some(binding) = whkdrc.bindings.get_mut(idx) {
                    let sb = split_binding(binding);
                    if modifier.is_empty() {
                        if pos < sb.modifiers.len() {
                            binding.keys.remove(pos);
                        }
                    } else if let Some(k) = binding.keys.iter_mut().filter(|m| is_mod(m)).nth(pos) {
                        *k = modifier;
                    } else if pos <= binding.keys.len() {
                        binding.keys.insert(pos, modifier);
                    } else {
                        //TODO: show error to user in case `i` is higher than len(), this shouldn't
                        //happen though
                        println!(
                            "Failed to add mod {modifier} to binding with index {pos} since len is {}",
                            binding.keys.len()
                        );
                    }
                }
            }
            Message::ChangeBindingKey(idx, keys) => {
                if let Some(binding) = whkdrc.bindings.get_mut(idx) {
                    let sb = split_binding(binding);
                    let mods_count = sb.modifiers.len();
                    let _ = binding.keys.split_off(mods_count);
                    if !keys.is_empty() {
                        let keys = keys.split(&UNPADDED_SEPARATOR).map(|s| s.to_string());
                        binding.keys.extend(keys);
                    }
                }
            }
            Message::ChangeBindingCommand(idx, command) => {
                if let Some((binding, content)) = whkdrc
                    .bindings
                    .get_mut(idx)
                    .zip(self.editing_contents.get_mut(&idx))
                {
                    *content = text_editor::Content::with_text(&command);
                    binding.command = command;
                }
            }
            Message::ChangeBindingContent(idx, action) => {
                if let Some((binding, content)) = whkdrc
                    .bindings
                    .get_mut(idx)
                    .zip(self.editing_contents.get_mut(&idx))
                {
                    content.perform(action);
                    binding.command = content.text();
                }
            }
            Message::AddNewBinding => {
                let default_binding = HotkeyBinding {
                    keys: Vec::new(),
                    command: String::new(),
                    process_name: None,
                };
                let new_binding = std::mem::replace(&mut self.new_binding, default_binding);
                self.new_binding_content = text_editor::Content::new();
                self.show_new_binding = false;
                whkdrc.bindings.push(new_binding);
                return (Action::None, operation::snap_to_end(SCROLLABLE_ID));
            }
            Message::RemoveBinding(idx) => {
                if whkdrc.bindings.len() > idx {
                    whkdrc.bindings.remove(idx);
                }
                self.editing.remove(&idx);
                self.editing_states.remove(&idx);
                self.editing_contents.remove(&idx);
            }
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
            Message::ChangeNewBindingMod(pos, modifier) => {
                let sb = split_binding(&self.new_binding);
                if modifier.is_empty() {
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
                    *k = modifier;
                } else if pos <= self.new_binding.keys.len() {
                    self.new_binding.keys.insert(pos, modifier);
                } else {
                    //TODO: show error to user in case `i` is higher than len(), this shouldn't
                    //happen though
                    println!(
                        "Failed to add mod {modifier} to binding with index {pos} since len is {}",
                        self.new_binding.keys.len()
                    );
                }
            }
            Message::ChangeNewBindingKey(keys) => {
                let sb = split_binding(&self.new_binding);
                let mods_count = sb.modifiers.len();
                let _ = self.new_binding.keys.split_off(mods_count);
                if !keys.is_empty() {
                    let keys = keys.split(&UNPADDED_SEPARATOR).map(|s| s.to_string());
                    self.new_binding.keys.extend(keys);
                }
            }
            Message::ChangeNewBindingCommand(command) => {
                self.new_binding_content = text_editor::Content::with_text(&command);
                self.new_binding.command = command;
            }
            Message::ChangeNewBindingContent(action) => {
                self.new_binding_content.perform(action);
                self.new_binding.command = self.new_binding_content.text();
            }
            Message::ToggleShowNewBinding => self.show_new_binding = !self.show_new_binding,
            Message::EditBinding(idx) => {
                self.editing.insert(idx);
                self.editing_states
                    .insert(idx, combo_box::State::new(commands.to_vec()));
                let content = if let Some(binding) = whkdrc.bindings.get(idx) {
                    text_editor::Content::with_text(&binding.command)
                } else {
                    text_editor::Content::new()
                };
                self.editing_contents.insert(idx, content);
            }
            Message::FinishEditBinding(idx) => {
                self.editing.remove(&idx);
                self.editing_states.remove(&idx);
                self.editing_contents.remove(&idx);
            }
            Message::OpenNewBindingKeysModal => {
                self.modal_opened = Some(Modal::NewBinding);
                self.pressed_mod = String::new();
                self.pressed_keys = Vec::new();
                self.pressed_keys_temp = Vec::new();
                return (Action::StopWhkd, unfocus());
            }
            Message::OpenBindingKeysModal(idx) => {
                self.modal_opened = Some(Modal::Binding(idx));
                self.pressed_mod = String::new();
                self.pressed_keys = Vec::new();
                self.pressed_keys_temp = Vec::new();
                return (Action::StopWhkd, unfocus());
            }
            Message::CloseModal(save) => {
                if save
                    && (!self.pressed_mod.is_empty() || !self.pressed_keys.is_empty())
                    && let Some(modal) = self.modal_opened.as_ref()
                {
                    let modifiers = std::mem::take(&mut self.pressed_mod);
                    let mods = modifiers.split(&SEPARATOR).map(|s| s.to_string());
                    let keys = self.pressed_keys.drain(..);
                    let key_combination = mods.chain(keys).collect();
                    match modal {
                        Modal::NewBinding => {
                            self.new_binding.keys = key_combination;
                        }
                        Modal::Binding(idx) => {
                            if let Some(binding) = whkdrc.bindings.get_mut(*idx) {
                                binding.keys = key_combination;
                            }
                        }
                    }
                }
                self.modal_opened = None;
                return (Action::StartWhkd, Task::none());
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
        let add_new_binding_button =
            widget::button_with_icon(icons::plus(), text("Add New Binding"))
                .on_press(Message::ToggleShowNewBinding)
                .style(button::secondary)
                .into();

        let mut new_binding_keys = self.new_binding.keys.clone();
        new_binding_keys.sort();

        let new_binding = if self.show_new_binding {
            let bind_button = button(widget::icons::edit())
                .style(button::subtle)
                .on_press(Message::OpenNewBindingKeysModal);
            let keybind = opt_helpers::opt_custom_el_disable_default(
                "Keybind",
                Some("A key combination to trigger the following command."),
                row![
                    bind_button,
                    keys(
                        &self.new_binding,
                        Message::ChangeNewBindingKey,
                        Message::ChangeNewBindingMod,
                    )
                ]
                .spacing(10)
                .align_y(Center),
                false,
                None,
                None,
            );
            let command = command_edit(
                Some(&self.new_binding_state),
                &self.new_binding_content,
                &self.new_binding.command,
                commands,
                commands_desc,
                theme,
                Message::ChangeNewBindingCommand,
                Message::ChangeNewBindingContent,
            );

            let duplicated_keys = whkdrc.bindings.iter().any(|b| {
                let mut b_keys = b.keys.clone();
                b_keys.sort();
                b_keys == new_binding_keys
            });
            let duplicated_warning = duplicated_keys.then_some(
                text(
                    "This key combination is already being used! \
                    Either use another combination or change/delete \
                    the existing one first.",
                )
                .width(iced::Fill)
                .align_x(iced::Right)
                .size(12)
                .style(text::warning),
            );

            let add_binding_button = button_with_icon(icons::plus(), "Add")
                .on_press_maybe(
                    (!self.new_binding.keys.is_empty()
                        && !self.new_binding.command.is_empty()
                        && !duplicated_keys)
                        .then_some(Message::AddNewBinding),
                )
                .width(77);

            column![
                keybind,
                command,
                right(
                    row![duplicated_warning, add_binding_button]
                        .align_y(Center)
                        .spacing(10)
                ),
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
                let mut binding_keys = binding.keys.clone();
                binding_keys.sort();
                let equals_new_binding =
                    !self.new_binding.keys.is_empty() && binding_keys == new_binding_keys;
                let duplicated_keys = equals_new_binding
                    || whkdrc.bindings.iter().enumerate().any(|(b_idx, b)| {
                        let mut b_keys = b.keys.clone();
                        b_keys.sort();
                        b_idx != idx && b_keys == binding_keys
                    });

                if self.editing.contains(&idx) {
                    let bind_button = button(widget::icons::edit())
                        .style(button::subtle)
                        .on_press(Message::OpenBindingKeysModal(idx));
                    let keybind = opt_helpers::opt_custom_el_disable_default(
                        "Keybind",
                        Some("A key combination to trigger the following command."),
                        row![
                            bind_button,
                            keys(
                                binding,
                                move |k| Message::ChangeBindingKey(idx, k),
                                move |pos, m| Message::ChangeBindingMod(idx, (pos, m)),
                            )
                        ]
                        .spacing(10)
                        .align_y(Center),
                        false,
                        None,
                        None,
                    );
                    let command = command_edit(
                        self.editing_states.get(&idx),
                        self.editing_contents
                            .get(&idx)
                            .expect("should have editing content"),
                        &binding.command,
                        commands,
                        commands_desc,
                        theme,
                        move |c| Message::ChangeBindingCommand(idx, c),
                        move |a| Message::ChangeBindingContent(idx, a),
                    );

                    let duplicated_warning = duplicated_keys.then_some(
                        text(
                            "This key combination is already being used! \
                            Either use another combination or change/delete \
                            the existing one.",
                        )
                        .width(iced::Fill)
                        .align_x(iced::Right)
                        .size(12)
                        .style(text::warning),
                    );

                    col.push(
                        container(
                            container(
                                column![
                                    keybind,
                                    command,
                                    row![
                                        space::horizontal(),
                                        duplicated_warning,
                                        button(icons::check())
                                            .on_press(Message::FinishEditBinding(idx)),
                                        button(icons::delete())
                                            .on_press(Message::RemoveBinding(idx))
                                            .style(button::danger),
                                    ]
                                    .align_y(Center)
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
                                    r.push(text(m.trim()))
                                } else {
                                    r.push(text(m.trim())).push(SEPARATOR)
                                }
                            },
                        ))
                        .padding(padding::all(2).left(4).right(4))
                        .style(move |t: &Theme| {
                            if duplicated_keys {
                                let warning = t.extended_palette().warning;
                                container::Style {
                                    background: Some(warning.base.color.into()),
                                    text_color: Some(warning.base.text),
                                    ..container::dark(t)
                                }
                            } else {
                                container::dark(t)
                            }
                        }),
                    );
                    let command = scrollable(text(&binding.command).wrapping(text::Wrapping::None))
                        .direction(scrollable::Direction::Horizontal(
                            scrollable::Scrollbar::new().width(5.0).scroller_width(5.0),
                        ))
                        .spacing(2.0);

                    let b = hover(
                        row![keybind, text(" -> "), command]
                            .height(30)
                            .align_y(Center)
                            .padding(padding::right(60).top(2.5).bottom(2.5)),
                        right(
                            button(icons::edit())
                                .on_press(Message::EditBinding(idx))
                                .style(button::secondary),
                        )
                        .padding(padding::right(5))
                        .align_y(Center),
                    );

                    col.push(b)
                }
            })
            .into();

        let content = column![
            opt_helpers::section_view("Bindings:", [add_new_binding_button, new_binding]),
            opt_helpers::section_view_id("Bindings:", SCROLLABLE_ID, [bindings]),
        ]
        .spacing(10);

        View::new(content).modal(
            self.modal_opened.is_some().then_some(modal_content(
                &self.pressed_mod,
                &self.pressed_keys,
                whkd_bin,
                Message::CloseModal,
            )),
            Message::CloseModal(false),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        if self.modal_opened.is_some() {
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
        }
    }

    /// Refreshes any internal state with a newly loaded config.
    pub fn refresh(&mut self, whkdrc: &Whkdrc) {
        self.modal_opened = None;
        if !self.editing.is_empty() {
            self.editing.iter().for_each(|idx| {
                let content = if let Some(binding) = whkdrc.bindings.get(*idx) {
                    text_editor::Content::with_text(&binding.command)
                } else {
                    text_editor::Content::new()
                };
                self.editing_contents.insert(*idx, content);
            });
        }
    }

    pub fn load_new_commands(&mut self, commands: &[String]) {
        self.new_binding_state = combo_box::State::new(commands.to_vec());
    }

    pub fn clear_editing(&mut self) {
        self.editing.clear();
        self.editing_states.clear();
        self.editing_contents.clear();
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
    let key = widget::input("", joined_keys, on_key_change, None);
    row![
        mod_choose(sb.modifiers, 3, on_mod_change.clone()),
        mod_choose(sb.modifiers, 2, on_mod_change.clone()),
        mod_choose(sb.modifiers, 1, on_mod_change.clone()),
        mod_choose(sb.modifiers, 0, on_mod_change),
        stack![keys_hidden, key],
    ]
    .align_y(Center)
    .spacing(5)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn command_edit<'a>(
    state: Option<&'a combo_box::State<String>>,
    content: &'a text_editor::Content,
    command: &'a str,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
    on_command_change: impl Fn(String) -> Message + Clone + 'static,
    on_content_change: impl Fn(text_editor::Action) -> Message + Clone + 'static,
) -> Element<'a, Message> {
    let (main_cmd, _rest) = split_command(command, commands);
    let on_command_change_clone = on_command_change.clone();
    widget::expandable::expandable(move |hovered, expanded| {
        let on_command_change_clone = on_command_change_clone.clone();
        let label = if false {
            row![text("Command")]
                .push(widget::opt_helpers::reset_button(Some(
                    on_command_change_clone(String::new()),
                )))
                .spacing(5)
                .height(30)
                .align_y(Center)
        } else {
            row![text("Command")].height(30).align_y(Center)
        };

        let main = row![widget::opt_helpers::label_element_with_description(
            label,
            Some("A command that should run when the keybind above is triggered.")
        )]
        .push(widget::opt_helpers::disable_checkbox(None))
        .push({
            let custom = text_editor(content).on_action(on_content_change.clone());
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
            .align_y(Center),
        )
        .padding(padding::bottom(-10.0));

        hover(main, top)
    })
    .bottom_elements(move || {
        let commands_box = state.map(|state| {
            let main_cmd = main_cmd.to_string();
            let on_command_change_clone = on_command_change.clone();
            combo_box(state, "", Some(&main_cmd), move |v| {
                on_command_change_clone(format!("komorebic {v}"))
            })
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
            commands_desc.get(main_cmd.strip_prefix("komorebic ").unwrap_or_default())
        {
            vec![selector, markdown(items, theme).map(Message::UrlClicked)]
        } else {
            vec![selector]
        }
    })
    .into()
}

fn split_command<'a>(command: &'a str, commands: &'a [String]) -> (&'a str, &'a str) {
    let prefix = "komorebic ";
    let prefix_len = prefix.len();
    let mut final_command = "";
    let final_custom;
    if let Some(actual_command) = command.strip_prefix(prefix)
        && let Some(actual_command) = actual_command.split_whitespace().next()
        && let Some((cmd, custom)) = commands
            .iter()
            .find_map(|c| (actual_command == c).then(|| command.split_at(c.len() + prefix_len)))
    {
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
