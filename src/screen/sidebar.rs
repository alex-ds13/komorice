use crate::{Screen, SCREENS_TO_RESET};

use iced::{
    widget::{button, column, container, Container, Space},
    Element,
    Length::{Fill, Shrink},
    Task,
};

#[derive(Clone, Debug)]
pub enum Message {
    SelectScreen(Screen),
}

#[derive(Clone, Debug)]
pub enum Action {
    None,
    UpdateMainScreen(Screen),
}

#[derive(Default)]
pub struct Sidebar {
    pub selected_screen: Screen,
}

impl Sidebar {
    pub fn update(&mut self, message: Message) -> (Action, Task<Message>) {
        match message {
            Message::SelectScreen(screen) => {
                if screen != self.selected_screen || SCREENS_TO_RESET.contains(&screen) {
                    self.selected_screen = screen.clone();
                    return (Action::UpdateMainScreen(screen), Task::none());
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        let home = screen_button(Screen::Home, &self.selected_screen);
        let general = screen_button(Screen::General, &self.selected_screen);
        let monitors = screen_button(Screen::Monitors, &self.selected_screen);
        let border = screen_button(Screen::Border, &self.selected_screen);
        let stackbar = screen_button(Screen::Stackbar, &self.selected_screen);
        let transparency = screen_button(Screen::Transparency, &self.selected_screen);
        let animation = screen_button(Screen::Animations, &self.selected_screen);
        let theme = screen_button(Screen::Theme, &self.selected_screen);
        let rules = screen_button(Screen::Rules, &self.selected_screen);
        let debug = screen_button(Screen::Debug, &self.selected_screen);
        let settings = screen_button(Screen::Settings, &self.selected_screen);
        let fixed_width = Space::new(120, Shrink);
        column![
            fixed_width,
            home,
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
        .width(Shrink)
        .into()
    }
}

fn screen_button(screen: Screen, selected: &Screen) -> Container<Message> {
    let is_selected = &screen == selected;
    container(
        button(&screen)
            .on_press(Message::SelectScreen(screen))
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
