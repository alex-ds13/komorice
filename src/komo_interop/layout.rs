use crate::utils::{DisplayOption, DisplayOptionCustom};

use std::fmt::{Display, Formatter};

use komorebi_client::{Axis, DefaultLayout};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LAYOUT_OPTIONS: [DisplayOptionCustom<Layout>; 10] = [
        DisplayOptionCustom(None, "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::BSP), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::VerticalStack), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::RightMainVerticalStack), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::UltrawideVerticalStack), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::HorizontalStack), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::Rows), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::Columns), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::Grid), "[None] (Floating)"),
        DisplayOptionCustom(Some(Layout::Scrolling), "[None] (Floating)"),
    ];
    pub static ref LAYOUT_OPTIONS_WITHOUT_NONE: [Layout; 9] = [
        Layout::BSP,
        Layout::VerticalStack,
        Layout::RightMainVerticalStack,
        Layout::UltrawideVerticalStack,
        Layout::HorizontalStack,
        Layout::Rows,
        Layout::Columns,
        Layout::Grid,
        Layout::Scrolling,
    ];
    pub static ref LAYOUT_FLIP_OPTIONS: [DisplayOption<Axis>; 4] = [
        DisplayOption(None),
        DisplayOption(Some(Axis::Vertical)),
        DisplayOption(Some(Axis::Horizontal)),
        DisplayOption(Some(Axis::HorizontalAndVertical)),
    ];
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Layout {
    #[default]
    BSP,
    VerticalStack,
    RightMainVerticalStack,
    UltrawideVerticalStack,
    HorizontalStack,
    Rows,
    Columns,
    Grid,
    Scrolling,
}

impl From<DefaultLayout> for Layout {
    fn from(value: DefaultLayout) -> Self {
        match value {
            DefaultLayout::BSP => Layout::BSP,
            DefaultLayout::Columns => Layout::Columns,
            DefaultLayout::Rows => Layout::Rows,
            DefaultLayout::VerticalStack => Layout::VerticalStack,
            DefaultLayout::HorizontalStack => Layout::HorizontalStack,
            DefaultLayout::UltrawideVerticalStack => Layout::UltrawideVerticalStack,
            DefaultLayout::Grid => Layout::Grid,
            DefaultLayout::RightMainVerticalStack => Layout::RightMainVerticalStack,
            DefaultLayout::Scrolling => Layout::Scrolling,
        }
    }
}

impl From<Layout> for DefaultLayout {
    fn from(value: Layout) -> Self {
        match value {
            Layout::BSP => DefaultLayout::BSP,
            Layout::Columns => DefaultLayout::Columns,
            Layout::Rows => DefaultLayout::Rows,
            Layout::VerticalStack => DefaultLayout::VerticalStack,
            Layout::HorizontalStack => DefaultLayout::HorizontalStack,
            Layout::UltrawideVerticalStack => DefaultLayout::UltrawideVerticalStack,
            Layout::Grid => DefaultLayout::Grid,
            Layout::RightMainVerticalStack => DefaultLayout::RightMainVerticalStack,
            Layout::Scrolling => DefaultLayout::Scrolling,
        }
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::BSP => write!(f, "\u{E800} BSP"),
            Layout::Columns => write!(f, "\u{E801} Columns"),
            Layout::Rows => write!(f, "\u{E805} Rows"),
            Layout::VerticalStack => write!(f, "\u{E808} VerticalStack"),
            Layout::HorizontalStack => write!(f, "\u{E803} HorizontalStack"),
            Layout::UltrawideVerticalStack => write!(f, "\u{E807} UltrawideVerticalStack"),
            Layout::Grid => write!(f, "\u{E802} Grid"),
            Layout::RightMainVerticalStack => write!(f, "\u{E804} RightMainVerticalStack"),
            Layout::Scrolling => write!(f, "\u{E806} Scrolling"),
        }
    }
}
