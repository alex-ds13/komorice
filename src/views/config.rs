use crate::{config::GlobalConfigChangeType, widget::opt_helpers, Komofig, Message};

use iced::{widget::Space, Element, Length::Shrink};
use komorebi::{CrossBoundaryBehaviour, MoveBehaviour};

pub fn view(app: &Komofig) -> Element<Message> {
    view_general(app)
}

fn view_general(app: &Komofig) -> Element<Message> {
    if let Some(config) = &app.config {
        opt_helpers::section_view(
            "General:",
            [
                opt_helpers::choose(
                    "Cross Boundary Behaviour",
                    Some("Determine what happens when an action is called on a window at a monitor boundary (default: Monitor)"),
                    [CrossBoundaryBehaviour::Monitor.to_string(), CrossBoundaryBehaviour::Workspace.to_string()],
                    Some(&app.config_strs.as_ref().unwrap().global_config_strs.cross_boundary_behaviour),
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::CrossBoundaryBehaviour(selected)),
                ),
                opt_helpers::choose(
                    "Cross Monitor Move Behaviour",
                    Some("Determine what happens when a window is moved across a monitor boundary (default: Swap)"),
                    [MoveBehaviour::Swap, MoveBehaviour::Insert, MoveBehaviour::NoOp],
                    config.cross_monitor_move_behaviour,
                    |selected| Message::GlobalConfigChanged(GlobalConfigChangeType::CrossMonitorMoveBehaviour(selected)),
                ),
                opt_helpers::input(
                    "Default Container Padding",
                    Some("Global default container padding (default: 10)"),
                    "",
                    &app.config_strs.as_ref().unwrap().global_config_strs.default_container_padding,
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::DefaultContainerPadding(value)),
                    None
                ),
                opt_helpers::bool(
                    "Float Override",
                    Some("Enable or disable float override, which makes it so every new window opens in floating mode (default: false)"),
                    config.float_override.unwrap_or_default(),
                    |value| Message::GlobalConfigChanged(GlobalConfigChangeType::FloatOverride(value))
                )
            ],
        )
    } else {
        Space::new(Shrink, Shrink).into()
    }
}
