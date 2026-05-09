use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ASPECT_RATIO_OPTIONS: [AspectRatio; 4] = [
        AspectRatio::Standard,
        AspectRatio::Widescreen,
        AspectRatio::Ultrawide,
        AspectRatio::Custom(4, 3),
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AspectRatio {
    #[default]
    Standard,
    Widescreen,
    Ultrawide,
    Custom(i32, i32),
}

impl From<komorebi_client::AspectRatio> for AspectRatio {
    fn from(value: komorebi_client::AspectRatio) -> Self {
        match value {
            komorebi_client::AspectRatio::Predefined(predefined_aspect_ratio) => {
                match predefined_aspect_ratio {
                    komorebi_client::PredefinedAspectRatio::Ultrawide => AspectRatio::Ultrawide,
                    komorebi_client::PredefinedAspectRatio::Widescreen => AspectRatio::Widescreen,
                    komorebi_client::PredefinedAspectRatio::Standard => AspectRatio::Standard,
                }
            }
            komorebi_client::AspectRatio::Custom(w, h) => AspectRatio::Custom(w, h),
        }
    }
}

impl From<AspectRatio> for komorebi_client::AspectRatio {
    fn from(value: AspectRatio) -> Self {
        match value {
            AspectRatio::Standard => komorebi_client::AspectRatio::Predefined(
                komorebi_client::PredefinedAspectRatio::Standard,
            ),
            AspectRatio::Widescreen => komorebi_client::AspectRatio::Predefined(
                komorebi_client::PredefinedAspectRatio::Widescreen,
            ),
            AspectRatio::Ultrawide => komorebi_client::AspectRatio::Predefined(
                komorebi_client::PredefinedAspectRatio::Ultrawide,
            ),
            AspectRatio::Custom(w, h) => komorebi_client::AspectRatio::Custom(w, h),
        }
    }
}

impl Display for AspectRatio {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AspectRatio::Standard => write!(f, "Standard (4:3)"),
            AspectRatio::Widescreen => write!(f, "Widescreen (16:9)"),
            AspectRatio::Ultrawide => write!(f, "Ultrawide (21:9)"),
            AspectRatio::Custom(_, _) => write!(f, "Custom"),
        }
    }
}
