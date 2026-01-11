use super::{MODIFIERS, SEPARATOR, UNPADDED_SEPARATOR, WhkdBinary, get_vk_key_mods, modal_content};

use crate::{
    BOLD_FONT,
    screen::View,
    whkd::{HotkeyBinding, Whkdrc},
    widget::{
        self, button_with_icon, hover, icons,
        opt_helpers::{self, DisableArgs},
        unfocus,
    },
};

use std::collections::{HashMap, HashSet};

use iced::{
    Center, Element, Fill, Shrink, Subscription, Task, Theme, Top, padding,
    widget::{
        bottom_center, button, column, combo_box, container, markdown, operation, pick_list, right,
        right_center, row, rule, scrollable, space, stack, text, text_editor,
    },
};
use smol::process::{Command, Stdio, windows::CommandExt};

const SCROLLABLE_ID: &str = "BINDINGS_SCROLLABLE";
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    // New Binding messages
    ChangeNewBindingMod(usize, String),
    ChangeNewBindingKey(String),
    ChangeNewBindingCommand(usize, String),
    ChangeNewBindingContent(usize, text_editor::Action),
    ChangeNewBindingProcess(usize, String),
    RemoveNewBindingSubBinding(usize),
    ToggleShowNewBinding,
    AddNewBindingCommand,
    AddNewBinding,

    // App Bindings messages
    RemoveBinding(usize),
    RemoveBindingSubBinding(usize, usize),
    AddBindingCommand(usize),
    ChangeBindingMod(usize, (usize, String)),
    ChangeBindingKey(usize, String),
    ChangeBindingCommand(usize, usize, String),
    ChangeBindingContent(usize, usize, text_editor::Action),
    ChangeBindingProcess(usize, usize, String),
    EditBinding(usize),
    FinishEditBinding(usize),

    // Keybind modal messages
    KeyPress(String, String),
    KeyRelease(String, String),
    OpenNewBindingKeysModal,
    OpenBindingKeysModal(usize),
    CloseModal(bool),

    // ProcessNames messages
    GotProcessNames(HashMap<String, String>),
    FailedToGetProcessNames,
    SelectNewBindingProcessName(usize),
    SelectBindingProcessName(usize, usize),
    SelectedProcessName(String),

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
    Keys(BindType),
    ProcessName(BindType, usize),
}

#[derive(Debug)]
enum BindType {
    New,
    Existing(usize),
}

#[derive(Debug)]
pub struct AppBindings {
    pressed_keys: Vec<String>,
    pressed_keys_temp: Vec<String>,
    pressed_mod: String,
    modal_opened: Option<Modal>,
    new_binding: (Vec<String>, Vec<HotkeyBinding>),
    new_binding_state: Vec<combo_box::State<String>>,
    new_binding_content: Vec<text_editor::Content>,
    new_binding_process: Vec<text_editor::Content>,
    show_new_binding: bool,
    editing: HashSet<usize>,
    editing_states: HashMap<usize, HashMap<usize, combo_box::State<String>>>,
    editing_commands: HashMap<usize, HashMap<usize, text_editor::Content>>,
    editing_processes: HashMap<usize, HashMap<usize, text_editor::Content>>,
    process_names: Option<HashMap<String, String>>,
    selected_process_name: Option<String>,
    selecting_process_state: combo_box::State<String>,
}

impl Default for AppBindings {
    fn default() -> Self {
        Self {
            pressed_keys: Default::default(),
            pressed_keys_temp: Default::default(),
            pressed_mod: Default::default(),
            modal_opened: Default::default(),
            new_binding: (
                Vec::new(),
                vec![HotkeyBinding {
                    keys: Vec::new(),
                    command: String::new(),
                    process_name: None,
                }],
            ),
            new_binding_state: Default::default(),
            new_binding_content: vec![text_editor::Content::new()],
            new_binding_process: vec![text_editor::Content::new()],
            show_new_binding: false,
            editing: Default::default(),
            editing_states: Default::default(),
            editing_commands: Default::default(),
            editing_processes: Default::default(),
            process_names: Default::default(),
            selected_process_name: Default::default(),
            selecting_process_state: Default::default(),
        }
    }
}

