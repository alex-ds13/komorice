use super::{SEPARATOR, WhkdBinary};

use crate::BOLD_FONT;
use crate::widget::modal;

use iced::widget::{button, column, container, rich_text, row, span, text};
use iced::{Center, Color, Element, Fill, Shrink, keyboard, padding};
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::{GetKeyboardLayout, VkKeyScanExW},
    WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

pub fn keybind_modal<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    show: bool,
    modifiers: &'a String,
    keys: &'a [String],
    whkd_bin: &'a WhkdBinary,
    on_close: impl Fn(bool) -> Message + Clone,
) -> Element<'a, Message> {
    modal(
        content.into(),
        show.then_some(modal_content(modifiers, keys, whkd_bin, on_close.clone())),
        on_close(false),
    )
}

pub fn modal_content<'a, Message: Clone + 'a>(
    modifiers: &'a String,
    keys: &'a [String],
    whkd_bin: &'a WhkdBinary,
    on_close: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    let whkd_warnings = whkd_bin_warnings(whkd_bin);
    let has_whkd_warnings = whkd_warnings.is_some();
    container(
        column![
            "Press some key to bind:",
            {
                let key_pressed = text!(
                    "{:^10}",
                    keys.iter().fold(modifiers.clone(), |mut str, k| {
                        if !str.is_empty() && !k.is_empty() {
                            str.push_str(&SEPARATOR);
                        }
                        str.push_str(k);
                        str
                    })
                )
                .font(iced::font::Font::MONOSPACE);

                container(key_pressed)
                    .center_x(Shrink)
                    .padding(padding::left(10).right(10))
                    .style(|t: &iced::Theme| {
                        let palette = t.extended_palette();
                        container::Style {
                            background: Some(palette.background.weaker.color.into()),
                            ..container::transparent(t)
                        }
                    })
            },
            row![
                button("Save").on_press(on_close(true)),
                button("Cancel")
                    .style(button::secondary)
                    .on_press(on_close(false))
            ]
            .padding(if has_whkd_warnings {
                padding::left(150).right(150).bottom(20)
            } else {
                padding::all(0)
            })
            .spacing(10),
            whkd_warnings,
        ]
        .width(Shrink)
        .align_x(Center)
        .spacing(10),
    )
    .padding(if has_whkd_warnings { 20 } else { 50 })
    .style(container::bordered_box)
    .into()
}

