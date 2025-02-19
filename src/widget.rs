mod helpers;
pub mod icons;
pub mod modal;
pub mod monitors_viewer;
pub mod opt_helpers;

pub use helpers::*;
pub use modal::modal;

use iced::{Color, Font};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref YELLOW: Color = Color::from_rgba8(0xEE, 0xD2, 0x02, 1.0);
    pub static ref RED: Color = Color::from_rgba8(0xCF, 0x14, 0x2B, 1.0);
    pub static ref BLUE: Color = Color::from_rgba8(0x2E, 0x67, 0xF8, 1.0);
}

pub const ICONS: Font = Font::with_name("icons");
