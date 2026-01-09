use std::path::PathBuf;

use crate::{
    screen::View,
    widget::{
        icons,
        opt_helpers::{self, DisableArgs},
    },
};

use std::sync::LazyLock;

use iced::{
    Fill, Task,
    widget::{button, center, opaque, row, space},
};
use komorebi_client::{ThemeOptions, Wallpaper};
use komorebi_themes::{Base16Value, ThemeVariant};

pub static DEFAULT_WALLPAPER: LazyLock<Wallpaper> = LazyLock::new(|| Wallpaper {
    path: PathBuf::new(),
    generate_theme: Some(true),
    theme_options: Some(DEFAULT_THEME_OPTIONS.clone()),
});

pub static DEFAULT_THEME_OPTIONS: ThemeOptions = ThemeOptions {
    theme_variant: Some(ThemeVariant::Dark),
    single_border: Some(Base16Value::Base0D),
    stack_border: Some(Base16Value::Base0B),
    monocle_border: Some(Base16Value::Base0F),
    floating_border: Some(Base16Value::Base09),
    unfocused_border: Some(Base16Value::Base01),
    unfocused_locked_border: Some(Base16Value::Base08),
    stackbar_focused_text: Some(Base16Value::Base0B),
    stackbar_unfocused_text: Some(Base16Value::Base05),
    stackbar_background: Some(Base16Value::Base01),
    bar_accent: Some(Base16Value::Base0D),
};

#[derive(Clone, Debug, Default)]
pub struct WallpaperScreen {
    show_picker: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Path(String),
    GenerateTheme(Option<bool>),
    ThemeVariant(Option<ThemeVariant>),
    Color(ThemeColor),
    PickFile,
    PickedFile(PathBuf),
    ClosedFilePicker,
}

#[derive(Debug, Clone)]
pub enum ThemeColor {
    SingleBorder(Option<Base16Value>),
    StackBorder(Option<Base16Value>),
    MonocleBorder(Option<Base16Value>),
    FloatingBorder(Option<Base16Value>),
    UnfocusedBorder(Option<Base16Value>),
    UnfocusedLockerBorder(Option<Base16Value>),
    StackbarFocusedText(Option<Base16Value>),
    StackbarUnfocusedText(Option<Base16Value>),
    StackbarBackground(Option<Base16Value>),
    BarAccent(Option<Base16Value>),
}

