use crate::utils::DisplayOptionCustom as DisplayOption;
use crate::widget::opt_helpers;

use iced::widget::column;
use iced::Element;
use komorebi::{WindowContainerBehaviour, WorkspaceConfig};
use komorebi_client::DefaultLayout;
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_LAYOUT_OPTIONS: [DisplayOption<DefaultLayout>; 9] = [
        DisplayOption(None, "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::BSP), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::VerticalStack), "[None] (Floating)"),
        DisplayOption(
            Some(DefaultLayout::RightMainVerticalStack),
            "[None] (Floating)"
        ),
        DisplayOption(
            Some(DefaultLayout::UltrawideVerticalStack),
            "[None] (Floating)"
        ),
        DisplayOption(Some(DefaultLayout::HorizontalStack), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Rows), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Columns), "[None] (Floating)"),
        DisplayOption(Some(DefaultLayout::Grid), "[None] (Floating)"),
    ];
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigChange(ConfigChange),
    ToggleOverrideGlobal(OverrideConfig),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
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

#[derive(Clone, Debug)]
pub enum OverrideConfig {
    ContainerPadding(bool),
    FloatOverride(bool),
    WindowContainerBehaviour(bool),
    WorkspacePadding(bool),
}

pub trait WorkspaceScreen {
    fn update(&mut self, message: Message) -> Action;

    fn view(&self) -> Element<Message>;
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
            Message::ToggleOverrideGlobal(to_override) => match to_override {
                OverrideConfig::ContainerPadding(disable) => {
                    if disable {
                        self.container_padding = None;
                    } else {
                        self.container_padding = Some(10);
                    }
                }
                OverrideConfig::FloatOverride(disable) => {
                    if disable {
                        self.float_override = None;
                    } else {
                        self.float_override = Some(false);
                    }
                }
                OverrideConfig::WindowContainerBehaviour(disable) => {
                    if disable {
                        self.window_container_behaviour = None;
                    } else {
                        self.window_container_behaviour = Some(WindowContainerBehaviour::Create);
                    }
                }
                OverrideConfig::WorkspacePadding(disable) => {
                    if disable {
                        self.workspace_padding = None;
                    } else {
                        self.workspace_padding = Some(10);
                    }
                }
            },
        }
        Action::None
    }

    fn view(&self) -> Element<Message> {
        let name = opt_helpers::input(
            "Name",
            Some("Name of the workspace. Should be unique."),
            "",
            &self.name,
            |v| Message::ConfigChange(ConfigChange::Name(v)),
            None,
        );
        let layout = opt_helpers::choose(
            "Layout",
            Some("Layout (default: BSP)"),
            &DEFAULT_LAYOUT_OPTIONS[..],
            Some(DisplayOption(self.layout, "[None] (Floating)")),
            |s| Message::ConfigChange(ConfigChange::Layout(s.0)),
        );
        let apply_window_based_offset = opt_helpers::toggle(
            "Apply Window Based Work Area Offset",
            Some("Apply this monitor's window-based work area offset (default: true)"),
            self.apply_window_based_work_area_offset.unwrap_or(true),
            |v| Message::ConfigChange(ConfigChange::ApplyWindowBasedWorkAreaOffset(v)),
        );
        let container_padding = opt_helpers::number_with_disable_default(
            "Container Padding",
            Some("Container padding (default: global)"),
            self.container_padding.unwrap_or(10),
            10,
            |v| Message::ConfigChange(ConfigChange::ContainerPadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.container_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::ContainerPadding(v)),
            }),
        );
        let float_override = opt_helpers::toggle_with_disable(
            "Float Override",
            Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: global)"),
            self.float_override.unwrap_or_default(),
            |v| Message::ConfigChange(ConfigChange::FloatOverride(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.float_override.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::FloatOverride(v)),
            })
        );
        let window_container_behaviour = opt_helpers::choose_with_disable(
            "Window Container Behaviour",
            Some("Determine what happens when a new window is opened (default: global)"),
            [
                WindowContainerBehaviour::Create,
                WindowContainerBehaviour::Append,
            ],
            self.window_container_behaviour,
            |v| Message::ConfigChange(ConfigChange::WindowContainerBehaviour(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.window_container_behaviour.is_none(),
                label: Some("Global"),
                on_toggle: |v| {
                    Message::ToggleOverrideGlobal(OverrideConfig::WindowContainerBehaviour(v))
                },
            }),
        );
        let workspace_padding = opt_helpers::number_with_disable(
            "Workspace Padding",
            Some("Workspace padding (default: global)"),
            self.workspace_padding.unwrap_or(10),
            |v| Message::ConfigChange(ConfigChange::WorkspacePadding(v)),
            Some(opt_helpers::DisableArgs {
                disable: self.workspace_padding.is_none(),
                label: Some("Global"),
                on_toggle: |v| Message::ToggleOverrideGlobal(OverrideConfig::WorkspacePadding(v)),
            }),
        );
        column![
            name,
            layout,
            apply_window_based_offset,
            container_padding,
            float_override,
            window_container_behaviour,
            workspace_padding
        ]
        .spacing(10)
        .into()
    }
}