impl AppBindings {
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
                if let Some(app_binding) = whkdrc.app_bindings.get_mut(idx) {
                    for keys in app_binding
                        .1
                        .iter_mut()
                        .map(|b| &mut b.keys)
                        .chain(std::iter::once(&mut app_binding.0))
                    {
                        let sb = split_keys(keys);
                        if modifier.is_empty() {
                            if pos < sb.modifiers.len() {
                                keys.remove(pos);
                            }
                        } else if let Some(k) = keys.iter_mut().filter(|m| is_mod(m)).nth(pos) {
                            *k = modifier.clone();
                        } else if pos <= keys.len() {
                            keys.insert(pos, modifier.clone());
                        } else {
                            //TODO: show error to user in case `i` is higher than len(), this shouldn't
                            //happen though
                            println!(
                                "Failed to add mod {modifier} to binding with index {pos} since len is {}",
                                keys.len()
                            );
                        }
                    }
                }
            }
            Message::ChangeBindingKey(idx, keys) => {
                if let Some(app_binding) = whkdrc.app_bindings.get_mut(idx) {
                    for binding_keys in app_binding
                        .1
                        .iter_mut()
                        .map(|b| &mut b.keys)
                        .chain(std::iter::once(&mut app_binding.0))
                    {
                        let sb = split_keys(binding_keys);
                        let mods_count = sb.modifiers.len();
                        let _ = binding_keys.split_off(mods_count);
                        if !keys.is_empty() {
                            let keys = keys.split(&UNPADDED_SEPARATOR).map(|s| s.to_string());
                            binding_keys.extend(keys);
                        }
                    }
                }
            }
            Message::ChangeBindingCommand(idx, binding_idx, command) => {
                if let Some((binding, content)) = whkdrc
                    .app_bindings
                    .get_mut(idx)
                    .and_then(|ab| ab.1.get_mut(binding_idx))
                    .zip(
                        self.editing_commands
                            .get_mut(&idx)
                            .and_then(|ec| ec.get_mut(&binding_idx)),
                    )
                {
                    *content = text_editor::Content::with_text(&command);
                    binding.command = command;
                }
            }
            Message::ChangeBindingContent(idx, binding_idx, action) => {
                if let Some((binding, content)) = whkdrc
                    .app_bindings
                    .get_mut(idx)
                    .and_then(|ab| ab.1.get_mut(binding_idx))
                    .zip(
                        self.editing_commands
                            .get_mut(&idx)
                            .and_then(|ec| ec.get_mut(&binding_idx)),
                    )
                {
                    content.perform(action);
                    binding.command = content.text();
                }
            }
            Message::ChangeBindingProcess(idx, binding_idx, process) => {
                if let Some((binding, content)) = whkdrc
                    .app_bindings
                    .get_mut(idx)
                    .and_then(|ab| ab.1.get_mut(binding_idx))
                    .zip(
                        self.editing_processes
                            .get_mut(&idx)
                            .and_then(|ep| ep.get_mut(&binding_idx)),
                    )
                {
                    *content = text_editor::Content::with_text(&process);
                    binding.process_name = (!process.is_empty()).then_some(process);
                }
            }
            Message::AddNewBinding => {
                let default_binding = (
                    Vec::new(),
                    vec![HotkeyBinding {
                        keys: Vec::new(),
                        command: String::new(),
                        process_name: None,
                    }],
                );
                let new_binding = std::mem::replace(&mut self.new_binding, default_binding);
                self.new_binding_content = vec![text_editor::Content::new()];
                self.show_new_binding = false;
                whkdrc.app_bindings.push(new_binding);
                return (Action::None, operation::snap_to_end(SCROLLABLE_ID));
            }
            Message::RemoveBinding(idx) => {
                if whkdrc.app_bindings.len() > idx {
                    whkdrc.app_bindings.remove(idx);
                }
                self.editing.remove(&idx);
                self.editing_states.remove(&idx);
                self.editing_commands.remove(&idx);
            }
            Message::RemoveBindingSubBinding(idx, binding_idx) => {
                if let Some(((binding, states), commands)) = whkdrc
                    .app_bindings
                    .get_mut(idx)
                    .zip(self.editing_states.get_mut(&idx))
                    .zip(self.editing_commands.get_mut(&idx))
                {
                    if binding.1.len() > binding_idx {
                        binding.1.remove(binding_idx);
                    }
                    states.remove(&binding_idx);
                    commands.remove(&binding_idx);
                }
            }
            Message::AddBindingCommand(idx) => {
                if let Some((((binding, states), edit_commands), processes)) = whkdrc
                    .app_bindings
                    .get_mut(idx)
                    .zip(self.editing_states.get_mut(&idx))
                    .zip(self.editing_commands.get_mut(&idx))
                    .zip(self.editing_processes.get_mut(&idx))
                {
                    binding.1.push(HotkeyBinding {
                        keys: binding.0.clone(),
                        command: String::new(),
                        process_name: None,
                    });
                    states.insert(states.len(), combo_box::State::new(commands.to_vec()));
                    edit_commands.insert(edit_commands.len(), text_editor::Content::new());
                    processes.insert(processes.len(), text_editor::Content::new());
                }
            }
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
            Message::ChangeNewBindingMod(pos, modifier) => {
                for binding_keys in self
                    .new_binding
                    .1
                    .iter_mut()
                    .map(|b| &mut b.keys)
                    .chain(std::iter::once(&mut self.new_binding.0))
                {
                    let sb = split_keys(binding_keys);
                    if modifier.is_empty() {
                        if pos < sb.modifiers.len() {
                            binding_keys.remove(pos);
                        }
                    } else if let Some(k) = binding_keys.iter_mut().filter(|m| is_mod(m)).nth(pos) {
                        *k = modifier.clone();
                    } else if pos <= binding_keys.len() {
                        binding_keys.insert(pos, modifier.clone());
                    } else {
                        //TODO: show error to user in case `i` is higher than len(), this shouldn't
                        //happen though
                        println!(
                            "Failed to add mod {modifier} to binding with index {pos} since len is {}",
                            binding_keys.len()
                        );
                    }
                }
            }
            Message::ChangeNewBindingKey(keys) => {
                for binding_keys in self
                    .new_binding
                    .1
                    .iter_mut()
                    .map(|b| &mut b.keys)
                    .chain(std::iter::once(&mut self.new_binding.0))
                {
                    let sb = split_keys(binding_keys);
                    let mods_count = sb.modifiers.len();
                    let _ = binding_keys.split_off(mods_count);
                    if !keys.is_empty() {
                        let keys = keys.split(&UNPADDED_SEPARATOR).map(|s| s.to_string());
                        binding_keys.extend(keys);
                    }
                }
            }
            Message::ChangeNewBindingCommand(idx, command) => {
                if let Some((content, binding)) = self
                    .new_binding_content
                    .get_mut(idx)
                    .zip(self.new_binding.1.get_mut(idx))
                {
                    *content = text_editor::Content::with_text(&command);
                    binding.command = command;
                }
            }
            Message::ChangeNewBindingContent(idx, action) => {
                if let Some((content, binding)) = self
                    .new_binding_content
                    .get_mut(idx)
                    .zip(self.new_binding.1.get_mut(idx))
                {
                    content.perform(action);
                    binding.command = content.text();
                }
            }
            Message::ChangeNewBindingProcess(idx, process) => {
                if let Some((content, binding)) = self
                    .new_binding_process
                    .get_mut(idx)
                    .zip(self.new_binding.1.get_mut(idx))
                {
                    *content = text_editor::Content::with_text(&process);
                    binding.process_name = Some(process);
                }
            }
            Message::AddNewBindingCommand => {
                self.new_binding.1.push(HotkeyBinding {
                    keys: self.new_binding.0.clone(),
                    command: String::new(),
                    process_name: None,
                });
                self.new_binding_state
                    .push(combo_box::State::new(commands.to_vec()));
                self.new_binding_content.push(text_editor::Content::new());
                self.new_binding_process.push(text_editor::Content::new());
            }
            Message::RemoveNewBindingSubBinding(binding_idx) => {
                if self.new_binding.1.len() > binding_idx {
                    self.new_binding.1.remove(binding_idx);
                }
                if self.new_binding_state.len() > binding_idx {
                    self.new_binding_state.remove(binding_idx);
                }
                if self.new_binding_content.len() > binding_idx {
                    self.new_binding_content.remove(binding_idx);
                }
                if self.new_binding_process.len() > binding_idx {
                    self.new_binding_process.remove(binding_idx);
                }
            }
            Message::ToggleShowNewBinding => self.show_new_binding = !self.show_new_binding,
            Message::EditBinding(idx) => {
                self.editing.insert(idx);
                if let Some(app_binding) = whkdrc.app_bindings.get(idx) {
                    let bindings_count = app_binding.1.len();
                    for i in 0..bindings_count {
                        self.editing_states
                            .entry(idx)
                            .or_insert(HashMap::new())
                            .insert(i, combo_box::State::new(commands.to_vec()));
                        let (command, process) = if let Some(binding) = app_binding.1.get(i) {
                            (
                                text_editor::Content::with_text(&binding.command),
                                binding
                                    .process_name
                                    .as_ref()
                                    .map_or(text_editor::Content::new(), |process| {
                                        text_editor::Content::with_text(process)
                                    }),
                            )
                        } else {
                            (text_editor::Content::new(), text_editor::Content::new())
                        };
                        self.editing_commands
                            .entry(idx)
                            .or_insert(HashMap::new())
                            .insert(i, command);
                        self.editing_processes
                            .entry(idx)
                            .or_insert(HashMap::new())
                            .insert(i, process);
                    }
                }
            }
            Message::FinishEditBinding(idx) => {
                self.editing.remove(&idx);
                self.editing_states.remove(&idx);
                self.editing_commands.remove(&idx);
            }
            Message::OpenNewBindingKeysModal => {
                self.modal_opened = Some(Modal::Keys(BindType::New));
                self.pressed_mod = String::new();
                self.pressed_keys = Vec::new();
                self.pressed_keys_temp = Vec::new();
                return (Action::StopWhkd, unfocus());
            }
            Message::OpenBindingKeysModal(idx) => {
                self.modal_opened = Some(Modal::Keys(BindType::Existing(idx)));
                self.pressed_mod = String::new();
                self.pressed_keys = Vec::new();
                self.pressed_keys_temp = Vec::new();
                return (Action::StopWhkd, unfocus());
            }
            Message::CloseModal(save) => {
                if save && let Some(modal) = self.modal_opened.as_ref() {
                    match modal {
                        Modal::Keys(bind_type) => {
                            if !self.pressed_mod.is_empty() || !self.pressed_keys.is_empty() {
                                let modifiers = std::mem::take(&mut self.pressed_mod);
                                let mods = modifiers.split(&SEPARATOR).map(|s| s.to_string());
                                let keys = self.pressed_keys.drain(..);
                                let key_combination = mods.chain(keys).collect::<Vec<_>>();
                                match bind_type {
                                    BindType::New => {
                                        for k in self
                                            .new_binding
                                            .1
                                            .iter_mut()
                                            .map(|b| &mut b.keys)
                                            .chain(std::iter::once(&mut self.new_binding.0))
                                        {
                                            *k = key_combination.clone();
                                        }
                                    }
                                    BindType::Existing(idx) => {
                                        if let Some(app_binding) = whkdrc.app_bindings.get_mut(*idx)
                                        {
                                            for k in app_binding
                                                .1
                                                .iter_mut()
                                                .map(|b| &mut b.keys)
                                                .chain(std::iter::once(&mut app_binding.0))
                                            {
                                                *k = key_combination.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Modal::ProcessName(bind_type, binding_idx) => {
                            if let Some(selected) = self.selected_process_name.take()
                                && let Some((_, app_name)) = selected.split_once(" -> ")
                            {
                                match bind_type {
                                    BindType::New => {
                                        if let Some(process_name) = self
                                            .new_binding
                                            .1
                                            .get_mut(*binding_idx)
                                            .map(|b| &mut b.process_name)
                                        {
                                            *process_name = Some(app_name.to_string());
                                        }
                                    }
                                    BindType::Existing(idx) => {
                                        if let Some(app_binding) = whkdrc.app_bindings.get_mut(*idx)
                                            && let Some(process_name) = app_binding
                                                .1
                                                .get_mut(*binding_idx)
                                                .map(|b| &mut b.process_name)
                                        {
                                            *process_name = Some(app_name.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.modal_opened = None;
                self.selected_process_name = None;
                return (Action::StartWhkd, Task::none());
            }
            Message::GotProcessNames(process_names) => {
                let mut options = process_names
                    .iter()
                    .map(|(process_name, app_name)| format!("{process_name} -> {app_name}"))
                    .collect::<Vec<_>>();
                options.sort();
                self.selecting_process_state = combo_box::State::new(options);
                self.process_names = Some(process_names);
            }
            Message::FailedToGetProcessNames => {}
            Message::SelectNewBindingProcessName(binding_idx) => {
                self.modal_opened = Some(Modal::ProcessName(BindType::New, binding_idx));
                return (Action::None, get_current_process_names());
            }
            Message::SelectBindingProcessName(idx, binding_idx) => {
                self.modal_opened = Some(Modal::ProcessName(BindType::Existing(idx), binding_idx));
                return (Action::None, get_current_process_names());
            }
            Message::SelectedProcessName(process_name) => {
                self.selected_process_name = Some(process_name);
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

        let mut new_binding_keys = self.new_binding.0.clone();
        new_binding_keys.sort();

        let new_binding = if self.show_new_binding {
            let bind_button = button(widget::icons::edit())
                .style(button::subtle)
                .on_press(Message::OpenNewBindingKeysModal);
            let keybind = opt_helpers::opt_custom_el_disable_default(
                "Keybind",
                Some("A key combination to trigger the following commands."),
                row![
                    bind_button,
                    keys(
                        &self.new_binding.0,
                        Message::ChangeNewBindingKey,
                        Message::ChangeNewBindingMod,
                    )
                ]
                .spacing(10)
                .align_y(Center),
                false,
                None,
                DisableArgs::none(),
            );
            let commands_col = self.new_binding.1.iter().enumerate().fold(Vec::new(), |mut col, (binding_idx, binding)| {
                let is_default = binding.process_name.as_ref().is_some_and(|p| p == "Default");
                let can_be_default = is_default ||
                    !self.new_binding.1.iter().enumerate().filter_map(|(i, b)| {
                        (b.process_name == Some("Default".into())).then_some(i)
                    }).any(|i| i != binding_idx);

                let process = opt_helpers::opt_custom_el_disable_default(
                    "Process Name",
                    Some("Name of the process on which the following command will run when this key combination is pressed. \
                        If you set it to 'Default', it means the command will run on all apps. If you only have one command \
                        for 'Default' it is the same as a normal 'Binding'.
                        "),
                    container(
                        widget::input(
                            "",
                            binding.process_name.as_ref().map(String::as_str).unwrap_or(""),
                            move |v| Message::ChangeNewBindingProcess(binding_idx, v),
                            None,
                        )
                        .on_input_maybe((!is_default).then_some(
                            move |v| Message::ChangeNewBindingProcess(binding_idx, v),
                        ))
                    )
                    .max_width(400)
                    .align_right(Fill),
                    false,
                    None,
                    Some(DisableArgs::new(
                        is_default,
                        Some("Default"),
                        move |v| {
                            let value = if v {
                                String::from("Default")
                            } else {
                                String::new()
                            };
                            Message::ChangeNewBindingProcess(binding_idx, value)
                        },
                    ).blocked(!can_be_default)),
                );

                let command = command_edit(
                    Some(&self.new_binding_state[binding_idx]),
                    &self.new_binding_content[binding_idx],
                    &binding.command,
                    commands,
                    commands_desc,
                    theme,
                    move |s| Message::ChangeNewBindingCommand(binding_idx, s),
                    move |a| Message::ChangeNewBindingContent(binding_idx, a),
                    Some(DisableArgs::new(
                        self.new_binding_content[binding_idx].text().as_str() == "Ignore",
                        Some("Ignore"),
                        move |v| {
                            let value = if v { String::from("Ignore") } else { String::new() };
                            Message::ChangeNewBindingCommand(binding_idx, value)
                        }
                    )),
                );

                let process_names_button = widget::create_tooltip(
                    button(icons::info().size(14))
                        .on_press(Message::SelectNewBindingProcessName(binding_idx))
                        .padding(padding::all(2.5).left(5).right(5))
                        .style(button::secondary),
                    "Select a process name from the currently running processes",
                );

                let delete_button = widget::create_tooltip(
                    button(icons::delete().size(14))
                        .on_press(Message::RemoveNewBindingSubBinding(binding_idx))
                        .padding(padding::all(2.5).left(5).right(5))
                        .style(button::danger),
                    "Delete this app command"
                );

                col.push(
                    hover(
                        column![
                            process,
                            rule::horizontal(2.0),
                            command,
                        ],
                        right(
                            row![
                                process_names_button,
                                delete_button,
                            ]
                            .spacing(10)
                        )
                        .padding(10),
                    )
                    .into()
                );
                col
            });

            let duplicated_keys = whkdrc.app_bindings.iter().any(|b| {
                let mut b_keys = b.0.clone();
                b_keys.sort();
                b_keys == new_binding_keys
            });
            let duplicated_warning = duplicated_keys.then_some(
                text(
                    "This key combination is already being used! \
                    Either use another combination or change/delete \
                    the existing one first.",
                )
                .width(Fill)
                .align_x(iced::Right)
                .size(12)
                .style(text::warning),
            );

            let add_binding_button = button_with_icon(icons::plus(), "Add")
                .on_press_maybe(
                    (!self.new_binding.0.is_empty()
                        && !self.new_binding.1.is_empty()
                        && !duplicated_keys)
                        .then_some(Message::AddNewBinding),
                )
                .width(77);

            let add_command_button =
                button_with_icon(icons::level_down(), text("Add App Command").size(12))
                    .style(button::secondary)
                    .on_press(Message::AddNewBindingCommand);

            column![
                keybind,
                opt_helpers::sub_section_view(
                    text("App Specific Commands:").font(*BOLD_FONT),
                    commands_col
                ),
                row![
                    add_command_button,
                    space::horizontal(),
                    right(
                        row![duplicated_warning, add_binding_button]
                            .align_y(Center)
                            .spacing(10)
                    ),
                ]
            ]
            .spacing(10)
            .into()
        } else {
            space().into()
        };

        let col = column![];
        let bindings = whkdrc
            .app_bindings
            .iter()
            .enumerate()
            .fold(col, |col, (idx, app_binding)| {
                let mut binding_keys = app_binding.0.clone();
                binding_keys.sort();
                let equals_new_binding =
                    !self.new_binding.0.is_empty() && binding_keys == new_binding_keys;
                let duplicated_keys = equals_new_binding
                    || whkdrc.app_bindings.iter().enumerate().any(|(b_idx, b)| {
                        let mut b_keys = b.0.clone();
                        b_keys.sort();
                        b_idx != idx && b_keys == binding_keys
                    });

                if self.editing.contains(&idx) {
                    let bind_button = button(widget::icons::edit())
                        .style(button::subtle)
                        .on_press(Message::OpenBindingKeysModal(idx));
                    let keybind = opt_helpers::opt_custom_el_disable_default(
                        "Keybind",
                        Some("A key combination to trigger the following commands."),
                        row![
                            bind_button,
                            keys(
                                &app_binding.0,
                                move |k| Message::ChangeBindingKey(idx, k),
                                move |pos, m| Message::ChangeBindingMod(idx, (pos, m)),
                            )
                        ]
                        .spacing(10)
                        .align_y(Center),
                        false,
                        None,
                        DisableArgs::none(),
                    );
                    let commands_col = app_binding.1.iter().enumerate().fold(Vec::new(), |mut col, (binding_idx, binding)| {
                        let is_default = binding.process_name.as_ref().is_some_and(|p| p == "Default");
                        let can_be_default = is_default ||
                            !app_binding.1.iter().enumerate().filter_map(|(i, b)| {
                                (b.process_name == Some("Default".into())).then_some(i)
                            }).any(|i| i != binding_idx);

                        let process = opt_helpers::opt_custom_el_disable_default(
                            "Process Name",
                            Some("Name of the process on which the following command will run when this key combination is pressed. \
                                If you set it to 'Default', it means the command will run on all apps. If you only have one command \
                                for 'Default' it is the same as a normal 'Binding'.
                                "),
                            container(
                                widget::input(
                                    "",
                                    binding.process_name.as_ref().map(String::as_str).unwrap_or(""),
                                    move |v| Message::ChangeBindingProcess(idx, binding_idx, v),
                                    None,
                                )
                                .on_input_maybe((!is_default).then_some(
                                    move |v| Message::ChangeBindingProcess(idx, binding_idx, v),
                                ))
                            )
                            .max_width(400)
                            .align_right(Fill),
                            false,
                            None,
                            Some(DisableArgs::new(
                                is_default,
                                Some("Default"),
                                move |v| {
                                    let value = if v {
                                        String::from("Default")
                                    } else {
                                        String::new()
                                    };
                                    Message::ChangeBindingProcess(idx, binding_idx, value)
                                },
                            ).blocked(!can_be_default)),
                        );

                        let command = command_edit(
                            self.editing_states.get(&idx).and_then(|es| es.get(&binding_idx)),
                            self.editing_commands
                                .get(&idx)
                                .expect("should have editing content")
                                .get(&binding_idx)
                                .expect("should have editing content"),
                            &binding.command,
                            commands,
                            commands_desc,
                            theme,
                            move |s| Message::ChangeBindingCommand(idx, binding_idx, s),
                            move |a| Message::ChangeBindingContent(idx, binding_idx, a),
                            Some(DisableArgs::new(
                                binding.command.as_str() == "Ignore",
                                Some("Ignore"),
                                move |v| {
                                    let value = if v { String::from("Ignore") } else { String::new() };
                                    Message::ChangeBindingCommand(idx, binding_idx, value)
                                }
                            )),
                        );

                        let process_names_button = widget::create_tooltip(
                            button(icons::info().size(14))
                                .on_press(Message::SelectBindingProcessName(idx, binding_idx))
                                .padding(padding::all(2.5).left(5).right(5))
                                .style(button::secondary),
                            "Select a process name from the currently running processes",
                        );

                        let delete_button = widget::create_tooltip(
                            button(icons::delete().size(14))
                                .on_press(Message::RemoveBindingSubBinding(idx, binding_idx))
                                .padding(padding::all(2.5).left(5).right(5))
                                .style(button::danger),
                            "Delete this app command from this app binding"
                        );

                        col.push(
                            hover(
                                column![
                                    process,
                                    rule::horizontal(2.0),
                                    command,
                                ],
                                right(
                                    row![
                                        process_names_button,
                                        delete_button,
                                    ]
                                    .spacing(10)
                                )
                                .padding(10),
                            )
                            .into()
                        );
                        col
                    });

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

                    let add_command_button =
                        button_with_icon(icons::level_down(), text("Add App Command").size(12))
                            .style(button::secondary)
                            .on_press(Message::AddBindingCommand(idx));

                    col.push(
                        container(
                            container(
                                column![
                                    keybind,
                                    opt_helpers::sub_section_view(text("App Specific Commands:").font(*BOLD_FONT), commands_col),
                                    row![
                                        add_command_button,
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
                    let keys_count = app_binding.0.len();
                    let keybind = container(
                        container(app_binding.0.iter().enumerate().fold(
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
                    )
                    .align_y(Center)
                    .height(30);

                    let commands = app_binding.1.iter().enumerate().fold(
                        column![].spacing(5),
                        |col, (_binding_idx, binding)| {
                            let process_name = binding
                                .process_name
                                .as_ref()
                                .map(String::as_str)
                                .unwrap_or("");
                            let is_default = process_name == "Default";
                            let process = container(if is_default {
                                text!("{}", process_name)
                            } else {
                                text!("'{}'", process_name)
                            })
                            .padding(padding::all(2).left(4).right(4))
                            .style(move |t: &Theme| {
                                let palette = theme.extended_palette();
                                if duplicated_keys {
                                    let warning = t.extended_palette().warning;
                                    container::Style {
                                        background: Some(warning.base.color.into()),
                                        text_color: Some(warning.base.text),
                                        ..container::dark(t)
                                    }
                                } else if is_default {
                                    container::background(palette.background.weaker.color)
                                } else {
                                    container::background(palette.background.strongest.color)
                                }
                            });
                            let is_ignore = binding.command.as_str() == "Ignore";
                            let command = container(
                                scrollable(text(&binding.command).wrapping(text::Wrapping::None))
                                    .direction(scrollable::Direction::Horizontal(
                                        scrollable::Scrollbar::new().width(5.0).scroller_width(5.0),
                                    ))
                                    .spacing(2.0),
                            )
                            .padding(if is_ignore {
                                padding::all(2).left(4).right(4)
                            } else {
                                padding::Padding::ZERO
                            })
                            .style(move |t| {
                                let palette = theme.extended_palette();
                                if is_ignore {
                                    container::background(palette.background.weaker.color)
                                } else {
                                    container::transparent(t)
                                }
                            });
                            col.push(
                                row![text("On "), process, text(" -> "), command]
                                    .height(30)
                                    .align_y(Center)
                                    .padding(padding::right(60).top(2.5).bottom(2.5)),
                            )
                        },
                    );

                    let b = hover(
                        row![keybind, text(" -> ").align_y(Center).height(30), commands]
                            .align_y(Top)
                            .padding(padding::right(60).top(2.5).bottom(2.5)),
                        right_center(
                            button(icons::edit())
                                .on_press(Message::EditBinding(idx))
                                .style(button::secondary),
                        )
                        .padding(padding::right(5)),
                    );

                    col.push(b)
                }
            })
            .into();

        let content = column![
            opt_helpers::section_view("App Bindings:", [add_new_binding_button, new_binding]),
            opt_helpers::section_view_id("App Bindings:", SCROLLABLE_ID, [bindings]),
        ]
        .spacing(10);

        View::new(content).modal(self.view_modal(whkd_bin), Message::CloseModal(false))
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
                if let Some(app_binding) = whkdrc.app_bindings.get(*idx) {
                    let bindings_count = app_binding.1.len();
                    for i in 0..bindings_count {
                        let (command, process) = if let Some(binding) = app_binding.1.get(i) {
                            (
                                text_editor::Content::with_text(&binding.command),
                                binding
                                    .process_name
                                    .as_ref()
                                    .map_or(text_editor::Content::new(), |process| {
                                        text_editor::Content::with_text(process)
                                    }),
                            )
                        } else {
                            (text_editor::Content::new(), text_editor::Content::new())
                        };
                        self.editing_commands
                            .entry(*idx)
                            .or_insert(HashMap::new())
                            .insert(i, command);
                        self.editing_processes
                            .entry(*idx)
                            .or_insert(HashMap::new())
                            .insert(i, process);
                    }
                }
            });
        }
    }

    pub fn load_new_commands(&mut self, commands: &[String]) {
        self.new_binding_state =
            vec![combo_box::State::new(commands.to_vec()); self.new_binding.1.len()];
    }

    pub fn clear_editing(&mut self) {
        self.editing.clear();
        self.editing_states.clear();
        self.editing_commands.clear();
    }

    fn view_modal<'a>(&'a self, whkd_bin: &'a WhkdBinary) -> Option<Element<'a, Message>> {
        self.modal_opened.as_ref().map(|modal| match modal {
            Modal::Keys(_) => modal_content(
                &self.pressed_mod,
                &self.pressed_keys,
                whkd_bin,
                Message::CloseModal,
            ),
            Modal::ProcessName(_, _) => {
                if self.process_names.is_some()
                    && !self.selecting_process_state.options().is_empty()
                {
                    let pick = pick_list(
                        self.selecting_process_state.options(),
                        self.selected_process_name.as_ref(),
                        Message::SelectedProcessName,
                    );
                    let combobox = combo_box(
                        &self.selecting_process_state,
                        "",
                        self.selected_process_name.as_ref(),
                        Message::SelectedProcessName,
                    );
                    let picker = stack!(pick, combobox);

                    container(
                        column![
                            text("Select a process name:").size(18).font(*BOLD_FONT),
                            rule::horizontal(2.0),
                            container(text("Currently running processes:"))
                                .padding(padding::top(10)),
                            row![picker, button("Select").on_press(Message::CloseModal(true))]
                                .spacing(10),
                        ]
                        .width(Shrink)
                        .spacing(10),
                    )
                    .style(container::bordered_box)
                    .padding(50)
                    .into()
                } else {
                    container("Loading currently running process names...")
                        .style(container::bordered_box)
                        .padding(50)
                        .into()
                }
            }
        })
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
    keys: &'a [String],
    on_key_change: impl Fn(String) -> Message + 'a,
    on_mod_change: impl Fn(usize, String) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let sb = split_keys(keys);
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
fn command_edit<'a, F>(
    state: Option<&'a combo_box::State<String>>,
    content: &'a text_editor::Content,
    command: &'a str,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
    on_command_change: impl Fn(String) -> Message + Clone + 'static,
    on_content_change: impl Fn(text_editor::Action) -> Message + Clone + 'static,
    disable_args: Option<DisableArgs<'a, Message, F>>,
) -> Element<'a, Message>
where
    F: Fn(bool) -> Message + Clone + 'a,
{
    let (main_cmd, _rest) = split_command(command, commands);
    let disable_args_clone = disable_args.clone();
    widget::expandable::expandable(move |hovered, expanded| {
        let label = row![text("Command")].height(30).align_y(Center);

        let main = row![widget::opt_helpers::label_element_with_description(
            label,
            Some("A command that should run when the keybind above is triggered and the focus is on this process. \
                If you set it to 'Ignore', it means this keybind will be ignored when this process is focused."),
        )]
        .push(widget::opt_helpers::disable_checkbox(
            disable_args_clone.as_ref(),
        ))
        .push({
            let mut editor = text_editor(content);
            let disabled = disable_args_clone.as_ref().is_some_and(|args| args.disable);
            if !disabled {
                editor = editor.on_action(on_content_change.clone());
            }
            container(editor)
                .max_width(400)
                .align_right(Fill)
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
            .max_width(400)
            .padding(padding::bottom(10))
            .spacing(10),
        )
        .align_left(Fill)
        .into();

        if let Some(items) =
            commands_desc.get(main_cmd.strip_prefix("komorebic ").unwrap_or_default())
        {
            vec![selector, markdown(items, theme).map(Message::UrlClicked)]
        } else {
            vec![selector]
        }
    })
    .disable_args_maybe(disable_args)
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

fn split_keys(keys: &[String]) -> SplitBinding<'_> {
    if let Some(split_at) = keys
        .iter()
        .enumerate()
        .find_map(|(i, k)| (!is_mod(k)).then_some(i))
    {
        keys.split_at(split_at).into()
    } else {
        keys.split_at(keys.len()).into()
    }
}

fn get_current_process_names() -> Task<Message> {
    Task::perform(
        async {
            Command::new("powershell")
                .arg("-NoProfile")
                .arg("-C")
                .raw_arg("\"Get-Process | Select ProcessName, Description\"")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .await
        },
        |output| match output {
            Ok(output) => {
                if output.status.success() {
                    let output = String::from_utf8_lossy(&output.stdout);
                    let mut process_names = HashMap::new();
                    dbg!(&output);
                    output.trim().lines().skip(2).for_each(|line| {
                        let trimmed_line = line.trim();
                        let process_name =
                            trimmed_line.split_whitespace().next().unwrap_or_default();
                        if !process_name.is_empty() {
                            let (_, process_app_name) =
                                trimmed_line.split_at(process_name.chars().count());
                            let process_app_name = process_app_name.trim();
                            if !process_app_name.is_empty() {
                                process_names
                                    .insert(process_name.to_string(), process_app_name.to_string());
                            }
                        }
                    });
                    Message::GotProcessNames(process_names)
                } else {
                    Message::FailedToGetProcessNames
                }
            }
            _ => Message::FailedToGetProcessNames,
        },
    )
}