impl WallpaperScreen {
    pub fn update(&mut self, wallpaper: &mut Wallpaper, message: Message) -> Task<Message> {
        match message {
            Message::Path(path) => {
                let path = PathBuf::from(path);
                wallpaper.path = path;
            }
            Message::GenerateTheme(generate) => wallpaper.generate_theme = generate,
            Message::ThemeVariant(theme_variant) => {
                if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                    theme_options.theme_variant = theme_variant;
                } else {
                    wallpaper.theme_options = Some(ThemeOptions {
                        theme_variant,
                        ..DEFAULT_THEME_OPTIONS
                    })
                }
            }
            Message::Color(theme_color) => match theme_color {
                ThemeColor::SingleBorder(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.single_border = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            single_border: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::StackBorder(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.stack_border = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            stack_border: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::MonocleBorder(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.monocle_border = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            monocle_border: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::FloatingBorder(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.floating_border = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            floating_border: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::UnfocusedBorder(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.unfocused_border = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            unfocused_border: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::UnfocusedLockerBorder(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.unfocused_locked_border = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            unfocused_locked_border: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::StackbarFocusedText(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.stackbar_focused_text = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            stackbar_focused_text: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::StackbarUnfocusedText(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.stackbar_unfocused_text = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            stackbar_unfocused_text: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::StackbarBackground(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.stackbar_background = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            stackbar_background: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
                ThemeColor::BarAccent(value) => {
                    if let Some(theme_options) = wallpaper.theme_options.as_mut() {
                        theme_options.bar_accent = value;
                    } else {
                        wallpaper.theme_options = Some(ThemeOptions {
                            bar_accent: value,
                            ..DEFAULT_THEME_OPTIONS
                        })
                    }
                }
            },
            Message::PickFile => {
                self.show_picker = true;
                return pick_file();
            }
            Message::PickedFile(path) => {
                wallpaper.path = path;
                self.show_picker = false;
            }
            Message::ClosedFilePicker => self.show_picker = false,
        }
        Task::none()
    }

    pub fn view<'a>(&self, wallpaper: &'a Wallpaper) -> View<'a, Message> {
        let mut contents = vec![
            opt_helpers::opt_custom_el_disable_default(
                "Path",
                Some("Full path to the wallpaper image file"),
                row![
                    button(icons::folder())
                        .style(subtle)
                        .on_press(Message::PickFile),
                    crate::widget::input(
                        "",
                        wallpaper.path.to_str().unwrap_or_default(),
                        Message::Path,
                        None,
                    )
                    .on_input(Message::Path)
                ]
                .spacing(10),
                !wallpaper.path.to_str().unwrap_or_default().is_empty(),
                Some(Message::Path(String::new())),
                DisableArgs::none(),
            ),
            opt_helpers::toggle_with_disable_default(
                "Generate Theme",
                Some("Generate and apply Base16 theme for this wallpaper (default: true)"),
                wallpaper.generate_theme,
                Some(true),
                Message::GenerateTheme,
                DisableArgs::none(),
            ),
        ];

        if wallpaper.generate_theme.unwrap_or_default() {
            contents.extend([
                opt_helpers::choose_with_disable_default(
                    "Theme Variant",
                    Some("Specify Light or Dark variant for theme generation (default: Dark)"),
                    vec![],
                    [ThemeVariant::Dark, ThemeVariant::Light],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.theme_variant),
                    Message::ThemeVariant,
                    DEFAULT_THEME_OPTIONS.theme_variant,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Single Border",
                    Some("Border colour when the container contains a single window (default: Base0D)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.single_border),
                    |v| Message::Color(ThemeColor::SingleBorder(v)),
                    DEFAULT_THEME_OPTIONS.single_border,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Stack Border",
                    Some("Border colour when the container contains multiple windows (default: Base0B)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.stack_border),
                    |v| Message::Color(ThemeColor::StackBorder(v)),
                    DEFAULT_THEME_OPTIONS.stack_border,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Monocle Border",
                    Some("Border colour when the container is in monocle mode (default: Base0F)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.monocle_border),
                    |v| Message::Color(ThemeColor::MonocleBorder(v)),
                    DEFAULT_THEME_OPTIONS.monocle_border,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Floating Border",
                    Some("Border colour when the window is floating (default: Base09)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.floating_border),
                    |v| Message::Color(ThemeColor::FloatingBorder(v)),
                    DEFAULT_THEME_OPTIONS.floating_border,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Unfocused Border",
                    Some("Border colour when the container is unfocused (default: Base01)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.unfocused_border),
                    |v| Message::Color(ThemeColor::UnfocusedBorder(v)),
                    DEFAULT_THEME_OPTIONS.unfocused_border,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Unfocused Locked Border",
                    Some("Border colour when the container is unfocused and locked (default: Base08)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.unfocused_locked_border),
                    |v| Message::Color(ThemeColor::UnfocusedLockerBorder(v)),
                    DEFAULT_THEME_OPTIONS.unfocused_locked_border,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Stackbar Focused Text",
                    Some("Stackbar focused tab text colour (default: Base0B)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.stackbar_focused_text),
                    |v| Message::Color(ThemeColor::StackbarFocusedText(v)),
                    DEFAULT_THEME_OPTIONS.stackbar_focused_text,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Stackbar Unfocused Text",
                    Some("Stackbar unfocused tab text colour (default: Base05)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.stackbar_unfocused_text),
                    |v| Message::Color(ThemeColor::StackbarUnfocusedText(v)),
                    DEFAULT_THEME_OPTIONS.stackbar_unfocused_text,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Stackbar Background",
                    Some("Stackbar tab background colour (default: Base01)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.stackbar_background),
                    |v| Message::Color(ThemeColor::StackbarBackground(v)),
                    DEFAULT_THEME_OPTIONS.stackbar_background,
                    DisableArgs::none(),
                ),
                opt_helpers::choose_with_disable_default(
                    "Bar Acccent",
                    Some("Komorebi status bar accent (default: Base0D)"),
                    vec![],
                    &super::theme::BASE16_VALUE_OPTIONS[..],
                    wallpaper
                        .theme_options
                        .as_ref()
                        .and_then(|o| o.bar_accent),
                    |v| Message::Color(ThemeColor::BarAccent(v)),
                    DEFAULT_THEME_OPTIONS.bar_accent,
                    DisableArgs::none(),
                ),
            ]);
        }

        let element = opt_helpers::section_view("Wallpaper", contents);

        if self.show_picker {
            View::new(element).modal(
                Some(opaque(center(space().width(Fill).height(Fill)))),
                Message::ClosedFilePicker,
            )
        } else {
            View::new(element)
        }
    }
}

fn pick_file() -> Task<Message> {
    let (home_dir, _) = crate::config::home_path();
    Task::future(async move {
        rfd::FileDialog::new()
            .add_filter(
                "image",
                &[
                    "jpg", "jpeg", "bmp", "dib", "png", "jfif", "jpe", "gif", "tif", "tiff", "wdp",
                    "heic", "heif", "heics", "heifs", "hif", "avci", "avcs", "avif", "avifs",
                    "jxr", "jxl",
                ],
            )
            .set_directory(home_dir.as_path())
            .pick_file()
    })
    .map(|res| match res {
        Some(file) => Message::PickedFile(file),
        None => Message::ClosedFilePicker,
    })
}

fn subtle(theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: None,
            ..button::subtle(theme, status)
        },
        button::Status::Hovered | button::Status::Pressed | button::Status::Disabled => {
            button::subtle(theme, status)
        }
    }
}
