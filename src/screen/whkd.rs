use crate::{
    whkd::{HotkeyBinding, Shell, Whkdrc, DEFAULT_WHKDRC},
    widget::opt_helpers,
};

use iced::{Element, Task};

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Shell(Shell),
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
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug, Default)]
pub struct Whkd;

impl Whkd {
    pub fn update(&mut self, message: Message, whkdrc: &mut Whkdrc) -> (Action, Task<Message>) {
        match message {
            Message::Shell(shell) => whkdrc.shell = shell,
            _ => {},
        }
        (Action::None, Task::none())
    }

    pub fn view(&self, whkdrc: &Whkdrc) -> Element<Message> {
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

        opt_helpers::section_view(
            "Whkd:",
            [shell]
        )
    }
}

