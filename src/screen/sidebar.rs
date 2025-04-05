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
pub struct Sidebar {
    pub selected_screen: Screen,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            selected_screen: Screen::General,
        }
    }
}

impl Sidebar {
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
        let general = screen_button(Screen::General, &self.selected_screen);
        let monitors = screen_button(Screen::Monitors, &self.selected_screen);
        let border = screen_button(Screen::Border, &self.selected_screen);
        let stackbar = screen_button(Screen::Stackbar, &self.selected_screen);
        let transparency = screen_button(Screen::Transparency, &self.selected_screen);
        let animation = screen_button(Screen::Animations, &self.selected_screen);
        let theme = screen_button(Screen::Theme, &self.selected_screen);
        let rules = screen_button(Screen::Rules, &self.selected_screen);
        let debug = screen_button(Screen::LiveDebug, &self.selected_screen);
        let settings = screen_button(Screen::Settings, &self.selected_screen);
        let fixed_width = Space::new(120, Shrink);
        let main_content = scrollable(
            column![
                fixed_width,
                general,
                monitors,
                border,
                stackbar,
                transparency,
                animation,
                theme,
                rules,
                debug,
                settings,
            ]
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
