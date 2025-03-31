use crate::config::{DEFAULT_BASE16_THEME, DEFAULT_CATPPUCCIN_THEME};
use crate::widget::opt_helpers;

use iced::{widget::combo_box, Element, Task};
use komorebi_client::{KomorebiTheme, StaticConfig};
use komorebi_themes::{Base16, Base16Value, Catppuccin, CatppuccinValue};
use lazy_static::lazy_static;

lazy_static! {
    static ref CATPPUCCIN_OPTIONS: [Catppuccin; 4] = [
        Catppuccin::Frappe,
        Catppuccin::Latte,
        Catppuccin::Macchiato,
        Catppuccin::Mocha,
    ];
    static ref CATPPUCCIN_VALUE_OPTIONS: [CatppuccinValue; 26] = [
        CatppuccinValue::Rosewater,
        CatppuccinValue::Flamingo,
        CatppuccinValue::Pink,
        CatppuccinValue::Mauve,
        CatppuccinValue::Red,
        CatppuccinValue::Maroon,
        CatppuccinValue::Peach,
        CatppuccinValue::Yellow,
        CatppuccinValue::Green,
        CatppuccinValue::Teal,
        CatppuccinValue::Sky,
        CatppuccinValue::Sapphire,
        CatppuccinValue::Blue,
        CatppuccinValue::Lavender,
        CatppuccinValue::Text,
        CatppuccinValue::Subtext1,
        CatppuccinValue::Subtext0,
        CatppuccinValue::Overlay2,
        CatppuccinValue::Overlay1,
        CatppuccinValue::Overlay0,
        CatppuccinValue::Surface2,
        CatppuccinValue::Surface1,
        CatppuccinValue::Surface0,
        CatppuccinValue::Base,
        CatppuccinValue::Mantle,
        CatppuccinValue::Crust,
    ];
    static ref BASE16_OPTIONS: [Base16; 269] = [
        Base16::_3024,
        Base16::Apathy,
        Base16::Apprentice,
        Base16::Ashes,
        Base16::AtelierCaveLight,
        Base16::AtelierCave,
        Base16::AtelierDuneLight,
        Base16::AtelierDune,
        Base16::AtelierEstuaryLight,
        Base16::AtelierEstuary,
        Base16::AtelierForestLight,
        Base16::AtelierForest,
        Base16::AtelierHeathLight,
        Base16::AtelierHeath,
        Base16::AtelierLakesideLight,
        Base16::AtelierLakeside,
        Base16::AtelierPlateauLight,
        Base16::AtelierPlateau,
        Base16::AtelierSavannaLight,
        Base16::AtelierSavanna,
        Base16::AtelierSeasideLight,
        Base16::AtelierSeaside,
        Base16::AtelierSulphurpoolLight,
        Base16::AtelierSulphurpool,
        Base16::Atlas,
        Base16::AyuDark,
        Base16::AyuLight,
        Base16::AyuMirage,
        Base16::Aztec,
        Base16::Bespin,
        Base16::BlackMetalBathory,
        Base16::BlackMetalBurzum,
        Base16::BlackMetalDarkFuneral,
        Base16::BlackMetalGorgoroth,
        Base16::BlackMetalImmortal,
        Base16::BlackMetalKhold,
        Base16::BlackMetalMarduk,
        Base16::BlackMetalMayhem,
        Base16::BlackMetalNile,
        Base16::BlackMetalVenom,
        Base16::BlackMetal,
        Base16::Blueforest,
        Base16::Blueish,
        Base16::Brewer,
        Base16::Bright,
        Base16::Brogrammer,
        Base16::BrushtreesDark,
        Base16::Brushtrees,
        Base16::Caroline,
        Base16::CatppuccinFrappe,
        Base16::CatppuccinLatte,
        Base16::CatppuccinMacchiato,
        Base16::CatppuccinMocha,
        Base16::Chalk,
        Base16::Circus,
        Base16::ClassicDark,
        Base16::ClassicLight,
        Base16::Codeschool,
        Base16::Colors,
        Base16::Cupcake,
        Base16::Cupertino,
        Base16::DaOneBlack,
        Base16::DaOneGray,
        Base16::DaOneOcean,
        Base16::DaOnePaper,
        Base16::DaOneSea,
        Base16::DaOneWhite,
        Base16::DanqingLight,
        Base16::Danqing,
        Base16::Darcula,
        Base16::Darkmoss,
        Base16::Darktooth,
        Base16::Darkviolet,
        Base16::Decaf,
        Base16::DefaultDark,
        Base16::DefaultLight,
        Base16::Dirtysea,
        Base16::Dracula,
        Base16::EdgeDark,
        Base16::EdgeLight,
        Base16::Eighties,
        Base16::EmbersLight,
        Base16::Embers,
        Base16::Emil,
        Base16::EquilibriumDark,
        Base16::EquilibriumGrayDark,
        Base16::EquilibriumGrayLight,
        Base16::EquilibriumLight,
        Base16::Eris,
        Base16::Espresso,
        Base16::EvaDim,
        Base16::Eva,
        Base16::EvenokDark,
        Base16::EverforestDarkHard,
        Base16::Everforest,
        Base16::Flat,
        Base16::Framer,
        Base16::FruitSoda,
        Base16::Gigavolt,
        Base16::Github,
        Base16::GoogleDark,
        Base16::GoogleLight,
        Base16::Gotham,
        Base16::GrayscaleDark,
        Base16::GrayscaleLight,
        Base16::Greenscreen,
        Base16::Gruber,
        Base16::GruvboxDarkHard,
        Base16::GruvboxDarkMedium,
        Base16::GruvboxDarkPale,
        Base16::GruvboxDarkSoft,
        Base16::GruvboxLightHard,
        Base16::GruvboxLightMedium,
        Base16::GruvboxLightSoft,
        Base16::GruvboxMaterialDarkHard,
        Base16::GruvboxMaterialDarkMedium,
        Base16::GruvboxMaterialDarkSoft,
        Base16::GruvboxMaterialLightHard,
        Base16::GruvboxMaterialLightMedium,
        Base16::GruvboxMaterialLightSoft,
        Base16::Hardcore,
        Base16::Harmonic16Dark,
        Base16::Harmonic16Light,
        Base16::HeetchLight,
        Base16::Heetch,
        Base16::Helios,
        Base16::Hopscotch,
        Base16::HorizonDark,
        Base16::HorizonLight,
        Base16::HorizonTerminalDark,
        Base16::HorizonTerminalLight,
        Base16::HumanoidDark,
        Base16::HumanoidLight,
        Base16::IaDark,
        Base16::IaLight,
        Base16::Icy,
        Base16::Irblack,
        Base16::Isotope,
        Base16::Jabuti,
        Base16::Kanagawa,
        Base16::Katy,
        Base16::Kimber,
        Base16::Lime,
        Base16::Macintosh,
        Base16::Marrakesh,
        Base16::Materia,
        Base16::MaterialDarker,
        Base16::MaterialLighter,
        Base16::MaterialPalenight,
        Base16::MaterialVivid,
        Base16::Material,
        Base16::MeasuredDark,
        Base16::MeasuredLight,
        Base16::MellowPurple,
        Base16::MexicoLight,
        Base16::Mocha,
        Base16::Monokai,
        Base16::Moonlight,
        Base16::Mountain,
        Base16::Nebula,
        Base16::NordLight,
        Base16::Nord,
        Base16::Nova,
        Base16::Ocean,
        Base16::Oceanicnext,
        Base16::OneLight,
        Base16::OnedarkDark,
        Base16::Onedark,
        Base16::OutrunDark,
        Base16::OxocarbonDark,
        Base16::OxocarbonLight,
        Base16::Pandora,
        Base16::PapercolorDark,
        Base16::PapercolorLight,
        Base16::Paraiso,
        Base16::Pasque,
        Base16::Phd,
        Base16::Pico,
        Base16::Pinky,
        Base16::Pop,
        Base16::Porple,
        Base16::PreciousDarkEleven,
        Base16::PreciousDarkFifteen,
        Base16::PreciousLightWarm,
        Base16::PreciousLightWhite,
        Base16::PrimerDarkDimmed,
        Base16::PrimerDark,
        Base16::PrimerLight,
        Base16::Purpledream,
        Base16::Qualia,
        Base16::Railscasts,
        Base16::Rebecca,
        Base16::RosePineDawn,
        Base16::RosePineMoon,
        Base16::RosePine,
        Base16::Saga,
        Base16::Sagelight,
        Base16::Sakura,
        Base16::Sandcastle,
        Base16::SelenizedBlack,
        Base16::SelenizedDark,
        Base16::SelenizedLight,
        Base16::SelenizedWhite,
        Base16::Seti,
        Base16::ShadesOfPurple,
        Base16::ShadesmearDark,
        Base16::ShadesmearLight,
        Base16::Shapeshifter,
        Base16::SilkDark,
        Base16::SilkLight,
        Base16::Snazzy,
        Base16::SolarflareLight,
        Base16::Solarflare,
        Base16::SolarizedDark,
        Base16::SolarizedLight,
        Base16::Spaceduck,
        Base16::Spacemacs,
        Base16::Sparky,
        Base16::StandardizedDark,
        Base16::StandardizedLight,
        Base16::Stella,
        Base16::StillAlive,
        Base16::Summercamp,
        Base16::SummerfruitDark,
        Base16::SummerfruitLight,
        Base16::SynthMidnightDark,
        Base16::SynthMidnightLight,
        Base16::Tango,
        Base16::Tarot,
        Base16::Tender,
        Base16::TerracottaDark,
        Base16::Terracotta,
        Base16::TokyoCityDark,
        Base16::TokyoCityLight,
        Base16::TokyoCityTerminalDark,
        Base16::TokyoCityTerminalLight,
        Base16::TokyoNightDark,
        Base16::TokyoNightLight,
        Base16::TokyoNightMoon,
        Base16::TokyoNightStorm,
        Base16::TokyoNightTerminalDark,
        Base16::TokyoNightTerminalLight,
        Base16::TokyoNightTerminalStorm,
        Base16::TokyodarkTerminal,
        Base16::Tokyodark,
        Base16::TomorrowNightEighties,
        Base16::TomorrowNight,
        Base16::Tomorrow,
        Base16::Tube,
        Base16::Twilight,
        Base16::UnikittyDark,
        Base16::UnikittyLight,
        Base16::UnikittyReversible,
        Base16::Uwunicorn,
        Base16::Vesper,
        Base16::Vice,
        Base16::Vulcan,
        Base16::Windows10Light,
        Base16::Windows10,
        Base16::Windows95Light,
        Base16::Windows95,
        Base16::WindowsHighcontrastLight,
        Base16::WindowsHighcontrast,
        Base16::WindowsNtLight,
        Base16::WindowsNt,
        Base16::Woodland,
        Base16::XcodeDusk,
        Base16::Zenbones,
        Base16::Zenburn,
    ];
    static ref BASE16_VALUE_OPTIONS: [Base16Value; 16] = [
        Base16Value::Base00,
        Base16Value::Base01,
        Base16Value::Base02,
        Base16Value::Base03,
        Base16Value::Base04,
        Base16Value::Base05,
        Base16Value::Base06,
        Base16Value::Base07,
        Base16Value::Base08,
        Base16Value::Base09,
        Base16Value::Base0A,
        Base16Value::Base0B,
        Base16Value::Base0C,
        Base16Value::Base0D,
        Base16Value::Base0E,
        Base16Value::Base0F,
    ];
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ThemeType {
    #[default]
    None,
    Catppuccin,
    Base16,
}
impl std::fmt::Display for ThemeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeType::None => write!(f, "[None]"),
            ThemeType::Catppuccin => write!(f, "Catppuccin"),
            ThemeType::Base16 => write!(f, "Base16"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    ChangeThemeType(Option<ThemeType>),
    ChangeCatppuccinThemeName(Option<Catppuccin>),
    ChangeCatppuccinThemeSingleBorder(Option<CatppuccinValue>),
    ChangeCatppuccinThemeStackBorder(Option<CatppuccinValue>),
    ChangeCatppuccinThemeMonocleBorder(Option<CatppuccinValue>),
    ChangeCatppuccinThemeFloatingBorder(Option<CatppuccinValue>),
    ChangeCatppuccinThemeUnfocusedBorder(Option<CatppuccinValue>),
    ChangeCatppuccinThemeUnfocusedLockedBorder(Option<CatppuccinValue>),
    ChangeCatppuccinThemeStackbarFocusedText(Option<CatppuccinValue>),
    ChangeCatppuccinThemeStackbarUnfocusedText(Option<CatppuccinValue>),
    ChangeCatppuccinThemeStackbarBackground(Option<CatppuccinValue>),
    ChangeCatppuccinThemeBarAccent(Option<CatppuccinValue>),
    ChangeBase16ThemeName(Option<Base16>),
    ChangeBase16ThemeSingleBorder(Option<Base16Value>),
    ChangeBase16ThemeStackBorder(Option<Base16Value>),
    ChangeBase16ThemeMonocleBorder(Option<Base16Value>),
    ChangeBase16ThemeFloatingBorder(Option<Base16Value>),
    ChangeBase16ThemeUnfocusedBorder(Option<Base16Value>),
    ChangeBase16ThemeUnfocusedLockedBorder(Option<Base16Value>),
    ChangeBase16ThemeStackbarFocusedText(Option<Base16Value>),
    ChangeBase16ThemeStackbarUnfocusedText(Option<Base16Value>),
    ChangeBase16ThemeStackbarBackground(Option<Base16Value>),
    ChangeBase16ThemeBarAccent(Option<Base16Value>),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
}

#[derive(Debug)]
pub struct Theme {
    base16_state: combo_box::State<Base16>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            base16_state: combo_box::State::new(BASE16_OPTIONS.to_vec()),
        }
    }
}

