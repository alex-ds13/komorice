#![allow(dead_code)]
use iced::widget::Text;
use iced::{Center, Color, Font};
use lazy_static::lazy_static;

pub const ICONS: Font = Font::with_name("icons");
lazy_static! {
    pub static ref YELLOW: Color = Color::from_rgba8(0xEE, 0xD2, 0x02, 1.0);
    pub static ref RED: Color = Color::from_rgba8(0xCF, 0x14, 0x2B, 1.0);
    pub static ref BLUE: Color = Color::from_rgba8(0x2E, 0x67, 0xF8, 1.0);
}

pub fn icon<'a>(unicode: char) -> Text<'a> {
    Text::new(unicode.to_string()).font(ICONS).align_x(Center)
}

pub fn down_chevron_icon<'a>() -> Text<'a> {
    icon('\u{F107}')
}

pub fn up_chevron_icon<'a>() -> Text<'a> {
    // icon('\u{F139}')
    icon('\u{F106}')
}

pub fn left_chevron_icon<'a>() -> Text<'a> {
    icon('\u{F104}')
}

pub fn right_chevron_icon<'a>() -> Text<'a> {
    icon('\u{F105}')
}

pub fn gear_icon<'a>() -> Text<'a> {
    icon('\u{E813}')
}

pub fn delete_icon<'a>() -> Text<'a> {
    icon('\u{F1F8}')
    // icon('\u{E80E}')
}

pub fn error_icon<'a>() -> Text<'a> {
    icon('\u{F057}')
}

pub fn warning_icon<'a>() -> Text<'a> {
    icon('\u{E80D}')
}

pub fn info_icon<'a>() -> Text<'a> {
    icon('\u{F05A}')
}

pub fn folder_icon<'a>() -> Text<'a> {
    icon('\u{E811}')
}

pub fn reload_icon<'a>() -> Text<'a> {
    icon('\u{E81E}')
}

pub fn lock_icon<'a>() -> Text<'a> {
    icon('\u{E829}')
}

pub fn unlock_icon<'a>() -> Text<'a> {
    icon('\u{E82A}')
}

pub fn back_icon<'a>() -> Text<'a> {
    icon('\u{E826}')
}

pub fn plus_icon<'a>() -> Text<'a> {
    icon('\u{F0FE}')
}

pub fn minus_icon<'a>() -> Text<'a> {
    icon('\u{F146}')
}

pub fn cross_icon<'a>() -> Text<'a> {
    icon('\u{E807}')
}

pub fn check_icon<'a>() -> Text<'a> {
    icon('\u{E806}')
}

pub fn level_up_icon<'a>() -> Text<'a> {
    icon('\u{F148}')
}

pub fn level_down_icon<'a>() -> Text<'a> {
    icon('\u{F149}')
}

pub fn edit_icon<'a>() -> Text<'a> {
    icon('\u{E80C}')
}

pub fn copy_icon<'a>() -> Text<'a> {
    icon('\u{F0C5}')
}

pub fn paste_icon<'a>() -> Text<'a> {
    icon('\u{F0EA}')
}

pub fn bsp_icon<'a>() -> Text<'a> {
    icon('\u{E82B}')
}

pub fn vstack_icon<'a>() -> Text<'a> {
    icon('\u{E82C}')
}

pub fn rmvstack_icon<'a>() -> Text<'a> {
    icon('\u{E82D}')
}

pub fn grid_icon<'a>() -> Text<'a> {
    icon('\u{E82E}')
}

pub fn hstack<'a>() -> Text<'a> {
    icon('\u{E82F}')
}

pub fn uwvstack_icon<'a>() -> Text<'a> {
    icon('\u{E830}')
}

pub fn columns_icon<'a>() -> Text<'a> {
    icon('\u{E831}')
}

pub fn rows_icon<'a>() -> Text<'a> {
    icon('\u{E832}')
}
