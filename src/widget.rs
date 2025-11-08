pub mod color_picker;
pub mod expandable;
mod helpers;
pub mod hover;
pub mod icons;
pub mod modal;
pub mod monitors_viewer;
pub mod number_input;
pub mod opt_button;
pub mod opt_helpers;
pub mod text_input;

pub use helpers::*;
pub use modal::modal;
pub use opt_button::opt_button;
pub use hover::hover;

use std::fmt::Display;
use std::str::FromStr;

use iced::{Color, Font};
use lazy_static::lazy_static;
use num_traits::{Bounded, Num, NumAssignOps};

lazy_static! {
    pub static ref YELLOW: Color = Color::from_rgba8(0xEE, 0xD2, 0x02, 1.0);
    pub static ref RED: Color = Color::from_rgba8(0xCF, 0x14, 0x2B, 1.0);
    pub static ref BLUE: Color = Color::from_rgba8(0x2E, 0x67, 0xF8, 1.0);
}

pub const ICONS: Font = Font::with_name("icons");

/// Creates a new [`NumberInput`].
///
/// Number inputs display fields that can be filled with numbers.
pub fn number_input<'a, T, Message, Theme, Renderer>(
    placeholder: &str,
    value: T,
) -> number_input::NumberInput<'a, T, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: number_input::Catalog + iced::widget::button::Catalog + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer + 'a,
    T: Num + NumAssignOps + PartialOrd + Display + FromStr + Clone + Default + Bounded + 'a,
{
    number_input::NumberInput::new(placeholder, value)
}