impl Theme {
    pub fn update(
        &mut self,
        message: Message,
        config: &mut StaticConfig,
    ) -> (Action, Task<Message>) {
        match message {
            Message::ChangeThemeType(theme_type) => {
                if let Some(theme_type) = theme_type {
                    match theme_type {
                        ThemeType::Catppuccin => config.theme = Some(*DEFAULT_CATPPUCCIN_THEME),
                        ThemeType::Base16 => config.theme = Some(*DEFAULT_BASE16_THEME),
                        ThemeType::None => config.theme = None,
                    }
                } else {
                    config.theme = None;
                }
            }
            Message::ChangeCatppuccinThemeName(name) => {
                if let Some(name) = name {
                    if let Some(KomorebiTheme::Catppuccin {
                        name: _,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    }) = config.theme
                    {
                        config.theme = Some(KomorebiTheme::Catppuccin {
                            name,
                            single_border,
                            stack_border,
                            monocle_border,
                            floating_border,
                            unfocused_border,
                            unfocused_locked_border,
                            stackbar_focused_text,
                            stackbar_unfocused_text,
                            stackbar_background,
                            bar_accent,
                        });
                    }
                }
            }
            Message::ChangeCatppuccinThemeSingleBorder(single_border) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border: _,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeStackBorder(stack_border) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border: _,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeMonocleBorder(monocle_border) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border: _,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeFloatingBorder(floating_border) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border: _,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeUnfocusedBorder(unfocused_border) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border: _,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeUnfocusedLockedBorder(unfocused_locked_border) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border: _,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeStackbarFocusedText(stackbar_focused_text) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text: _,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeStackbarUnfocusedText(stackbar_unfocused_text) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text: _,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeStackbarBackground(stackbar_background) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background: _,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeCatppuccinThemeBarAccent(bar_accent) => {
                if let Some(KomorebiTheme::Catppuccin {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent: _,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeName(name) => {
                if let Some(name) = name {
                    if let Some(KomorebiTheme::Base16 {
                        name: _,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    }) = config.theme
                    {
                        config.theme = Some(KomorebiTheme::Base16 {
                            name,
                            single_border,
                            stack_border,
                            monocle_border,
                            floating_border,
                            unfocused_border,
                            unfocused_locked_border,
                            stackbar_focused_text,
                            stackbar_unfocused_text,
                            stackbar_background,
                            bar_accent,
                        });
                    }
                }
            }
            Message::ChangeBase16ThemeSingleBorder(single_border) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border: _,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeStackBorder(stack_border) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border: _,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeMonocleBorder(monocle_border) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border: _,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeFloatingBorder(floating_border) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border: _,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeUnfocusedBorder(unfocused_border) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border: _,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeUnfocusedLockedBorder(unfocused_locked_border) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border: _,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeStackbarFocusedText(stackbar_focused_text) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text: _,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeStackbarUnfocusedText(stackbar_unfocused_text) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text: _,
                    stackbar_background,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeStackbarBackground(stackbar_background) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background: _,
                    bar_accent,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
            Message::ChangeBase16ThemeBarAccent(bar_accent) => {
                if let Some(KomorebiTheme::Base16 {
                    name,
                    single_border,
                    stack_border,
                    monocle_border,
                    floating_border,
                    unfocused_border,
                    unfocused_locked_border,
                    stackbar_focused_text,
                    stackbar_unfocused_text,
                    stackbar_background,
                    bar_accent: _,
                }) = config.theme
                {
                    config.theme = Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    });
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view<'a>(&'a self, config: &'a StaticConfig) -> Element<'a, Message> {
        let theme_type = match config.theme.as_ref() {
            Some(KomorebiTheme::Catppuccin { .. }) => ThemeType::Catppuccin,
            Some(KomorebiTheme::Base16 { .. }) => ThemeType::Base16,
            None => ThemeType::None,
        };
        let mut contents: Vec<Element<Message>> = match theme_type {
            ThemeType::None => Vec::new(),
            ThemeType::Catppuccin => {
                if let (
                    Some(KomorebiTheme::Catppuccin {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    }),
                    KomorebiTheme::Catppuccin {
                        name: d_name,
                        single_border: d_single_border,
                        stack_border: d_stack_border,
                        monocle_border: d_monocle_border,
                        floating_border: d_floating_border,
                        unfocused_border: d_unfocused_border,
                        unfocused_locked_border: d_unfocused_locked_border,
                        stackbar_focused_text: d_stackbar_focused_text,
                        stackbar_unfocused_text: d_stackbar_unfocused_text,
                        stackbar_background: d_stackbar_background,
                        bar_accent: d_bar_accent,
                    },
                ) = (config.theme, *DEFAULT_CATPPUCCIN_THEME)
                {
                    let t = name.as_theme();
                    let get_color = |c: Option<CatppuccinValue>, d_c: Option<CatppuccinValue>| {
                        iced::Color::from(c.or(d_c).unwrap().color32(t).to_normalized_gamma_f32())
                    };
                    let single_border_color = get_color(single_border, d_single_border);
                    let stack_border_color = get_color(stack_border, d_stack_border);
                    let monocle_border_color = get_color(monocle_border, d_monocle_border);
                    let floating_border_color = get_color(floating_border, d_floating_border);
                    let unfocused_border_color = get_color(unfocused_border, d_unfocused_border);
                    let unfocused_locked_border_color =
                        get_color(unfocused_locked_border, d_unfocused_locked_border);
                    let stackbar_focused_text_color =
                        get_color(stackbar_focused_text, d_stackbar_focused_text);
                    let stackbar_unfocused_text_color =
                        get_color(stackbar_unfocused_text, d_stackbar_unfocused_text);
                    let stackbar_background_color =
                        get_color(stackbar_background, d_stackbar_background);
                    let bar_accent_color = get_color(bar_accent, d_bar_accent);
                    vec![
                        opt_helpers::choose_with_disable_default(
                            "Theme Name",
                            Some("The Theme variant to use"),
                            Vec::new(),
                            &CATPPUCCIN_OPTIONS[..],
                            Some(name),
                            Message::ChangeCatppuccinThemeName,
                            Some(d_name),
                            None,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Single Border",
                            Some("Border colour when the container contains a single window (default: Blue)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            single_border.or(d_single_border),
                            Message::ChangeCatppuccinThemeSingleBorder,
                            d_single_border,
                            None,
                            single_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stack Border",
                            Some("Border colour when the container contains multiple windows (default: Green)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            stack_border.or(d_stack_border),
                            Message::ChangeCatppuccinThemeStackBorder,
                            d_stack_border,
                            None,
                            stack_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Monocle Border",
                            Some("Border colour when the container is in monocle mode (default: Pink)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            monocle_border.or(d_monocle_border),
                            Message::ChangeCatppuccinThemeMonocleBorder,
                            d_monocle_border,
                            None,
                            monocle_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Floating Border",
                            Some("Border colour when the window is floating (default: Yellow)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            floating_border.or(d_floating_border),
                            Message::ChangeCatppuccinThemeFloatingBorder,
                            d_floating_border,
                            None,
                            floating_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Unfocused Border",
                            Some("Border colour when the container is unfocused (default: Base)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            unfocused_border.or(d_unfocused_border),
                            Message::ChangeCatppuccinThemeUnfocusedBorder,
                            d_unfocused_border,
                            None,
                            unfocused_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Unfocused Locked Border",
                            Some("Border colour when the container is unfocused and locked (default: Red)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            unfocused_locked_border.or(d_unfocused_locked_border),
                            Message::ChangeCatppuccinThemeUnfocusedLockedBorder,
                            d_unfocused_locked_border,
                            None,
                            unfocused_locked_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stackbar Focused Text",
                            Some("Stackbar focused tab text colour (default: Green)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            stackbar_focused_text.or(d_stackbar_focused_text),
                            Message::ChangeCatppuccinThemeStackbarFocusedText,
                            d_stackbar_focused_text,
                            None,
                            stackbar_focused_text_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stackbar Unfocused Text",
                            Some("Stackbar unfocused tab text colour (default: Text)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            stackbar_unfocused_text.or(d_stackbar_unfocused_text),
                            Message::ChangeCatppuccinThemeStackbarUnfocusedText,
                            d_stackbar_unfocused_text,
                            None,
                            stackbar_unfocused_text_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stackbar Background",
                            Some("Stackbar tab background colour (default: Base)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            stackbar_background.or(d_stackbar_background),
                            Message::ChangeCatppuccinThemeStackbarBackground,
                            d_stackbar_background,
                            None,
                            stackbar_background_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Bar Accent",
                            Some("Komorebi status bar accent (default: Blue)"),
                            Vec::new(),
                            &CATPPUCCIN_VALUE_OPTIONS[..],
                            bar_accent.or(d_bar_accent),
                            Message::ChangeCatppuccinThemeBarAccent,
                            d_bar_accent,
                            None,
                            bar_accent_color,
                        ),
                    ]
                } else {
                    Vec::new()
                }
            }
            ThemeType::Base16 => {
                if let (
                    Some(KomorebiTheme::Base16 {
                        name,
                        single_border,
                        stack_border,
                        monocle_border,
                        floating_border,
                        unfocused_border,
                        unfocused_locked_border,
                        stackbar_focused_text,
                        stackbar_unfocused_text,
                        stackbar_background,
                        bar_accent,
                    }),
                    KomorebiTheme::Base16 {
                        name: d_name,
                        single_border: d_single_border,
                        stack_border: d_stack_border,
                        monocle_border: d_monocle_border,
                        floating_border: d_floating_border,
                        unfocused_border: d_unfocused_border,
                        unfocused_locked_border: d_unfocused_locked_border,
                        stackbar_focused_text: d_stackbar_focused_text,
                        stackbar_unfocused_text: d_stackbar_unfocused_text,
                        stackbar_background: d_stackbar_background,
                        bar_accent: d_bar_accent,
                    },
                ) = (config.theme, *DEFAULT_BASE16_THEME)
                {
                    let get_color = |c: Option<Base16Value>, d_c: Option<Base16Value>| {
                        iced::Color::from(
                            c.or(d_c).unwrap().color32(name).to_normalized_gamma_f32(),
                        )
                    };
                    let single_border_color = get_color(single_border, d_single_border);
                    let stack_border_color = get_color(stack_border, d_stack_border);
                    let monocle_border_color = get_color(monocle_border, d_monocle_border);
                    let floating_border_color = get_color(floating_border, d_floating_border);
                    let unfocused_border_color = get_color(unfocused_border, d_unfocused_border);
                    let unfocused_locked_border_color =
                        get_color(unfocused_locked_border, d_unfocused_locked_border);
                    let stackbar_focused_text_color =
                        get_color(stackbar_focused_text, d_stackbar_focused_text);
                    let stackbar_unfocused_text_color =
                        get_color(stackbar_unfocused_text, d_stackbar_unfocused_text);
                    let stackbar_background_color =
                        get_color(stackbar_background, d_stackbar_background);
                    let bar_accent_color = get_color(bar_accent, d_bar_accent);
                    vec![
                        opt_helpers::combo_with_disable_default(
                            "Theme Name",
                            "",
                            Some("The Theme variant to use"),
                            Vec::new(),
                            &self.base16_state,
                            Some(name),
                            Message::ChangeBase16ThemeName,
                            Some(d_name),
                            None,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Single Border",
                            Some("Border colour when the container contains a single window (default: Base0D)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            single_border.or(d_single_border),
                            Message::ChangeBase16ThemeSingleBorder,
                            d_single_border,
                            None,
                            single_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stack Border",
                            Some("Border colour when the container contains multiple windows (default: Base0B)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            stack_border.or(d_stack_border),
                            Message::ChangeBase16ThemeStackBorder,
                            d_stack_border,
                            None,
                            stack_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Monocle Border",
                            Some("Border colour when the container is in monocle mode (default: Base0F)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            monocle_border.or(d_monocle_border),
                            Message::ChangeBase16ThemeMonocleBorder,
                            d_monocle_border,
                            None,
                            monocle_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Floating Border",
                            Some("Border colour when the window is floating (default: Base09)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            floating_border.or(d_floating_border),
                            Message::ChangeBase16ThemeFloatingBorder,
                            d_floating_border,
                            None,
                            floating_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Unfocused Border",
                            Some("Border colour when the container is unfocused (default: Base01)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            unfocused_border.or(d_unfocused_border),
                            Message::ChangeBase16ThemeUnfocusedBorder,
                            d_unfocused_border,
                            None,
                            unfocused_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Unfocused Locked Border",
                            Some("Border colour when the container is unfocused and locked (default: Base08)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            unfocused_locked_border.or(d_unfocused_locked_border),
                            Message::ChangeBase16ThemeUnfocusedLockedBorder,
                            d_unfocused_locked_border,
                            None,
                            unfocused_locked_border_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stackbar Focused Text",
                            Some("Stackbar focused tab text colour (default: Base0B)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            stackbar_focused_text.or(d_stackbar_focused_text),
                            Message::ChangeBase16ThemeStackbarFocusedText,
                            d_stackbar_focused_text,
                            None,
                            stackbar_focused_text_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stackbar Unfocused Text",
                            Some("Stackbar unfocused tab text colour (default: Base05)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            stackbar_unfocused_text.or(d_stackbar_unfocused_text),
                            Message::ChangeBase16ThemeStackbarUnfocusedText,
                            d_stackbar_unfocused_text,
                            None,
                            stackbar_unfocused_text_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Stackbar Background",
                            Some("Stackbar tab background colour (default: Base01)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            stackbar_background.or(d_stackbar_background),
                            Message::ChangeBase16ThemeStackbarBackground,
                            d_stackbar_background,
                            None,
                            stackbar_background_color,
                        ),
                        opt_helpers::choose_with_disable_default_bg(
                            "Bar Accent",
                            Some("Komorebi status bar accent (default: Base0D)"),
                            Vec::new(),
                            &BASE16_VALUE_OPTIONS[..],
                            bar_accent.or(d_bar_accent),
                            Message::ChangeBase16ThemeBarAccent,
                            d_bar_accent,
                            None,
                            bar_accent_color,
                        ),
                    ]
                } else {
                    Vec::new()
                }
            }
        };
        contents.insert(
            0,
            opt_helpers::choose_with_disable_default(
                "Theme Type",
                Some(
                    "Set a Theme to define all colours (default: None)\n\n\
                    NOTE: If you set a theme, komorebi will ignore all your other colour \
                    configs and use the ones from the selected theme.",
                ),
                Vec::new(),
                [ThemeType::None, ThemeType::Catppuccin, ThemeType::Base16],
                Some(theme_type),
                Message::ChangeThemeType,
                Some(ThemeType::None),
                None,
            ),
        );
        opt_helpers::section_view("Theme:", contents)
    }
}
