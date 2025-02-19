// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../../assets/icons.toml
// 44e0bfbac8c396ebffeb2c32793f6c4ae10bbe256138293c94ec1d4f1c296c75
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../../assets/icons.ttf");

pub fn back<'a>() -> Text<'a> {
    icon("\u{1F519}")
}

pub fn bsp<'a>() -> Text<'a> {
    icon("\u{E800}")
}

pub fn check<'a>() -> Text<'a> {
    icon("\u{2713}")
}

pub fn columns<'a>() -> Text<'a> {
    icon("\u{E801}")
}

pub fn copy<'a>() -> Text<'a> {
    icon("\u{F0C5}")
}

pub fn delete<'a>() -> Text<'a> {
    icon("\u{F1F8}")
}

pub fn down_chevron<'a>() -> Text<'a> {
    icon("\u{F107}")
}

pub fn edit<'a>() -> Text<'a> {
    icon("\u{270D}")
}

pub fn error<'a>() -> Text<'a> {
    icon("\u{2716}")
}

pub fn grid<'a>() -> Text<'a> {
    icon("\u{E802}")
}

pub fn hstack<'a>() -> Text<'a> {
    icon("\u{E803}")
}

pub fn info<'a>() -> Text<'a> {
    icon("\u{E705}")
}

pub fn level_down<'a>() -> Text<'a> {
    icon("\u{F149}")
}

pub fn level_up<'a>() -> Text<'a> {
    icon("\u{F148}")
}

pub fn paste<'a>() -> Text<'a> {
    icon("\u{F0EA}")
}

pub fn plus<'a>() -> Text<'a> {
    icon("\u{F0FE}")
}

pub fn rmvstack<'a>() -> Text<'a> {
    icon("\u{E804}")
}

pub fn rows<'a>() -> Text<'a> {
    icon("\u{E805}")
}

pub fn up_chevron<'a>() -> Text<'a> {
    icon("\u{F106}")
}

pub fn uwvstack<'a>() -> Text<'a> {
    icon("\u{E806}")
}

pub fn vstack<'a>() -> Text<'a> {
    icon("\u{E807}")
}

pub fn warning<'a>() -> Text<'a> {
    icon("\u{2757}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("icons"))
}
