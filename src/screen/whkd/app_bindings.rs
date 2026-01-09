use super::{MODIFIERS, SEPARATOR, UNPADDED_SEPARATOR, WhkdBinary, get_vk_key_mods, modal_content};

use crate::{
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
    Center, Element, Subscription, Task, Theme, Top, padding,
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
    ChangeNewBindingCommand(usize, String),
    ChangeNewBindingContent(usize, text_editor::Action),
    ChangeNewBindingProcess(usize, String),
    ToggleShowNewBinding,
    AddNewBinding,
    RemoveBinding(usize),
    ChangeBindingMod(usize, (usize, String)),
    ChangeBindingKey(usize, String),
    ChangeBindingCommand(usize, usize, String),
    ChangeBindingContent(usize, usize, text_editor::Action),
    ChangeBindingProcess(usize, usize, String),
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
                        let content = if let Some(binding) = app_binding.1.get(i) {
                            text_editor::Content::with_text(&binding.command)
                        } else {
                            text_editor::Content::new()
                        };
                        self.editing_commands
                            .entry(idx)
                            .or_insert(HashMap::new())
                            .insert(i, content);
                    }
                }
            }
            Message::FinishEditBinding(idx) => {
                self.editing.remove(&idx);
                self.editing_states.remove(&idx);
                self.editing_commands.remove(&idx);
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
                    let key_combination = mods.chain(keys).collect::<Vec<_>>();
                    match modal {
                        Modal::NewBinding => {
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
                        Modal::Binding(idx) => {
                            if let Some(app_binding) = whkdrc.app_bindings.get_mut(*idx) {
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
            let commands_col = self.new_binding.1.iter().enumerate().fold(column![].spacing(10), |col, (binding_idx, binding)| {
                let process = opt_helpers::input_with_disable_default(
                    "Process Name",
                    Some("Name of the process on which the following command will run when this key combination is pressed."),
                    "",
                    binding.process_name.as_ref().map(String::as_str).unwrap_or(""),
                    String::new(),
                    move |v| Message::ChangeNewBindingProcess(binding_idx, v),
                    None,
                    Some(DisableArgs::new(
                        binding.process_name.as_ref().is_some_and(|p| p == "Default"),
                        Some("Default"),
                        move |v| {
                            let value = if v {
                                String::from("Default")
                            } else {
                                String::new()
                            };
                            Message::ChangeNewBindingProcess(binding_idx, value)
                        },
                    )),
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
                );
                col.push(opt_helpers::opt_box(column![process, command].spacing(10)))
            });
            let command = command_edit(
                Some(&self.new_binding_state[0]),
                &self.new_binding_content[0],
                &self.new_binding.1[0].command,
                commands,
                commands_desc,
                theme,
                |s| Message::ChangeNewBindingCommand(0, s),
                |a| Message::ChangeNewBindingContent(0, a),
            );

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
                .width(iced::Fill)
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
                    let command = command_edit(
                        self.editing_states.get(&idx).and_then(|es| es.get(&0)),
                        self.editing_commands
                            .get(&idx)
                            .expect("should have editing content")
                            .get(&0)
                            .expect("should have editing content"),
                        &app_binding.1[0].command,
                        commands,
                        commands_desc,
                        theme,
                        move |c| Message::ChangeBindingCommand(idx, 0, c),
                        move |a| Message::ChangeBindingContent(idx, 0, a),
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
            opt_helpers::section_view("App Bindings:", [add_new_binding_button, new_binding]),
            opt_helpers::section_view_id("App Bindings:", SCROLLABLE_ID, [bindings]),
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
                if let Some(app_binding) = whkdrc.app_bindings.get(*idx) {
                    let bindings_count = app_binding.1.len();
                    for i in 0..bindings_count {
                        let content = if let Some(binding) = app_binding.1.get(i) {
                            text_editor::Content::with_text(&binding.command)
                        } else {
                            text_editor::Content::new()
                        };
                        self.editing_commands
                            .entry(*idx)
                            .or_insert(HashMap::new())
                            .insert(i, content);
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
        .push(widget::opt_helpers::disable_checkbox(
            DisableArgs::none().as_ref(),
        ))
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
