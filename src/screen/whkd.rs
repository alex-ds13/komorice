pub mod bindings;

pub use bindings::Bindings;

use crate::{
    whkd::{DEFAULT_WHKDRC, Shell, WhkdBinary, Whkdrc},
    widget::{self, hover, icons, opt_helpers},
};

use std::collections::HashMap;

use iced::{
    Element, Subscription, Task, Theme, keyboard, padding,
    widget::{bottom_center, column, combo_box, container, markdown, pick_list, row, text},
};
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::{GetKeyboardLayout, VkKeyScanExW},
    WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

static MODIFIERS: [&str; 4] = ["CTRL", "SHIFT", "ALT", "WIN"];

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Shell(Shell),
    PBMod(usize, String),
    PBKey(String),
    PauseBinding(Option<Vec<String>>),
    PauseHook(Option<String>),
    // AppBindings(Vec<(Vec<String>, Vec<HotkeyBinding>)>),
    // AddNewAppBinding,
    // RemoveAppBinding(usize),
    // ChangeAppBindingKeys(usize, Vec<String>),
    // ChangeAppBindingProcessName(usize, String),
    // ChangeAppBindingCommand(usize, String),
    KeyPress(String, String),
    KeyRelease(String, String),
    Navigate(NavMessage),
    UrlClicked(markdown::Url),
    ToggleWhkd,
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    ToggleWhkd,
}

#[derive(Debug, Default)]
pub struct Whkd {
    pressed_keys: Vec<String>,
    pressed_keys_temp: Vec<String>,
    pressed_mod: String,
    pause_hook_state: iced::widget::combo_box::State<String>,
}

