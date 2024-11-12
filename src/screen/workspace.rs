use iced::widget::column;
use iced::{widget::text, Element};
use komorebi::WorkspaceConfig;
use komorebi_client::DefaultLayout;

use crate::utils::DisplayOptionCustom as DisplayOption;
use crate::widget::opt_helpers;

use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_LAYOUT_OPTIONS: [DisplayOption<DefaultLayout>; 9] = [
        DisplayOption(None, "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::BSP), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::VerticalStack), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::RightMainVerticalStack), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::UltrawideVerticalStack), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::HorizontalStack), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Rows), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Columns), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Grid), "[None] (Floating)"),
    ];
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleExpanded(usize),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    ToggleExpanded(usize),
}

#[derive(Clone, Debug)]
pub enum ConfigChange {
    ApplyWindowBasedWorkAreaOffset(bool),
    ContainerPadding(i32),
    FloatOverride(bool),
    Layout(Option<DefaultLayout>),
    Name(String),
    WindowContainerBehaviour(komorebi::WindowContainerBehaviour),
    WorkspacePadding(i32),
}

pub trait WorkspaceScreen {
    fn update(&mut self, message: Message) -> Action;

    fn view(&self, idx: usize, expanded: bool) -> Element<Message>;
}

impl WorkspaceScreen for WorkspaceConfig {
    fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ConfigChange(change) => match change {
                ConfigChange::ApplyWindowBasedWorkAreaOffset(value) => {
                    self.apply_window_based_work_area_offset = Some(value)
                }
                ConfigChange::ContainerPadding(value) => self.container_padding = Some(value),
                ConfigChange::FloatOverride(value) => self.float_override = Some(value),
                ConfigChange::Layout(value) => self.layout = value,
                ConfigChange::Name(value) => self.name = value,
                ConfigChange::WindowContainerBehaviour(value) => {
                    self.window_container_behaviour = Some(value)
                }
                ConfigChange::WorkspacePadding(value) => self.workspace_padding = Some(value),
            },
            Message::ToggleExpanded(idx) => {
                return Action::ToggleExpanded(idx);
            }
        }
        Action::None
    }

    fn view(&self, idx: usize, expanded: bool) -> Element<Message> {
        let title = text!("Workspace [{}] - \"{}\":", idx, self.name).size(20);
        let name = opt_helpers::input(
            "Name:",
            Some("Name of the workspace. Should be unique."),
            "",
            &self.name,
            |v| Message::ConfigChange(ConfigChange::Name(v)),
            None,
        );
        let layout = opt_helpers::choose(
            "Layout:",
            Some("Layout (default: BSP)"),
            &DEFAULT_LAYOUT_OPTIONS[..],
            Some(DisplayOption(self.layout, "[None] (Floating)")),
            |s| Message::ConfigChange(ConfigChange::Layout(s.0)),
        );
        // opt_helpers::expandable(
        //     title,
        //     None,
        //     [name, layout],
        //     expanded,
        //     Message::ToggleExpanded(idx),
        // )
        column![title, name, layout].into()
    }
}
