use super::{Screen, SidebarAction, SidebarMessage};

use crate::SCREENS_TO_RESET;

use iced::{
    padding,
    widget::{button, column, container, horizontal_rule, scrollable, Container, Space},
    Element,
    Length::{Fill, Shrink},
    Task,
};

#[derive(Debug, Clone)]
pub struct WhkdSidebar {
    pub selected_screen: Screen,
}

impl Default for WhkdSidebar {
    fn default() -> Self {
        Self {
            selected_screen: Screen::Whkd,
        }
    }
}

impl WhkdSidebar {
    pub fn update(&mut self, message: SidebarMessage) -> (SidebarAction, Task<SidebarMessage>) {
        match message {
            SidebarMessage::SelectScreen(screen) => {
                if screen != self.selected_screen || SCREENS_TO_RESET.contains(&screen) {
                    self.selected_screen = screen.clone();
                    return (SidebarAction::UpdateMainScreen(screen), Task::none());
                }
            }
        }
        (SidebarAction::None, Task::none())
    }

    pub fn view(&self) -> Element<SidebarMessage> {
        let home = screen_button(Screen::Home, &self.selected_screen);
        let whkd = screen_button(Screen::Whkd, &self.selected_screen);
        let fixed_width = Space::new(120, Shrink);
        let main_content = scrollable(
            column![fixed_width, whkd]
                .spacing(10)
                .padding(padding::right(15).left(5).bottom(0))
                .width(Shrink),
        )
        .height(Fill);
        let fixed_width_wider = Space::new(135, Shrink);
        let fixed_width = Space::new(120, Shrink);
        let bottom_content = column![
            fixed_width_wider,
            container(horizontal_rule(2.0)).padding(padding::bottom(5)),
            column![fixed_width, home].width(Shrink),
        ]
        .width(Shrink)
        .padding(padding::left(5));

        column![main_content, bottom_content].into()
    }
}

fn screen_button(screen: Screen, selected: &Screen) -> Container<SidebarMessage> {
    let is_selected = &screen == selected;
    container(
        button(&screen)
            .on_press(SidebarMessage::SelectScreen(screen))
            .style(move |t, s| {
                if is_selected {
                    button::primary(t, s)
                } else {
                    button::secondary(t, s)
                }
            })
            .width(Fill),
    )
    .width(Fill)
}