impl Whkd {
    pub fn update(&mut self, message: Message, whkdrc: &mut Whkdrc) -> (Action, Task<Message>) {
        match message {
            Message::Shell(shell) => whkdrc.shell = shell,
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
                whkdrc.pause_hook = pause_hook;
            }
            Message::Navigate(nav) => match nav {
                NavMessage::Forward => {}
                NavMessage::Back => {}
            },
            Message::UrlClicked(url) => {
                println!("Clicked url: {}", url);
            }
            Message::ToggleWhkd => {
                return (Action::ToggleWhkd, Task::none());
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
        let pause_hook = hook_custom(
            &self.pause_hook_state,
            &whkdrc.pause_hook,
            commands,
            commands_desc,
            theme,
        );

        let mut key_pressed = row![text("PRESSED: "), text!("{}", self.pressed_mod),];

        key_pressed = key_pressed.push(
            (!self.pressed_mod.is_empty() && !self.pressed_keys.is_empty()).then_some(text(" + ")),
        );
        key_pressed = key_pressed.push(text!(
            "{}",
            self.pressed_keys.iter().fold(String::new(), |mut str, k| {
                if !str.is_empty() {
                    str.push_str(" + ");
                }
                str.push_str(k);
                str
            })
        ));

        let toggle_whkd_but = if whkd_bin.found {
            iced::widget::button(if whkd_bin.is_running() {
                "Stop Whkd"
            } else {
                "Start Whkd"
            })
            .on_press(Message::ToggleWhkd)
            .into()
        } else {
            iced::widget::space().into()
        };

        opt_helpers::section_view(
            "Whkd:",
            [
                shell,
                pause_binding,
                pause_hook,
                key_pressed.into(),
                toggle_whkd_but,
            ],
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = iced::event::listen_with(|event, status, _id| {
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
                        println!("Physical Pressed: {physical_key:#?}");
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
        });
        // let press = keyboard::on_key_press(|k, m| {
        //     let (k, m) = get_vk_key_mods(k, m);
        //     if !k.is_empty() {
        //         Some(Message::KeyPress(k, m))
        //     } else {
        //         None
        //     }
        // });
        // let release = keyboard::on_key_release(|k, m| {
        //     let (k, m) = get_vk_key_mods(k, m);
        //     if !k.is_empty() {
        //         Some(Message::KeyRelease(k, m))
        //     } else {
        //         None
        //     }
        // });
        let navigation = navigation_sub().map(Message::Navigate);

        Subscription::batch([
            // press,
            // release,
            navigation,
            keys,
        ])
    }

    pub fn load_new_commands(&mut self, commands: &[String]) {
        self.pause_hook_state = combo_box::State::new(commands.to_vec());
    }
}

fn get_vk_key_mods(
    key: keyboard::key::Key,
    physical: keyboard::key::Physical,
    location: keyboard::Location,
    modifiers: keyboard::Modifiers,
) -> (String, String) {
    use keyboard::key::{Code, Key, Named, Physical};

    /// Return the first codepoint of a string.
    ///
    /// # Panics
    /// Panics if the string is empty.
    fn first_char(s: &str) -> char {
        s.chars().next().expect("empty string")
    }

    /// Determine a *keyCode* value for a key.
    ///
    /// The *keyCode* is an implementation specific legacy property of DOM keyboard events.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-keyboardevent-keycode>
    pub fn keycode(key: Key) -> Option<win_hotkeys::VKey> {
        use win_hotkeys::VKey;
        let vkey = match key {
            Key::Named(named) => match named {
                Named::Backspace => VKey::Back,
                Named::Tab => VKey::Tab,
                Named::Enter => VKey::Return,
                Named::Shift => VKey::Shift,
                Named::Control => VKey::Control,
                Named::Alt => VKey::Menu,
                Named::CapsLock => VKey::Capital,
                Named::Escape => VKey::Escape,
                Named::PageUp => VKey::Prior,
                Named::PageDown => VKey::Next,
                Named::End => VKey::End,
                Named::Home => VKey::Home,
                Named::ArrowLeft => VKey::Left,
                Named::ArrowUp => VKey::Up,
                Named::ArrowRight => VKey::Right,
                Named::ArrowDown => VKey::Down,
                Named::Delete => VKey::Delete,
                Named::Insert => VKey::Insert,
                Named::AltGraph => VKey::RMenu,
                Named::NumLock => VKey::Numlock,
                Named::ScrollLock => VKey::Scroll,
                Named::Meta => VKey::LWin,
                Named::Hyper => VKey::LWin,
                Named::Super => VKey::LWin,
                Named::Space => VKey::Space,
                Named::Clear => VKey::Clear,
                Named::CrSel => VKey::Crsel,
                Named::ExSel => VKey::Exsel,
                Named::Attn => VKey::Attn,
                Named::ContextMenu => VKey::Apps,
                Named::Execute => VKey::Execute,
                Named::Help => VKey::Help,
                Named::Pause => VKey::Pause,
                Named::Play => VKey::Play,
                Named::Select => VKey::Select,
                Named::ZoomIn => VKey::Zoom,
                Named::ZoomOut => VKey::Zoom,
                Named::PrintScreen => VKey::Snapshot,
                Named::Convert => VKey::CustomKeyCode(0x1C),
                Named::ModeChange => VKey::CustomKeyCode(0x1F),
                Named::NonConvert => VKey::CustomKeyCode(0x1D),
                Named::Process => VKey::CustomKeyCode(0xE5),
                Named::HangulMode => VKey::CustomKeyCode(0x15),
                Named::HanjaMode => VKey::CustomKeyCode(0x19),
                Named::JunjaMode => VKey::CustomKeyCode(0x17),
                Named::KanaMode => VKey::CustomKeyCode(0x15),
                Named::KanjiMode => VKey::CustomKeyCode(0x19),
                Named::MediaPlayPause => VKey::MediaPlayPause,
                Named::MediaStop => VKey::MediaStop,
                Named::MediaTrackNext => VKey::MediaPrevTrack,
                Named::MediaTrackPrevious => VKey::MediaNextTrack,
                Named::Print => VKey::Print,
                Named::AudioVolumeDown => VKey::VolumeDown,
                Named::AudioVolumeUp => VKey::VolumeUp,
                Named::AudioVolumeMute => VKey::VolumeMute,
                Named::LaunchApplication1 => VKey::LaunchApp1,
                Named::LaunchApplication2 => VKey::LaunchApp1,
                Named::LaunchMail => VKey::LaunchMail,
                Named::LaunchMediaPlayer => VKey::LaunchMediaSelect,
                Named::BrowserBack => VKey::BrowserBack,
                Named::BrowserFavorites => VKey::BrowserFavorites,
                Named::BrowserForward => VKey::BrowserForward,
                Named::BrowserHome => VKey::BrowserHome,
                Named::BrowserRefresh => VKey::BrowserRefresh,
                Named::BrowserSearch => VKey::BrowserSearch,
                Named::BrowserStop => VKey::BrowserStop,
                Named::ZoomToggle => VKey::Zoom,
                Named::F1 => VKey::F1,
                Named::F2 => VKey::F2,
                Named::F3 => VKey::F3,
                Named::F4 => VKey::F4,
                Named::F5 => VKey::F5,
                Named::F6 => VKey::F6,
                Named::F7 => VKey::F7,
                Named::F8 => VKey::F8,
                Named::F9 => VKey::F9,
                Named::F10 => VKey::F10,
                Named::F11 => VKey::F11,
                Named::F12 => VKey::F12,
                Named::F13 => VKey::F13,
                Named::F14 => VKey::F14,
                Named::F15 => VKey::F15,
                Named::F16 => VKey::F16,
                Named::F17 => VKey::F17,
                Named::F18 => VKey::F18,
                Named::F19 => VKey::F19,
                Named::F20 => VKey::F20,
                Named::F21 => VKey::F21,
                Named::F22 => VKey::F22,
                Named::F23 => VKey::F23,
                Named::F24 => VKey::F24,
                _ => return None,
            },
            Key::Unidentified => return None,
            Key::Character(ref c) => {
                let x = c.encode_utf16();
                let count = x.count();
                println!("Count: {}", count);
                let mut x = c.encode_utf16();
                if count == 1
                    && let Some(x) = x.next()
                {
                    let current_window_thread_id = unsafe {
                        GetWindowThreadProcessId(GetForegroundWindow(), std::ptr::null_mut())
                    };
                    let locale_id = unsafe { GetKeyboardLayout(current_window_thread_id) };
                    let res = unsafe { VkKeyScanExW(x as _, locale_id) };
                    let vk = res & 0x00FF;
                    let m_state = res >> 8;
                    println!("m_state: {m_state}");
                    VKey::from_vk_code(vk as u16)
                } else {
                    println!("Key: {key:?}");
                    return None;
                }
            }
        };
        Some(vkey)
    }

    fn physical_to_vkey(key: Physical) -> win_hotkeys::VKey {
        use win_hotkeys::VKey;
        let current_window_thread_id = unsafe { GetWindowThreadProcessId(GetForegroundWindow(), std::ptr::null_mut()) };
        let locale_id = unsafe { GetKeyboardLayout(current_window_thread_id) } as u64;
        let lang_id = locale_id & 0xFFFF;
        let is_brazil = lang_id == 0x416;
        match key {
            Physical::Code(code) => match code {
                Code::Backquote => todo!(),
                Code::Backslash => todo!(),
                Code::BracketLeft => todo!(),
                Code::BracketRight => todo!(),
                Code::Comma => todo!(),
                Code::Digit0 => todo!(),
                Code::Digit1 => todo!(),
                Code::Digit2 => todo!(),
                Code::Digit3 => todo!(),
                Code::Digit4 => todo!(),
                Code::Digit5 => todo!(),
                Code::Digit6 => todo!(),
                Code::Digit7 => todo!(),
                Code::Digit8 => todo!(),
                Code::Digit9 => todo!(),
                Code::Equal => todo!(),
                Code::IntlBackslash => todo!(),
                Code::IntlRo => todo!(),
                Code::IntlYen => todo!(),
                Code::KeyA => todo!(),
                Code::KeyB => todo!(),
                Code::KeyC => todo!(),
                Code::KeyD => todo!(),
                Code::KeyE => todo!(),
                Code::KeyF => todo!(),
                Code::KeyG => todo!(),
                Code::KeyH => todo!(),
                Code::KeyI => todo!(),
                Code::KeyJ => todo!(),
                Code::KeyK => todo!(),
                Code::KeyL => todo!(),
                Code::KeyM => todo!(),
                Code::KeyN => todo!(),
                Code::KeyO => todo!(),
                Code::KeyP => todo!(),
                Code::KeyQ => todo!(),
                Code::KeyR => todo!(),
                Code::KeyS => todo!(),
                Code::KeyT => todo!(),
                Code::KeyU => todo!(),
                Code::KeyV => todo!(),
                Code::KeyW => todo!(),
                Code::KeyX => todo!(),
                Code::KeyY => todo!(),
                Code::KeyZ => todo!(),
                Code::Minus => todo!(),
                Code::Period => todo!(),
                Code::Quote => todo!(),
                Code::Semicolon => todo!(),
                Code::Slash => todo!(),
                Code::AltLeft => todo!(),
                Code::AltRight => todo!(),
                Code::Backspace => todo!(),
                Code::CapsLock => todo!(),
                Code::ContextMenu => todo!(),
                Code::ControlLeft => todo!(),
                Code::ControlRight => todo!(),
                Code::Enter => todo!(),
                Code::SuperLeft => todo!(),
                Code::SuperRight => todo!(),
                Code::ShiftLeft => todo!(),
                Code::ShiftRight => todo!(),
                Code::Space => todo!(),
                Code::Tab => todo!(),
                Code::Convert => todo!(),
                Code::KanaMode => todo!(),
                Code::Lang1 => todo!(),
                Code::Lang2 => todo!(),
                Code::Lang3 => todo!(),
                Code::Lang4 => todo!(),
                Code::Lang5 => todo!(),
                Code::NonConvert => todo!(),
                Code::Delete => todo!(),
                Code::End => todo!(),
                Code::Help => todo!(),
                Code::Home => todo!(),
                Code::Insert => todo!(),
                Code::PageDown => todo!(),
                Code::PageUp => todo!(),
                Code::ArrowDown => todo!(),
                Code::ArrowLeft => todo!(),
                Code::ArrowRight => todo!(),
                Code::ArrowUp => todo!(),
                Code::NumLock => VKey::Numlock,
                Code::Numpad0 => VKey::Numpad0,
                Code::Numpad1 => VKey::Numpad1,
                Code::Numpad2 => VKey::Numpad2,
                Code::Numpad3 => VKey::Numpad3,
                Code::Numpad4 => VKey::Numpad4,
                Code::Numpad5 => VKey::Numpad5,
                Code::Numpad6 => VKey::Numpad6,
                Code::Numpad7 => VKey::Numpad7,
                Code::Numpad8 => VKey::Numpad8,
                Code::Numpad9 => VKey::Numpad9,
                Code::NumpadAdd => VKey::Add,
                Code::NumpadBackspace => VKey::Back,
                Code::NumpadClear => VKey::Clear,
                Code::NumpadClearEntry => VKey::Clear,
                Code::NumpadComma => if is_brazil { VKey::CustomKeyCode(0xC2) } else { VKey::Separator },
                Code::NumpadDecimal => VKey::Decimal,
                Code::NumpadDivide => VKey::Divide,
                Code::NumpadEnter => VKey::Return,
                Code::NumpadEqual => VKey::Return,
                Code::NumpadHash => todo!(),
                Code::NumpadMemoryAdd => todo!(),
                Code::NumpadMemoryClear => todo!(),
                Code::NumpadMemoryRecall => todo!(),
                Code::NumpadMemoryStore => todo!(),
                Code::NumpadMemorySubtract => todo!(),
                Code::NumpadMultiply => VKey::Multiply,
                Code::NumpadParenLeft => todo!(),
                Code::NumpadParenRight => todo!(),
                Code::NumpadStar => VKey::Multiply,
                Code::NumpadSubtract => VKey::Subtract,
                Code::Escape => todo!(),
                Code::Fn => todo!(),
                Code::FnLock => todo!(),
                Code::PrintScreen => todo!(),
                Code::ScrollLock => todo!(),
                Code::Pause => todo!(),
                Code::BrowserBack => todo!(),
                Code::BrowserFavorites => todo!(),
                Code::BrowserForward => todo!(),
                Code::BrowserHome => todo!(),
                Code::BrowserRefresh => todo!(),
                Code::BrowserSearch => todo!(),
                Code::BrowserStop => todo!(),
                Code::Eject => todo!(),
                Code::LaunchApp1 => todo!(),
                Code::LaunchApp2 => todo!(),
                Code::LaunchMail => todo!(),
                Code::MediaPlayPause => todo!(),
                Code::MediaSelect => todo!(),
                Code::MediaStop => todo!(),
                Code::MediaTrackNext => todo!(),
                Code::MediaTrackPrevious => todo!(),
                Code::Power => todo!(),
                Code::Sleep => todo!(),
                Code::AudioVolumeDown => todo!(),
                Code::AudioVolumeMute => todo!(),
                Code::AudioVolumeUp => todo!(),
                Code::WakeUp => todo!(),
                Code::Meta => todo!(),
                Code::Hyper => todo!(),
                Code::Turbo => todo!(),
                Code::Abort => todo!(),
                Code::Resume => todo!(),
                Code::Suspend => todo!(),
                Code::Again => todo!(),
                Code::Copy => todo!(),
                Code::Cut => todo!(),
                Code::Find => todo!(),
                Code::Open => todo!(),
                Code::Paste => todo!(),
                Code::Props => todo!(),
                Code::Select => todo!(),
                Code::Undo => todo!(),
                Code::Hiragana => todo!(),
                Code::Katakana => todo!(),
                Code::F1 => todo!(),
                Code::F2 => todo!(),
                Code::F3 => todo!(),
                Code::F4 => todo!(),
                Code::F5 => todo!(),
                Code::F6 => todo!(),
                Code::F7 => todo!(),
                Code::F8 => todo!(),
                Code::F9 => todo!(),
                Code::F10 => todo!(),
                Code::F11 => todo!(),
                Code::F12 => todo!(),
                Code::F13 => todo!(),
                Code::F14 => todo!(),
                Code::F15 => todo!(),
                Code::F16 => todo!(),
                Code::F17 => todo!(),
                Code::F18 => todo!(),
                Code::F19 => todo!(),
                Code::F20 => todo!(),
                Code::F21 => todo!(),
                Code::F22 => todo!(),
                Code::F23 => todo!(),
                Code::F24 => todo!(),
                Code::F25 => todo!(),
                Code::F26 => todo!(),
                Code::F27 => todo!(),
                Code::F28 => todo!(),
                Code::F29 => todo!(),
                Code::F30 => todo!(),
                Code::F31 => todo!(),
                Code::F32 => todo!(),
                Code::F33 => todo!(),
                Code::F34 => todo!(),
                Code::F35 => todo!(),
                _ => todo!(),
            },
            Physical::Unidentified(_native_code) => todo!(),
        }
    }

    let k = match key {
        Key::Named(named) => match named {
            Named::Alt
            | Named::AltGraph
            | Named::Shift
            | Named::Control
            | Named::Meta
            | Named::Hyper
            | Named::Super => String::new(),
            _ => {
                if location == keyboard::Location::Numpad {
                    physical_to_vkey(physical).to_string()
                } else {
                    keycode(key).map(|k| k.to_string()).unwrap_or_default()
                }
            }
        },
        Key::Character(_) => {
            if location == keyboard::Location::Numpad {
                physical_to_vkey(physical).to_string()
            } else {
                keycode(key).map(|k| k.to_string()).unwrap_or_default()
            }
        }
        Key::Unidentified => String::new(),
    };
    let k = k.trim_start_matches("VK_")
            .trim_start_matches("Custom(")
            .trim_end_matches(")")
            .to_lowercase();
    // let k = keycode(key)
    //     .map(|k| k.to_string())
    //     .unwrap_or_default()
    //     .trim_start_matches("VK_")
    //     .to_lowercase();
    let m = modifiers
        .iter_names()
        .fold(String::new(), |mut s, (n, _m)| {
            if !s.is_empty() {
                s.push_str(" + ");
            }
            if n.to_lowercase() == "logo" {
                s.push_str("win");
            } else {
                s.push_str(n);
            }
            s
        });
    (k, m)
}

fn get_key_mods(key: keyboard::Key, modifiers: keyboard::Modifiers) -> (String, String) {
    use keyboard::key::{Key, Named};

    let k = match key {
        Key::Named(named) => match named {
            Named::Alt
            | Named::AltGraph
            | Named::Shift
            | Named::Control
            | Named::Meta
            | Named::Hyper
            | Named::Super => String::new(),
            _ => format!("{:?}", named),
        },
        Key::Character(c) => format!("{}", c),
        Key::Unidentified => String::new(),
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
    (k, m)
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
    row![
        mod_choose(sb.modifiers, 3),
        mod_choose(sb.modifiers, 2),
        mod_choose(sb.modifiers, 1),
        mod_choose(sb.modifiers, 0),
        key,
    ]
    .spacing(5)
    .into()
}

fn hook_custom<'a>(
    state: &'a combo_box::State<String>,
    pause_hook: &'a Option<String>,
    commands: &'a [String],
    commands_desc: &'a HashMap<String, Vec<markdown::Item>>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let (hook_command, hook_custom) = split_pause_hook(pause_hook, commands);
    let is_dirty = pause_hook != &DEFAULT_WHKDRC.pause_hook;
    widget::expandable::expandable(move |hovered, expanded| {
        let label = if is_dirty {
            row![text("Command")]
                .push(widget::opt_helpers::reset_button(Some(Message::PauseHook(
                    DEFAULT_WHKDRC.pause_hook.clone(),
                ))))
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
            let ph = pause_hook.as_ref().map_or("", String::as_str);
            let custom = widget::input("", ph, |v| Message::PauseHook(Some(v)), None);
            container(
                column![
                    row!["Komorebic commands:", commands_box].spacing(5),
                    "Command:",
                    custom,
                ]
                .max_width(700)
                .padding(padding::bottom(10))
                .spacing(10),
            )
            .align_right(iced::FillPortion(3))
        })
        .spacing(10);

        let top = commands_desc
            .get(hook_command.strip_prefix("komorebic ").unwrap_or_default())
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
            commands_desc.get(hook_command.strip_prefix("komorebic ").unwrap_or_default())
        {
            vec![markdown(items, theme).map(Message::UrlClicked)]
        } else {
            vec![]
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
