use crate::Screen;

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
                if screen != self.selected_screen {
                    self.selected_screen = screen.clone();
                    return (Action::UpdateMainScreen(screen), Task::none());
                }
            }
        }
        (Action::None, Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        let home = screen_button("Home", Screen::Home);
        let general = screen_button("General", Screen::General);
        let monitors = screen_button("Monitors", Screen::Monitors);
        let border = screen_button("Border", Screen::Border);
        let stackbar = screen_button("Stackbar", Screen::Stackbar);
        let transparency = screen_button("Transparency", Screen::Transparency);
        let rules = screen_button("Rules", Screen::Rules);
        let fixed_width = Space::new(120, Shrink);
        column![
            fixed_width,
            home,
            general,
            monitors,
            border,
            stackbar,
            transparency,
            rules,
        ]
        .spacing(10)
        .width(Shrink)
        .into()
    }
}

fn screen_button(name: &str, screen: Screen) -> Container<Message> {
    container(
        button(name)
            .on_press(Message::SelectScreen(screen))
            .width(Fill),
    )
    .width(Fill)
}