fn whkd_bin_warnings<'a, Message: 'a>(whkd_bin: &'a WhkdBinary) -> Option<Element<'a, Message>> {
    let normal_warning = (whkd_bin.found && whkd_bin.running_initial && !whkd_bin.running_current)
        .then_some(
            rich_text![
                span("WARNING: ")
                    .font(*BOLD_FONT)
                    .color(*crate::widget::YELLOW),
                span("Whkd was running! "),
                span("Komorice stopped it temporarily so that you can setup your keybind. "),
                span("Once this modal is closed it will restart it again."),
            ]
            .size(12)
            .width(Fill)
            .on_link_click(iced::never),
        );
    let normal_warning_temp =
        (whkd_bin.found && whkd_bin.running_initial && whkd_bin.running_current).then_some(
            rich_text![
                span("WARNING: ")
                    .font(*BOLD_FONT)
                    .color(*crate::widget::YELLOW),
                span("Whkd is running! "),
                span("Komorice stopped it temporarily so that you can setup your keybind.  ")
                    .color(Color::TRANSPARENT),
                span("Once this modal is closed it will restart it again. ")
                    .color(Color::TRANSPARENT),
            ]
            .size(12)
            .width(Fill)
            .on_link_click(iced::never),
        );
    let unable_to_toggle_warning = (!whkd_bin.found && whkd_bin.running_initial).then_some(
        rich_text![
            span("WARNING: ")
                .font(*BOLD_FONT)
                .color(*crate::widget::YELLOW),
            span("Whkd is running! "),
            span(
                "However Komorice failed to find the 'whkd.exe' binary \
                on path, so it can't stop it and restart it for you! This means that you \
                can't use this modal to bind to a key that will be captured by 'whkd'.\n\n\
                You need to either stop 'whkd' yourself manually or make sure you add it \
                on path and restart Komorice so that it can do it for you."
            )
        ]
        .size(12)
        .width(Fill)
        .on_link_click(iced::never),
    );
    // let failed_to_stop_warning =
    //     (whkd_bin.found && whkd_bin.running_initial && whkd_bin.running_current).then_some(
    //         rich_text![
    //             span("WARNING: ")
    //                 .font(*BOLD_FONT)
    //                 .color(*crate::widget::YELLOW),
    //             span(
    //                 "Whkd is running! Komorice tried to stop it temporarily so that you can setup \
    //             your keybind, but failed to do so."
    //             )
    //         ]
    //         .size(12)
    //         .width(Fill)
    //         .on_link_click(iced::never),
    //     );

    normal_warning
        .or(normal_warning_temp)
        .or(unable_to_toggle_warning)
        // .or(failed_to_stop_warning)
        .map(Into::into)
}

pub fn get_vk_key_mods(
    key: keyboard::key::Key,
    physical: keyboard::key::Physical,
    location: keyboard::Location,
    modifiers: keyboard::Modifiers,
) -> (String, String) {
    use keyboard::key::{Code, Key, Named, Physical};

    /// Converts a [`Key`] into a potential [`win_hotkeys::VKey`]
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
                // println!("Count: {}", count);
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
                    // let m_state = res >> 8;
                    // println!("m_state: {m_state}");
                    VKey::from_vk_code(vk as u16)
                } else {
                    // println!("Key: {key:?}");
                    return None;
                }
            }
        };
        Some(vkey)
    }

    /// Converts a [`Physical`] into a potential [`win_hotkeys::VKey`]
    fn physical_to_vkey(key: Physical) -> Option<win_hotkeys::VKey> {
        use win_hotkeys::VKey;
        let current_window_thread_id =
            unsafe { GetWindowThreadProcessId(GetForegroundWindow(), std::ptr::null_mut()) };
        let locale_id = unsafe { GetKeyboardLayout(current_window_thread_id) } as u64;
        let lang_id = locale_id & 0xFFFF;
        let is_brazil = lang_id == 0x416;
        match key {
            Physical::Code(code) => match code {
                Code::Backquote => None,
                Code::Backslash => None,
                Code::BracketLeft => None,
                Code::BracketRight => None,
                Code::Comma => None,
                Code::Digit0 => None,
                Code::Digit1 => None,
                Code::Digit2 => None,
                Code::Digit3 => None,
                Code::Digit4 => None,
                Code::Digit5 => None,
                Code::Digit6 => None,
                Code::Digit7 => None,
                Code::Digit8 => None,
                Code::Digit9 => None,
                Code::Equal => None,
                Code::IntlBackslash => None,
                Code::IntlRo => None,
                Code::IntlYen => None,
                Code::KeyA => None,
                Code::KeyB => None,
                Code::KeyC => None,
                Code::KeyD => None,
                Code::KeyE => None,
                Code::KeyF => None,
                Code::KeyG => None,
                Code::KeyH => None,
                Code::KeyI => None,
                Code::KeyJ => None,
                Code::KeyK => None,
                Code::KeyL => None,
                Code::KeyM => None,
                Code::KeyN => None,
                Code::KeyO => None,
                Code::KeyP => None,
                Code::KeyQ => None,
                Code::KeyR => None,
                Code::KeyS => None,
                Code::KeyT => None,
                Code::KeyU => None,
                Code::KeyV => None,
                Code::KeyW => None,
                Code::KeyX => None,
                Code::KeyY => None,
                Code::KeyZ => None,
                Code::Minus => None,
                Code::Period => None,
                Code::Quote => None,
                Code::Semicolon => None,
                Code::Slash => None,
                Code::AltLeft => None,
                Code::AltRight => None,
                Code::Backspace => None,
                Code::CapsLock => None,
                Code::ContextMenu => None,
                Code::ControlLeft => None,
                Code::ControlRight => None,
                Code::Enter => None,
                Code::SuperLeft => None,
                Code::SuperRight => None,
                Code::ShiftLeft => None,
                Code::ShiftRight => None,
                Code::Space => None,
                Code::Tab => None,
                Code::Convert => None,
                Code::KanaMode => None,
                Code::Lang1 => None,
                Code::Lang2 => None,
                Code::Lang3 => None,
                Code::Lang4 => None,
                Code::Lang5 => None,
                Code::NonConvert => None,
                Code::Delete => None,
                Code::End => None,
                Code::Help => None,
                Code::Home => None,
                Code::Insert => None,
                Code::PageDown => None,
                Code::PageUp => None,
                Code::ArrowDown => None,
                Code::ArrowLeft => None,
                Code::ArrowRight => None,
                Code::ArrowUp => None,
                Code::NumLock => Some(VKey::Numlock),
                Code::Numpad0 => Some(VKey::Numpad0),
                Code::Numpad1 => Some(VKey::Numpad1),
                Code::Numpad2 => Some(VKey::Numpad2),
                Code::Numpad3 => Some(VKey::Numpad3),
                Code::Numpad4 => Some(VKey::Numpad4),
                Code::Numpad5 => Some(VKey::Numpad5),
                Code::Numpad6 => Some(VKey::Numpad6),
                Code::Numpad7 => Some(VKey::Numpad7),
                Code::Numpad8 => Some(VKey::Numpad8),
                Code::Numpad9 => Some(VKey::Numpad9),
                Code::NumpadAdd => Some(VKey::Add),
                Code::NumpadBackspace => Some(VKey::Back),
                Code::NumpadClear => Some(VKey::Clear),
                Code::NumpadClearEntry => Some(VKey::Clear),
                Code::NumpadComma => {
                    if is_brazil {
                        Some(VKey::CustomKeyCode(0xC2))
                    } else {
                        Some(VKey::Separator)
                    }
                }
                Code::NumpadDecimal => Some(VKey::Decimal),
                Code::NumpadDivide => Some(VKey::Divide),
                Code::NumpadEnter => Some(VKey::Return),
                Code::NumpadEqual => Some(VKey::Return),
                Code::NumpadHash => None,
                Code::NumpadMemoryAdd => None,
                Code::NumpadMemoryClear => None,
                Code::NumpadMemoryRecall => None,
                Code::NumpadMemoryStore => None,
                Code::NumpadMemorySubtract => None,
                Code::NumpadMultiply => Some(VKey::Multiply),
                Code::NumpadParenLeft => None,
                Code::NumpadParenRight => None,
                Code::NumpadStar => Some(VKey::Multiply),
                Code::NumpadSubtract => Some(VKey::Subtract),
                Code::Escape => None,
                Code::Fn => None,
                Code::FnLock => None,
                Code::PrintScreen => None,
                Code::ScrollLock => None,
                Code::Pause => None,
                Code::BrowserBack => None,
                Code::BrowserFavorites => None,
                Code::BrowserForward => None,
                Code::BrowserHome => None,
                Code::BrowserRefresh => None,
                Code::BrowserSearch => None,
                Code::BrowserStop => None,
                Code::Eject => None,
                Code::LaunchApp1 => None,
                Code::LaunchApp2 => None,
                Code::LaunchMail => None,
                Code::MediaPlayPause => None,
                Code::MediaSelect => None,
                Code::MediaStop => None,
                Code::MediaTrackNext => None,
                Code::MediaTrackPrevious => None,
                Code::Power => None,
                Code::Sleep => None,
                Code::AudioVolumeDown => None,
                Code::AudioVolumeMute => None,
                Code::AudioVolumeUp => None,
                Code::WakeUp => None,
                Code::Meta => None,
                Code::Hyper => None,
                Code::Turbo => None,
                Code::Abort => None,
                Code::Resume => None,
                Code::Suspend => None,
                Code::Again => None,
                Code::Copy => None,
                Code::Cut => None,
                Code::Find => None,
                Code::Open => None,
                Code::Paste => None,
                Code::Props => None,
                Code::Select => None,
                Code::Undo => None,
                Code::Hiragana => None,
                Code::Katakana => None,
                Code::F1 => None,
                Code::F2 => None,
                Code::F3 => None,
                Code::F4 => None,
                Code::F5 => None,
                Code::F6 => None,
                Code::F7 => None,
                Code::F8 => None,
                Code::F9 => None,
                Code::F10 => None,
                Code::F11 => None,
                Code::F12 => None,
                Code::F13 => None,
                Code::F14 => None,
                Code::F15 => None,
                Code::F16 => None,
                Code::F17 => None,
                Code::F18 => None,
                Code::F19 => None,
                Code::F20 => None,
                Code::F21 => None,
                Code::F22 => None,
                Code::F23 => None,
                Code::F24 => None,
                Code::F25 => None,
                Code::F26 => None,
                Code::F27 => None,
                Code::F28 => None,
                Code::F29 => None,
                Code::F30 => None,
                Code::F31 => None,
                Code::F32 => None,
                Code::F33 => None,
                Code::F34 => None,
                Code::F35 => None,
                _ => None,
            },
            Physical::Unidentified(_native_code) => match _native_code {
                keyboard::key::NativeCode::Unidentified => None,
                keyboard::key::NativeCode::Android(_) => None,
                keyboard::key::NativeCode::MacOS(_) => None,
                keyboard::key::NativeCode::Windows(vk) => Some(VKey::CustomKeyCode(vk)),
                keyboard::key::NativeCode::Xkb(_) => None,
            },
        }
    }

    let k = match key {
        Key::Named(named) => match named {
            Named::Alt
            | Named::Shift
            | Named::Control
            | Named::Meta
            | Named::Hyper
            | Named::Super => String::new(),
            _ => {
                if location == keyboard::Location::Numpad {
                    physical_to_vkey(physical)
                        .map(|k| k.to_string())
                        .unwrap_or_default()
                } else {
                    keycode(key).map(|k| k.to_string()).unwrap_or_default()
                }
            }
        },
        Key::Character(_) => {
            if location == keyboard::Location::Numpad {
                physical_to_vkey(physical)
                    .map(|k| k.to_string())
                    .unwrap_or_default()
            } else {
                keycode(key).map(|k| k.to_string()).unwrap_or_default()
            }
        }
        Key::Unidentified => String::new(),
    };
    let k = k
        .trim_start_matches("VK_")
        .trim_start_matches("Custom(")
        .trim_end_matches(")")
        .to_lowercase();
    let m = modifiers
        .iter_names()
        .fold(String::new(), |mut s, (n, _m)| {
            if !s.is_empty() {
                s.push_str(" + ");
            }
            if n.to_lowercase() == "logo" {
                s.push_str("win");
            } else {
                s.push_str(&n.to_lowercase());
            }
            s
        });
    (k, m)
}
